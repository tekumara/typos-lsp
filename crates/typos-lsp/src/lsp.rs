use std::borrow::Cow;
use std::collections::HashMap;

use bstr::ByteSlice;
use serde_json::json;
use tower_lsp::lsp_types::*;
use tower_lsp::*;
use tower_lsp::{Client, LanguageServer};

use typos_cli::policy;

pub struct Backend<'a> {
    client: Client,
    policy: policy::Policy<'a, 'a, 'a>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DiagnosticData<'c> {
    corrections: Vec<Cow<'c, str>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend<'static> {
    async fn initialize(&self, params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        tracing::debug!("initialize: {:?}", params);

        if let Some(TextDocumentClientCapabilities {
            publish_diagnostics:
                Some(PublishDiagnosticsClientCapabilities {
                    data_support: Some(true),
                    ..
                }),
            ..
        }) = params.capabilities.text_document
        {
            tracing::debug!("client supports diagnostics data")
        } else {
            tracing::warn!(
                "client does not support diagnostics data.. code actions will not be available"
            )
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
                ..ServerCapabilities::default()
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
        tracing::debug!("did_open: {:?}", params);
        self.report_diagnostics(params.text_document).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        tracing::debug!("did_change: {:?}", params);
        self.report_diagnostics(TextDocumentItem {
            language_id: "FOOBAR".to_string(),
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
        })
        .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        tracing::debug!("did_save: {:?}", params);
        // noop to avoid unimplemented warning log line
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        tracing::debug!("did_close: {:?}", params);
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
        tracing::debug!("code_action: {:?}", params);

        let actions = params
            .context
            .diagnostics
            .iter()
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
                    tracing::warn!("client doesn't support diagnostic data");
                    vec![]
                }
            })
            .collect::<Vec<_>>();

        Ok(Some(actions))
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }
}

impl Backend<'static> {
    pub fn new(client: Client) -> Self {
        let policy = policy::Policy::new();
        Self { client, policy }
    }

    async fn report_diagnostics(&self, params: TextDocumentItem) {
        let diagnostics = self.check_text(&params.text);

        self.client
            .publish_diagnostics(params.uri, diagnostics, Some(params.version))
            .await;
    }

    // mimics typos_cli::file::FileChecker::check_file
    fn check_text(&self, buffer: &str) -> Vec<Diagnostic> {
        let mut accum = AccumulatePosition::new();

        // TODO: support ignores & typos.toml

        typos::check_str(buffer, self.policy.tokenizer, self.policy.dict)
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
            .collect()
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
                r#"{{"jsonrpc":"2.0","result":{{"capabilities":{{"codeActionProvider":{{"codeActionKinds":["quickfix"],"workDoneProgress":false}},"textDocumentSync":1}},"serverInfo":{{"name":"typos","version":"{}"}}}},"id":1}}"#,
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
            r#"{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"diagnostics":[{"data":{"corrections":["appropriate"]},"message":"`apropriate` should be `appropriate`","range":{"end":{"character":21,"line":0},"start":{"character":11,"line":0}},"severity":2,"source":"typos"},{"data":{"corrections":["of","for"]},"message":"`fo` should be `of`, `for`","range":{"end":{"character":2,"line":1},"start":{"character":0,"line":1}},"severity":2,"source":"typos"}],"uri":"file:///diagnostics.txt","version":1}}"#,
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

    fn start_server() -> (tokio::io::DuplexStream, tokio::io::DuplexStream) {
        let (req_client, req_server) = tokio::io::duplex(1024);
        let (resp_server, resp_client) = tokio::io::duplex(1024);

        let (service, socket) = LspService::new(Backend::new);

        // start server as concurrent task
        tokio::spawn(Server::new(req_server, resp_server, socket).serve(service));

        (req_client, resp_client)
    }

    fn req(msg: &str) -> String {
        format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg)
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
