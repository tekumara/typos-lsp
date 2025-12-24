use matchit::{Match, Router};

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{json, to_string};
use tower_lsp_server::ls_types::*;
use tower_lsp_server::*;
use tower_lsp_server::{Client, LanguageServer};
use typos_cli::policy;

use crate::state::{uri_path_sanitised, BackendState, Document};

const IGNORE_IN_PROJECT: &str = "ignore-in-project";

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
    /// The configuration file that should be modified to ignore the typo
    config_file_path: PathBuf,
}

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

        if state.severity.is_none() {
            state.severity = Some(DiagnosticSeverity::INFORMATION);
        }

        if let Err(e) = state.set_workspace_folders(params.workspace_folders.unwrap_or_default()) {
            tracing::warn!("Falling back to default config: {}", e);
        }

        if state.workspace_folders.is_empty() {
            tracing::warn!("Initialised without workspaces folders");
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // only support UTF-16 positions for now, which is the default when unspecified
                position_encoding: Some(PositionEncodingKind::UTF16),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
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
                    commands: vec![IGNORE_IN_PROJECT.to_string()],
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
        let TextDocumentItem {
            uri, text, version, ..
        } = params.text_document;
        {
            let mut state = self.state.lock().unwrap();
            state
                .documents
                .insert(uri.clone(), Document::new(version, text.clone()));
        }
        self.report_diagnostics(uri, version, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        tracing::debug!("did_change: {:?}", to_string(&params).unwrap_or_default());

        let DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version },
            content_changes,
        } = params;

        let diagnostics = {
            let mut state = self.state.lock().unwrap();
            let BackendState {
                documents,
                router,
                severity,
                ..
            } = &mut *state;

            let Some(doc) = documents.get_mut(&uri) else {
                tracing::warn!("Received update for unknown document: {:?}", uri);
                return;
            };

            doc.update(version, content_changes);
            self.check_text_inner(&doc.text, &uri, router, *severity)
        };

        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        tracing::debug!("did_save: {:?}", to_string(&params).unwrap_or_default());
        // noop to avoid unimplemented warning log line
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        tracing::debug!("did_close: {:?}", to_string(&params).unwrap_or_default());
        self.state
            .lock()
            .unwrap()
            .documents
            .remove(&params.text_document.uri);
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

                        let uri_path = uri_path_sanitised(&params.text_document.uri);
                        if let Ok(Match { value: instance, .. }) = self
                            .state
                            .lock()
                            .unwrap()
                            .router
                            .at(&uri_path)
                        {
                            match serde_json::to_value(IgnoreInProjectCommandArguments {
                                typo: typo.to_string(),
                                config_file_path: instance.config_file.clone(),
                            }) {
                                Ok(arg) => {
                                    suggestions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: format!("Ignore `{}` in the project", typo),
                                        kind: Some(CodeActionKind::QUICKFIX),
                                        diagnostics: Some(vec![diag.clone()]),
                                        command: Some(Command {
                                            title: format!("Ignore `{}` in the project", typo),
                                            command: IGNORE_IN_PROJECT.to_string(),
                                            arguments: Some(vec![arg]),
                                        }),
                                        ..Default::default()
                                    }));
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to serialize ignore-in-project arguments. Error: {}",
                                        e
                                    );
                                }
                            }

                            if let Some(explicit_config) = &instance.custom_config {
                                match serde_json::to_value(IgnoreInProjectCommandArguments {
                                    typo: typo.to_string(),
                                    config_file_path: explicit_config.clone(),
                                }) {
                                    Ok(arg) => {
                                        suggestions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                            title: format!("Ignore `{}` in the configuration file", typo),
                                            kind: Some(CodeActionKind::QUICKFIX),
                                            diagnostics: Some(vec![diag.clone()]),
                                            command: Some(Command {
                                                title: format!("Ignore `{}` in the configuration file", typo),
                                                command: IGNORE_IN_PROJECT.to_string(),
                                                arguments: Some(vec![arg]),
                                            }),
                                            ..Default::default()
                                        }));
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Failed to serialize ignore-in-config arguments. Error: {}",
                                            e
                                        );
                                    }
                                }
                            }
                        } else {
                            tracing::warn!(
                                "code_action: Cannot create a code action for ignoring a typo in the project. Reason: No route found for file '{:?}'",
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
        params: ExecuteCommandParams,
    ) -> jsonrpc::Result<Option<serde_json::Value>> {
        tracing::debug!(
            "execute_command: {:?}",
            to_string(&params).unwrap_or_default()
        );

        match params.command.as_str() {
            IGNORE_IN_PROJECT => {
                match params
                    .arguments
                    .into_iter()
                    .next()
                    .map(serde_json::from_value)
                {
                    Some(Ok(args)) => self.handle_ignore_in_project(args).await,
                    Some(Err(e)) => {
                        tracing::warn!("failed to parse ignore-in-project arguments: {}", e)
                    }
                    None => tracing::warn!("no arguments for ignore-in-project command"),
                }
                Ok(None)
            }
            _ => {
                tracing::warn!("Unknown command: {}", params.command);
                Ok(None)
            }
        }
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

impl<'s> Backend<'s, '_> {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: Mutex::new(BackendState::default()),
            default_policy: policy::Policy::default(),
        }
    }

    async fn handle_ignore_in_project(&self, args: IgnoreInProjectCommandArguments) {
        if let Err(e) = crate::config::add_ignore(Path::new(&args.config_file_path), &args.typo) {
            tracing::warn!("Failed to add ignore: {}", e);
            return;
        }

        let docs = {
            let mut state = self.state.lock().unwrap();
            // reload the instance so new ignore takes effect
            // TODO: reloading does introduce a noticeable delay, perhaps we should apply ignores in place?
            if let Err(e) = state.update_router() {
                tracing::warn!("Failed to update router: {}", e);
                return;
            }
            state
                .documents
                .iter()
                .map(|(uri, doc)| (uri.clone(), doc.version, doc.text.clone()))
                .collect::<Vec<_>>()
        };

        // report diagnostics for all documents, in case the new ignore affects them
        for (uri, version, text) in docs {
            self.report_diagnostics(uri, version, &text).await;
        }
    }

    async fn report_diagnostics(&self, uri: Uri, version: i32, text: &str) {
        let diagnostics = self.check_text(text, &uri);
        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }

    fn check_text(&self, buffer: &str, uri: &Uri) -> Vec<Diagnostic> {
        let state = self.state.lock().unwrap();
        self.check_text_inner(buffer, uri, &state.router, state.severity)
    }

    fn check_text_inner(
        &self,
        buffer: &str,
        uri: &Uri,
        router: &Router<crate::typos::Instance<'s>>,
        severity: Option<DiagnosticSeverity>,
    ) -> Vec<Diagnostic> {
        let Some((tokenizer, dict, ignore)) = self.workspace_policy(uri, router) else {
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
                    severity,
                    source: Some("typos".to_string()),
                    message: match &typo.corrections {
                        typos::Status::Invalid => format!("`{}` is disallowed", typo.typo),
                        typos::Status::Corrections(corrections) => format!(
                            "`{}` should be {}",
                            typo.typo,
                            itertools::join(corrections.iter().map(|s| format!("`{s}`")), ", ")
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
        uri: &Uri,
        router: &'a Router<crate::typos::Instance<'s>>,
    ) -> Option<(
        &'a typos::tokens::Tokenizer,
        &'a dyn typos::Dictionary,
        &'a [regex::Regex],
    )> {
        let (tokenizer, dict, ignore) = match uri.to_file_path() {
            None => {
                // eg: uris like untitled:* or term://*
                tracing::debug!(
                    "workspace_policy: Using default policy because cannot convert uri {:?} to file path",
                    uri
                );
                (
                    self.default_policy.tokenizer,
                    self.default_policy.dict,
                    self.default_policy.ignore,
                )
            }
            Some(path) => {
                let uri_path = uri_path_sanitised(uri);

                // find relevant tokenizer, and dict for the workspace folder
                let (tokenizer, dict, ignore) = match router.at(&uri_path) {
                    Err(_) => {
                        // ie: file:///
                        tracing::debug!(
                            "workspace_policy: Using default policy because no route found for {:?}",
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
                                "workspace_policy: Ignoring {:?} because it matches extend-exclude.",
                                uri
                            );
                            return None;
                        }
                        let policy = value.engine.policy(&path);
                        // skip file types that are not checked
                        // see https://github.com/crate-ci/typos/blob/fb1f64595962a79113c92d4879e6b3b2e8f524b4/crates/typos-cli/src/file_type_specifics.rs#L7
                        if !policy.check_files {
                            tracing::debug!(
                                "workspace_policy: Ignoring {:?} because its file type is not checked.",
                                uri
                            );
                            return None;
                        }
                        (policy.tokenizer, policy.dict, policy.ignore)
                    }
                };

                (tokenizer, dict, ignore)
            }
        };
        Some((tokenizer, dict, ignore))
    }
}
