use matchit::Match;

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{json, to_string};
use tower_lsp::lsp_types::*;
use tower_lsp::*;
use tower_lsp::{Client, LanguageServer};
use typos_cli::config::DictConfig;
use typos_cli::policy;

use crate::state::{url_path_sanitised, BackendState};
pub struct Backend<'s, 'p> {
    client: Client,
    state: Mutex<crate::state::BackendState<'s>>,
    default_policy: policy::Policy<'p, 'p, 'p>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DiagnosticData<'c> {
    corrections: Vec<Cow<'c, str>>,
    typo: Cow<'c, str>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct IgnoreInProjectCommandArguments {
    typo: String,
    /// The file that contains the typo to ignore
    typo_file_path: String,
    /// The configuration file that should be modified to ignore the typo
    config_file_path: String,
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
                        "information" | "info" => {
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
                // only support UTF-16 positions for now, which is the default when unspecified
                position_encoding: Some(PositionEncodingKind::UTF16),
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
                execute_command_provider: Some(ExecuteCommandOptions {
                    // TODO this magic string should be a constant
                    commands: vec!["ignore-in-project".to_string()],
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
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

    /// Called by the editor to request displaying a list of code actions and commands for a given
    /// position in the current file.
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
                    if let Ok(DiagnosticData { corrections, typo }) =
                        serde_json::from_value::<DiagnosticData>(data.clone())
                    {
                        let mut suggestions: Vec<_> = corrections
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
                            .collect();

                        if let Ok(Match { value, .. }) = self
                            .state
                            .lock()
                            .unwrap()
                            .router
                            .at(params.text_document.uri.to_file_path().unwrap().to_str().unwrap())
                        {
                            let typo_file: &Url = &params.text_document.uri;
                            let config_files =
                                value.config_files_in_project(Path::new(typo_file.as_str()));

                            suggestions.push(CodeActionOrCommand::Command(Command {
                                title: format!("Ignore `{}` in the project", typo),
                                command: "ignore-in-project".to_string(),
                                arguments: Some(
                                    [serde_json::to_value(IgnoreInProjectCommandArguments {
                                        typo: typo.to_string(),
                                        typo_file_path: typo_file.to_string(),
                                        config_file_path: config_files
                                            .project_root
                                            .path
                                            .to_string_lossy()
                                            .to_string(),
                                    })
                                    .unwrap()]
                                    .into(),
                                ),
                            }));
                        } else {
                            tracing::warn!(
                                "code_action: Cannot create a code action for ignoring a typo in the project. Reason: No route found for file '{}'",
                                params.text_document.uri
                            );
                        }

                        suggestions
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

    /// Called by the editor to execute a server side command, such as ignoring a typo.
    async fn execute_command(
        &self,
        raw_params: ExecuteCommandParams,
    ) -> jsonrpc::Result<Option<serde_json::Value>> {
        tracing::debug!(
            "execute_command: {:?}",
            to_string(&raw_params).unwrap_or_default()
        );

        // TODO reduce the nesting
        if raw_params.command == "ignore-in-project" {
            let argument = raw_params
                .arguments
                .into_iter()
                .next()
                .expect("no arguments for ignore-in-project command");

            if let Ok(IgnoreInProjectCommandArguments {
                typo,
                config_file_path,
                ..
            }) = serde_json::from_value::<IgnoreInProjectCommandArguments>(argument)
            {
                let mut config = typos_cli::config::Config::from_file(Path::new(&config_file_path))
                    .ok()
                    .flatten()
                    .unwrap_or_default();

                config.default.dict.update(&DictConfig {
                    extend_words: HashMap::from([(typo.clone().into(), typo.into())]),
                    ..Default::default()
                });

                std::fs::write(
                    &config_file_path,
                    toml::to_string_pretty(&config).expect("cannot serialize config"),
                )
                .unwrap_or_else(|_| panic!("Cannot write to {}", config_file_path));
            };
        }

        Ok(None)
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

    fn check_text(&self, buffer: &str, uri: &Url) -> Vec<Diagnostic> {
        let state = self.state.lock().unwrap();

        let Some((tokenizer, dict, ignore)) = self.workspace_policy(uri, &state) else {
            // skip file because it matches extend-exclude
            return Vec::default();
        };

        crate::typos::check_str(buffer, tokenizer, dict, ignore)
            .map(|(typo, line_num, line_pos)| {
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
                        typos::Status::Corrections(corrections) => Some(json!(DiagnosticData {
                            corrections,
                            typo: typo.typo
                        })),
                        _ => None,
                    },
                    ..Diagnostic::default()
                }
            })
            .collect()
    }

    fn workspace_policy<'a>(
        &'a self,
        uri: &Url,
        state: &'a std::sync::MutexGuard<'a, BackendState<'s>>,
    ) -> Option<(
        &typos::tokens::Tokenizer,
        &dyn typos::Dictionary,
        &[regex::Regex],
    )> {
        let (tokenizer, dict, ignore) = match uri.to_file_path() {
            Err(_) => {
                // eg: uris like untitled:* or term://*
                tracing::debug!(
                    "workspace_policy: Using default policy because cannot convert uri {} to file path",
                    uri
                );
                (
                    self.default_policy.tokenizer,
                    self.default_policy.dict,
                    self.default_policy.ignore,
                )
            }
            Ok(path) => {
                let uri_path = url_path_sanitised(uri);

                // find relevant tokenizer, and dict for the workspace folder
                let (tokenizer, dict, ignore) = match state.router.at(&uri_path) {
                    Err(_) => {
                        // ie: file:///
                        tracing::debug!(
                            "workspace_policy: Using default policy because no route found for {}",
                            uri_path
                        );
                        (
                            self.default_policy.tokenizer,
                            self.default_policy.dict,
                            self.default_policy.ignore,
                        )
                    }
                    Ok(Match { value, params: _ }) => {
                        tracing::debug!("workspace_policy: path {}", &path.display());
                        // skip file if matches extend-exclude
                        if value.ignores.matched(&path, false).is_ignore() {
                            tracing::debug!(
                                "workspace_policy: Ignoring {} because it matches extend-exclude.",
                                uri
                            );
                            return None;
                        }
                        let policy = value.engine.policy(&path);
                        (policy.tokenizer, policy.dict, policy.ignore)
                    }
                };

                (tokenizer, dict, ignore)
            }
        };
        Some((tokenizer, dict, ignore))
    }
}
