use anyhow::anyhow;
use matchit::{Match, Router};

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use bstr::ByteSlice;
use serde_json::{json, to_string};
use tower_lsp::lsp_types::*;
use tower_lsp::*;
use tower_lsp::{Client, LanguageServer};

use ignore::overrides::{Override, OverrideBuilder};
use typos_cli::policy;
pub struct Backend<'s, 'p> {
    client: Client,
    state: Mutex<BackendState<'s>>,
    default_policy: policy::Policy<'p, 'p, 'p>,
}

#[derive(Default)]
struct BackendState<'s> {
    workspace_folders: Vec<WorkspaceFolder>,
    router: Router<TyposCli<'s>>,
}

struct TyposCli<'s> {
    overrides: Override,
    engine: policy::ConfigEngine<'s>,
}

impl<'s> TryFrom<&PathBuf> for TyposCli<'s> {
    type Error = anyhow::Error;

    // initialise an engine and overrides using the config file from path or its parent
    fn try_from(path: &PathBuf) -> anyhow::Result<Self, Self::Error> {
        // leak to get a 'static which is needed to satisfy the 's lifetime
        // but does mean memory will grow unbounded
        let storage = Box::leak(Box::new(policy::ConfigStorage::new()));
        let mut engine = typos_cli::policy::ConfigEngine::new(storage);
        engine.init_dir(path)?;

        let walk_policy = engine.walk(path);

        // add any explicit excludes
        let mut overrides = OverrideBuilder::new(path);
        for pattern in walk_policy.extend_exclude.iter() {
            overrides.add(&format!("!{}", pattern))?;
        }
        let overrides = overrides.build()?;

        Ok(TyposCli { overrides, engine })
    }
}

impl<'s> BackendState<'s> {
    fn set_workspace_folders(
        &mut self,
        workspace_folders: Vec<WorkspaceFolder>,
    ) -> anyhow::Result<(), anyhow::Error> {
        self.workspace_folders = workspace_folders;
        self.update_router()?;
        Ok(())
    }

    fn update_workspace_folders(
        &mut self,
        added: Vec<WorkspaceFolder>,
        removed: Vec<WorkspaceFolder>,
    ) -> anyhow::Result<(), anyhow::Error> {
        self.workspace_folders.extend(added);
        if !removed.is_empty() {
            self.workspace_folders.retain(|x| !removed.contains(x));
        }
        self.update_router()?;
        Ok(())
    }

    fn update_router(&mut self) -> anyhow::Result<(), anyhow::Error> {
        self.router = Router::new();
        for folder in self.workspace_folders.iter() {
            let path = folder
                .uri
                .to_file_path()
                .map_err(|_| anyhow!("Cannot convert uri {} to file path", folder.uri))?;
            let path_wildcard = format!(
                "{}{}",
                path.to_str()
                    .ok_or_else(|| anyhow!("Invalid unicode in path {:?}", path))?,
                "/*p"
            );
            let config = TyposCli::try_from(&path)?;
            self.router.insert(path_wildcard, config)?;
        }
        Ok(())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DiagnosticData<'c> {
    corrections: Vec<Cow<'c, str>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend<'static, 'static> {
    async fn initialize(&self, params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        tracing::debug!("initialize: {}", to_string(&params).unwrap_or_default());

        if let Some(TextDocumentClientCapabilities {
            publish_diagnostics:
                Some(PublishDiagnosticsClientCapabilities {
                    data_support: Some(true),
                    ..
                }),
            ..
        }) = params.capabilities.text_document
        {
            tracing::debug!("Client supports diagnostics data")
        } else {
            tracing::warn!(
                "Client does not support diagnostics data.. code actions will not be available"
            )
        }

        let mut state = self.state.lock().unwrap();
        if let Err(e) = state.set_workspace_folders(params.workspace_folders.unwrap_or_default()) {
            tracing::warn!("Cannot set workspace folders: {}", e);
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    // TODO: should we support incremental?
                    TextDocumentSyncKind::FULL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: Some(false),
                        },
                        resolve_provider: None,
                    },
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "typos".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        tracing::debug!("did_open: {:?}", to_string(&params).unwrap_or_default());
        self.report_diagnostics(params.text_document).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        tracing::debug!("did_change: {:?}", to_string(&params).unwrap_or_default());
        self.report_diagnostics(TextDocumentItem {
            language_id: "FOOBAR".to_string(),
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
        })
        .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        tracing::debug!("did_save: {:?}", to_string(&params).unwrap_or_default());
        // noop to avoid unimplemented warning log line
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        tracing::debug!("did_close: {:?}", to_string(&params).unwrap_or_default());
        // clear diagnostics to avoid a stale diagnostics flash on open
        // if the file has typos fixed outside of vscode
        // see https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> jsonrpc::Result<Option<CodeActionResponse>> {
        tracing::debug!("code_action: {:?}", to_string(&params).unwrap_or_default());

        let actions = params
            .context
            .diagnostics
            .iter()
            .filter(|diag| diag.source == Some("typos".to_string()))
            .flat_map(|diag| match &diag.data {
                Some(data) => {
                    if let Ok(DiagnosticData { corrections }) =
                        serde_json::from_value::<DiagnosticData>(data.clone())
                    {
                        corrections
                            .iter()
                            .map(|c| {
                                CodeActionOrCommand::CodeAction(CodeAction {
                                    title: c.to_string(),
                                    kind: Some(CodeActionKind::QUICKFIX),
                                    diagnostics: Some(vec![diag.clone()]),
                                    edit: Some(WorkspaceEdit {
                                        changes: Some(HashMap::from([(
                                            params.text_document.uri.clone(),
                                            vec![TextEdit {
                                                range: diag.range,
                                                new_text: c.to_string(),
                                            }],
                                        )])),
                                        ..WorkspaceEdit::default()
                                    }),
                                    is_preferred: if corrections.len() == 1 {
                                        Some(true)
                                    } else {
                                        None
                                    },
                                    ..CodeAction::default()
                                })
                            })
                            .collect()
                    } else {
                        tracing::error!(
                            "Deserialization failed: received {:?} as diagnostic data",
                            data
                        );
                        vec![]
                    }
                }
                None => {
                    tracing::warn!("Client doesn't support diagnostic data");
                    vec![]
                }
            })
            .collect::<Vec<_>>();

        Ok(Some(actions))
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        tracing::debug!(
            "did_change_workspace_folders: {:?}",
            to_string(&params).unwrap_or_default()
        );

        let mut state = self.state.lock().unwrap();
        if let Err(e) = state.update_workspace_folders(params.event.added, params.event.removed) {
            tracing::warn!("Cannot update workspace folders {}", e);
        }
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }
}

impl<'s, 'p> Backend<'s, 'p> {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: Mutex::new(BackendState::default()),
            default_policy: policy::Policy::default(),
        }
    }

    async fn report_diagnostics(&self, params: TextDocumentItem) {
        let diagnostics = match self.check_text(&params.text, &params.uri) {
            Err(e) => {
                tracing::warn!("{}", e);
                Vec::new()
            }
            Ok(diagnostics) => diagnostics,
        };

        self.client
            .publish_diagnostics(params.uri, diagnostics, Some(params.version))
            .await;
    }

    // mimics typos_cli::file::FileChecker::check_file
    fn check_text(
        &self,
        buffer: &str,
        uri: &Url,
    ) -> anyhow::Result<Vec<Diagnostic>, anyhow::Error> {
        let path = uri
            .to_file_path()
            .map_err(|_| anyhow!("Cannot convert uri {} to file path", uri))?;

        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid unicode in path {:?}", path))?;

        let state = self.state.lock().unwrap();

        // find relevant overrides and engine for the workspace folder
        let (overrides, tokenizer, dict) = match state.router.at(path_str) {
            Err(_) => {
                tracing::debug!(
                    "Using default policy because no workspace folder found for {}",
                    uri
                );
                (
                    None,
                    self.default_policy.tokenizer,
                    self.default_policy.dict,
                )
            }
            Ok(Match { value, params: _ }) => {
                let policy = value.engine.policy(&path);
                (Some(&value.overrides), policy.tokenizer, policy.dict)
            }
        };

        // skip file if matches extend-exclude
        if let Some(overrides) = overrides {
            if overrides.matched(path_str, false).is_ignore() {
                tracing::debug!("Ignoring {} because it matches extend-exclude.", uri);
                return Ok(Vec::default());
            }
        }

        let mut accum = AccumulatePosition::new();

        Ok(typos::check_str(buffer, tokenizer, dict)
            .map(|typo| {
                tracing::debug!("typo: {:?}", typo);

                let (line_num, line_pos) = accum.pos(buffer.as_bytes(), typo.byte_offset);

                Diagnostic {
                    range: Range::new(
                        Position::new(line_num as u32, line_pos as u32),
                        Position::new(line_num as u32, (line_pos + typo.typo.len()) as u32),
                    ),
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("typos".to_string()),
                    message: match &typo.corrections {
                        typos::Status::Invalid => format!("`{}` is disallowed", typo.typo),
                        typos::Status::Corrections(corrections) => format!(
                            "`{}` should be {}",
                            typo.typo,
                            itertools::join(corrections.iter().map(|s| format!("`{}`", s)), ", ")
                        ),
                        typos::Status::Valid => panic!("unexpected typos::Status::Valid"),
                    },
                    // store corrections for retrieval during code_action
                    data: match typo.corrections {
                        typos::Status::Corrections(corrections) => {
                            Some(json!(DiagnosticData { corrections }))
                        }
                        _ => None,
                    },
                    ..Diagnostic::default()
                }
            })
            .collect())
    }
}

struct AccumulatePosition {
    line_num: usize,
    line_pos: usize,
    last_offset: usize,
}

impl AccumulatePosition {
    fn new() -> Self {
        Self {
            // LSP ranges are 0-indexed see https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#range
            line_num: 0,
            line_pos: 0,
            last_offset: 0,
        }
    }

    fn pos(&mut self, buffer: &[u8], byte_offset: usize) -> (usize, usize) {
        assert!(self.last_offset <= byte_offset);
        let slice = &buffer[self.last_offset..byte_offset];
        let newlines = slice.find_iter(b"\n").count();
        let line_num = self.line_num + newlines;

        let line_start = buffer[0..byte_offset]
            .rfind_byte(b'\n')
            // Skip the newline
            .map(|s| s + 1)
            .unwrap_or(0);

        self.line_num = line_num;
        self.line_pos = byte_offset - line_start;
        self.last_offset = byte_offset;

        (self.line_num, self.line_pos)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_initialize() {
        let (service, _) = LspService::new(Backend::new);

        let params = InitializeParams::default();
        let result = service.inner().initialize(params).await.unwrap();
        let server_info = result.server_info.unwrap();

        assert_eq!(server_info.name, "typos".to_string());
        assert_eq!(server_info.version, Some(env!("CARGO_PKG_VERSION").into()));
    }

    #[test_log::test(tokio::test)]
    async fn test_initialize_e2e() {
        let req_init =
            req(r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}"#);

        let mut output = Vec::new();

        let (service, socket) = LspService::new(Backend::new);
        Server::new(req_init.as_ref(), &mut output, socket)
            .serve(service)
            .await;

        similar_asserts::assert_eq!(
            body(&output).unwrap(),
            format!(
                r#"{{"jsonrpc":"2.0","result":{{"capabilities":{{"codeActionProvider":{{"codeActionKinds":["quickfix"],"workDoneProgress":false}},"textDocumentSync":1,"workspace":{{"workspaceFolders":{{"changeNotifications":true,"supported":true}}}}}},"serverInfo":{{"name":"typos","version":"{}"}}}},"id":1}}"#,
                env!("CARGO_PKG_VERSION")
            )
        )
    }

    #[test_log::test(tokio::test)]
    async fn test_e2e() {
        let initialize = r#"{
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
              "capabilities": {
                "textDocument": { "publishDiagnostics": { "dataSupport": true } }
              }
            },
            "id": 1
          }
        "#;

        let did_open = r#"{
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                  "textDocument": {
                    "uri": "file:///diagnostics.txt",
                    "languageId": "plaintext",
                    "version": 1,
                    "text": "this is an apropriate test\nfo typos\n"
                  }
                }
              }
            "#;

        let code_action = r#"
        {
            "jsonrpc": "2.0",
            "method": "textDocument/codeAction",
            "params": {
              "textDocument": {
                "uri": "file:///diagnostics.txt"
              },
              "range": {
                "start": {
                  "line": 1,
                  "character": 5
                },
                "end": {
                  "line": 1,
                  "character": 7
                }
              },
              "context": {
                "diagnostics": [
                  {
                    "range": {
                      "start": {
                        "line": 1,
                        "character": 5
                      },
                      "end": {
                        "line": 1,
                        "character": 7
                      }
                    },
                    "message": "`fo` should be `of`, `for`",
                    "data": {
                        "corrections": ["of", "for"]
                    },
                    "severity": 2,
                    "source": "typos"
                  }
                ],
                "only": ["quickfix"],
                "triggerKind": 1
              }
            },
            "id": 2
          }
        "#;

        let code_action_insertion = r#"
        {
            "jsonrpc": "2.0",
            "method": "textDocument/codeAction",
            "params": {
              "textDocument": {
                "uri": "file:///diagnostics.txt"
              },
              "range": {
                "start": {
                  "line": 0,
                  "character": 11
                },
                "end": {
                  "line": 0,
                  "character": 21
                }
              },
              "context": {
                "diagnostics": [
                  {
                    "range": {
                      "start": {
                        "line": 0,
                        "character": 11
                      },
                      "end": {
                        "line": 0,
                        "character": 21
                      }
                    },
                    "message": "`apropriate` should be `appropriate`",
                    "data": {
                      "corrections": ["appropriate"]
                    },
                    "severity": 2,
                    "source": "typos"
                  }
                ],
                "only": ["quickfix"],
                "triggerKind": 1
              }
            },
            "id": 3
          }
        "#;

        let (mut req_client, mut resp_client) = start_server();
        let mut buf = vec![0; 1024];

        req_client
            .write_all(req(initialize).as_bytes())
            .await
            .unwrap();
        let _ = resp_client.read(&mut buf).await.unwrap();

        tracing::debug!("{}", did_open);
        req_client
            .write_all(req(did_open).as_bytes())
            .await
            .unwrap();
        let n = resp_client.read(&mut buf).await.unwrap();

        similar_asserts::assert_eq!(
            body(&buf[..n]).unwrap(),
            r#"{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"diagnostics":[{"data":{"corrections":["appropriate"]},"message":"`apropriate` should be `appropriate`","range":{"end":{"character":21,"line":0},"start":{"character":11,"line":0}},"severity":2,"source":"typos"},{"data":{"corrections":["of","for","do","go","to"]},"message":"`fo` should be `of`, `for`, `do`, `go`, `to`","range":{"end":{"character":2,"line":1},"start":{"character":0,"line":1}},"severity":2,"source":"typos"}],"uri":"file:///diagnostics.txt","version":1}}"#,
        );

        tracing::debug!("{}", code_action);
        req_client
            .write_all(req(code_action).as_bytes())
            .await
            .unwrap();
        let n = resp_client.read(&mut buf).await.unwrap();

        similar_asserts::assert_eq!(
            body(&buf[..n]).unwrap(),
            r#"{"jsonrpc":"2.0","result":[{"diagnostics":[{"data":{"corrections":["of","for"]},"message":"`fo` should be `of`, `for`","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}},"severity":2,"source":"typos"}],"edit":{"changes":{"file:///diagnostics.txt":[{"newText":"of","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}}}]}},"kind":"quickfix","title":"of"},{"diagnostics":[{"data":{"corrections":["of","for"]},"message":"`fo` should be `of`, `for`","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}},"severity":2,"source":"typos"}],"edit":{"changes":{"file:///diagnostics.txt":[{"newText":"for","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}}}]}},"kind":"quickfix","title":"for"}],"id":2}"#,
        );

        tracing::debug!("{}", code_action_insertion);
        req_client
            .write_all(req(code_action_insertion).as_bytes())
            .await
            .unwrap();
        let n = resp_client.read(&mut buf).await.unwrap();

        similar_asserts::assert_eq!(
            body(&buf[..n]).unwrap(),
            r#"{"jsonrpc":"2.0","result":[{"diagnostics":[{"data":{"corrections":["appropriate"]},"message":"`apropriate` should be `appropriate`","range":{"end":{"character":21,"line":0},"start":{"character":11,"line":0}},"severity":2,"source":"typos"}],"edit":{"changes":{"file:///diagnostics.txt":[{"newText":"appropriate","range":{"end":{"character":21,"line":0},"start":{"character":11,"line":0}}}]}},"isPreferred":true,"kind":"quickfix","title":"appropriate"}],"id":3}"#,
        );
    }

    #[test_log::test(tokio::test)]
    async fn test_config_file_e2e() {
        let workspace_folder_uri =
            Url::from_file_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")).unwrap();

        let initialize = format!(
            r#"{{
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {{
              "capabilities": {{
                "textDocument": {{ "publishDiagnostics": {{ "dataSupport": true }} }}
              }},
              "workspaceFolders": [
                {{
                  "uri": "{}",
                  "name": "tests"
                }}
              ]
            }},
            "id": 1
          }}
        "#,
            workspace_folder_uri
        );

        let did_open_diag_txt = format!(
            r#"{{
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {{
                  "textDocument": {{
                    "uri": "{}/diagnostics.txt",
                    "languageId": "plaintext",
                    "version": 1,
                    "text": "this is an apropriate test\nfo typos\n"
                  }}
                }}
              }}
            "#,
            workspace_folder_uri
        );

        let did_open_changelog = format!(
            r#"{{
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {{
                  "textDocument": {{
                    "uri": "{}/CHANGELOG.md",
                    "languageId": "plaintext",
                    "version": 1,
                    "text": "this is an apropriate test\nfo typos\n"
                  }}
                }}
              }}
            "#,
            workspace_folder_uri
        );

        let (mut req_client, mut resp_client) = start_server();
        let mut buf = vec![0; 1024];

        req_client
            .write_all(req(initialize).as_bytes())
            .await
            .unwrap();
        let _ = resp_client.read(&mut buf).await.unwrap();

        // check "fo" is corrected to "of" because of default.extend-words
        tracing::debug!("{}", did_open_diag_txt);
        req_client
            .write_all(req(did_open_diag_txt).as_bytes())
            .await
            .unwrap();
        let n = resp_client.read(&mut buf).await.unwrap();

        similar_asserts::assert_eq!(
            body(&buf[..n]).unwrap(),
            format!(
                r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"diagnostics":[{{"data":{{"corrections":["appropriate"]}},"message":"`apropriate` should be `appropriate`","range":{{"end":{{"character":21,"line":0}},"start":{{"character":11,"line":0}}}},"severity":2,"source":"typos"}},{{"data":{{"corrections":["of"]}},"message":"`fo` should be `of`","range":{{"end":{{"character":2,"line":1}},"start":{{"character":0,"line":1}}}},"severity":2,"source":"typos"}}],"uri":"{}/diagnostics.txt","version":1}}}}"#,
                workspace_folder_uri
            ),
        );

        // check changelog is excluded because of files.extend-exclude
        tracing::debug!("{}", did_open_changelog);
        req_client
            .write_all(req(did_open_changelog).as_bytes())
            .await
            .unwrap();
        let n = resp_client.read(&mut buf).await.unwrap();

        similar_asserts::assert_eq!(
            body(&buf[..n]).unwrap(),
            format!(
                r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"diagnostics":[],"uri":"{}/CHANGELOG.md","version":1}}}}"#,
                workspace_folder_uri
            ),
        );
    }

    fn start_server() -> (tokio::io::DuplexStream, tokio::io::DuplexStream) {
        let (req_client, req_server) = tokio::io::duplex(1024);
        let (resp_server, resp_client) = tokio::io::duplex(1024);

        let (service, socket) = LspService::new(Backend::new);

        // start server as concurrent task
        tokio::spawn(Server::new(req_server, resp_server, socket).serve(service));

        (req_client, resp_client)
    }

    fn req<T: AsRef<str>>(msg: T) -> String {
        format!(
            "Content-Length: {}\r\n\r\n{}",
            msg.as_ref().len(),
            msg.as_ref()
        )
    }

    fn body(src: &[u8]) -> Result<&str, anyhow::Error> {
        // parse headers to get headers length
        let mut dst = [httparse::EMPTY_HEADER; 2];

        let (headers_len, _) = match httparse::parse_headers(src, &mut dst)? {
            httparse::Status::Complete(output) => output,
            httparse::Status::Partial => return Err(anyhow::anyhow!("partial headers")),
        };

        // skip headers
        let skipped = &src[headers_len..];

        // return the rest (ie: the body) as &str
        std::str::from_utf8(skipped).map_err(anyhow::Error::from)
    }
}
