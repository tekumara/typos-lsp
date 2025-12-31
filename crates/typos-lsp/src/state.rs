use anyhow::anyhow;
use matchit::Router;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tower_lsp_server::ls_types::{
    DiagnosticSeverity, TextDocumentContentChangeEvent, Uri, WorkspaceFolder,
};

use crate::typos::Instance;

#[derive(Default)]
pub(crate) struct BackendState<'s> {
    pub severity: Option<DiagnosticSeverity>,
    /// The path to the configuration file given to the LSP server. Settings in this configuration
    /// file override the typos.toml settings.
    pub config: Option<PathBuf>,
    pub workspace_folders: Vec<WorkspaceFolder>,

    /// Maps routes (file system paths) to TyposCli instances, so that we can quickly find the
    /// correct instance for a given file path
    pub router: Router<crate::typos::Instance<'s>>,
    pub documents: HashMap<Uri, Document>,
}

pub(crate) struct Document {
    pub version: i32,
    pub text: String,
}

impl Document {
    pub fn new(version: i32, text: String) -> Self {
        Self { version, text }
    }

    pub fn update(&mut self, version: i32, changes: Vec<TextDocumentContentChangeEvent>) {
        for change in changes {
            if change.range.is_some() {
                tracing::warn!("Incremental document updates are not supported");
                return;
            }
            self.text = change.text;
        }
        self.version = version;
    }
}

impl<'s> BackendState<'s> {
    pub(crate) fn set_workspace_folders(
        &mut self,
        workspace_folders: Vec<WorkspaceFolder>,
    ) -> anyhow::Result<(), anyhow::Error> {
        self.workspace_folders = workspace_folders;
        self.update_router()?;
        Ok(())
    }

    pub(crate) fn update_workspace_folders(
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

    pub(crate) fn update_router(&mut self) -> anyhow::Result<(), anyhow::Error> {
        self.router = Router::new();
        for folder in self.workspace_folders.iter() {
            let path = folder
                .uri
                .to_file_path()
                .ok_or_else(|| anyhow!("Cannot convert uri {:?} to file path", folder.uri))?;
            let route = format!("{}{}", uri_path_sanitised(&folder.uri), "/{*p}");
            self.router
                .insert_instance(&route, &path, self.config.as_deref())?;
        }

        // add low priority catch all route used for files outside the workspace, or
        // when there is no workspace folder
        #[cfg(windows)]
        for drive in crate::windows::get_drives() {
            let route = format!("/{}%3A/{{*p}}", &drive);
            self.router.insert_instance(
                &route,
                &PathBuf::from(format!("{}:\\", &drive)),
                self.config.as_deref(),
            )?;
        }

        #[cfg(not(windows))]
        {
            let route = "/{*p}";
            self.router
                .insert_instance(route, &PathBuf::from("/"), self.config.as_deref())?;
        }

        Ok(())
    }
}

trait RouterExt {
    fn insert_instance(
        &mut self,
        route: &str,
        path: &Path,
        config: Option<&Path>,
    ) -> anyhow::Result<(), anyhow::Error>;
}

impl RouterExt for Router<Instance<'_>> {
    // convenience method to insert a new TyposCli into the router
    // implemented as an extension trait to avoid interprocedural conflicts
    fn insert_instance(
        &mut self,
        route: &str,
        path: &Path,
        config: Option<&Path>,
    ) -> anyhow::Result<(), anyhow::Error> {
        tracing::debug!("Adding route {} for path {}", route, path.display());
        let instance = Instance::new(path, config)?;
        self.insert(route, instance)?;
        Ok(())
    }
}

pub fn uri_path_sanitised(uri: &Uri) -> String {
    // windows paths (eg: /C:/Users/..) may not be percent-encoded by some clients
    // and therefore contain colons, see
    // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#uri
    //
    // and because matchit treats colons as a wildcard we need to strip them
    uri.path().to_string().replace(':', "%3A")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_document_full() {
        let mut doc = Document::new(0, "".to_string());
        let version = 1;
        let text = "hello world";

        let changes = vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: text.to_string(),
        }];

        doc.update(version, changes);

        assert_eq!(doc.text, "hello world");
        assert_eq!(doc.version, 1);
    }
}
