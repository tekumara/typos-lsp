use anyhow::anyhow;
use matchit::{Match, Router};

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    severity: Option<DiagnosticSeverity>,
    config: Option<PathBuf>,
    workspace_folders: Vec<WorkspaceFolder>,
    router: Router<TyposCli<'s>>,
}

struct TyposCli<'s> {
    overrides: Override,
    engine: policy::ConfigEngine<'s>,
}

// initialise an engine and overrides using the config file from path or its parent
fn try_new_cli<'s>(
    path: &Path,
    config: Option<&Path>,
) -> anyhow::Result<TyposCli<'s>, anyhow::Error> {
    // leak to get a 'static which is needed to satisfy the 's lifetime
    // but does mean memory will grow unbounded
    let storage = Box::leak(Box::new(policy::ConfigStorage::new()));
    let mut engine = typos_cli::policy::ConfigEngine::new(storage);

    // TODO: currently mimicking typos here but do we need to create and the update
    // a default config?
    let mut overrides = typos_cli::config::Config::default();
    if let Some(config_path) = config {
        let custom = typos_cli::config::Config::from_file(config_path)?;
        if let Some(custom) = custom {
            overrides.update(&custom);
            engine.set_overrides(overrides);
        }
    }

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
                path_to_route(
                    path.to_str()
                        .ok_or_else(|| anyhow!("Invalid unicode in path {:?}", path))?
                ),
                "/*p"
            );
            tracing::debug!("Adding route {}", &path_wildcard);
            let cli = try_new_cli(&path, self.config.as_deref())?;
            self.router.insert(path_wildcard, cli)?;
        }
        Ok(())
    }
}

fn path_to_route(path: &str) -> Cow<str> {
    if path.starts_with('\\') {
        // unix path
        path.into()
    } else {
        // normalise windows path to make matchit happy:
        // - strip out : so matchit doesn't use it as a wildcard
        // - path segments must be forward slashes not backslashes to match
        // - leading / required on wildcard routes
        let mut path = path.replace(':', "").replace('\\', "/");
        path.insert(0, '/');
        path.into()
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
                "Client does not support diagnostics data. Code actions will not be available"
            )
        }

        let mut state = self.state.lock().unwrap();

        if let Some(ops) = params.initialization_options {
            if let Some(values) = ops.as_object() {
                if let Some(value) = values.get("diagnosticSeverity").cloned() {
                    match value.as_str().unwrap_or("").to_lowercase().as_str() {
                        "error" => {
                            state.severity = Some(DiagnosticSeverity::ERROR);
                        }
                        "warning" => {
                            state.severity = Some(DiagnosticSeverity::WARNING);
                        }
                        "information" => {
                            state.severity = Some(DiagnosticSeverity::INFORMATION);
                        }
                        "hint" => {
                            state.severity = Some(DiagnosticSeverity::HINT);
                        }
                        _ => {
                            tracing::warn!("Unknown diagnostic severity: {}", value);
                        }
                    }
                }
                if let Some(value) = values.get("config").cloned() {
                    if let Some(value) = value.as_str() {
                        let expanded_path = PathBuf::from(shellexpand::tilde(value).to_string());
                        state.config = Some(expanded_path);
                    }
                }
            }
        }

        if let Err(e) = state.set_workspace_folders(params.workspace_folders.unwrap_or_default()) {
            tracing::warn!("Falling back to default config: {}", e);
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
        let diagnostics = self.check_text(&params.text, &params.uri);
        self.client
            .publish_diagnostics(params.uri, diagnostics, Some(params.version))
            .await;
    }

    // mimics typos_cli::file::FileChecker::check_file
    fn check_text(&self, buffer: &str, uri: &Url) -> Vec<Diagnostic> {
        let path = uri.to_file_path().unwrap_or_else(|_| {
            tracing::warn!("check_text: Cannot convert uri {} to file path", uri);
            PathBuf::default()
        });

        let path_str = path_to_route(path.to_str().unwrap_or_else(|| {
            tracing::warn!("check_text: Invalid unicode in path {:?}", path);
            "/"
        }));

        let state = self.state.lock().unwrap();

        // find relevant overrides and engine for the workspace folder
        let (overrides, tokenizer, dict) = match state.router.at(&path_str) {
            Err(_) => {
                tracing::debug!(
                    "Using default policy because no route found for {}",
                    path_str
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
            if overrides.matched(path, false).is_ignore() {
                tracing::debug!("Ignoring {} because it matches extend-exclude.", uri);
                return Vec::default();
            }
        }

        let mut accum = AccumulatePosition::new();

        typos::check_str(buffer, tokenizer, dict)
            .map(|typo| {
                tracing::debug!("typo: {:?}", typo);

                let (line_num, line_pos) = accum.pos(buffer.as_bytes(), typo.byte_offset);

                Diagnostic {
                    range: Range::new(
                        Position::new(line_num as u32, line_pos as u32),
                        Position::new(line_num as u32, (line_pos + typo.typo.len()) as u32),
                    ),
                    severity: state.severity,
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
