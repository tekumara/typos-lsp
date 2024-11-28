use anyhow::anyhow;
use matchit::Router;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::{DiagnosticSeverity, Url, WorkspaceFolder};

use crate::typos::Instance;

#[derive(Default)]
pub(crate) struct BackendState<'s> {
    pub severity: Option<DiagnosticSeverity>,
    pub config: Option<PathBuf>,
    pub workspace_folders: Vec<WorkspaceFolder>,
    pub router: Router<crate::typos::Instance<'s>>,
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

    /// Insert repositories into the router,
    ///   but do not traverse further.
    ///
    /// For now, this only looks for git-repositories.
    ///
    /// TODO: Check for other repositories, e.g. mercurial.
    fn update_repositories(
        &self,
        router: &mut Router<crate::typos::Instance<'s>>,
        root: &Path,
    ) -> anyhow::Result<()> {
        if root.join(".git").try_exists()? {
            // Found a directory or file `.git`:
            // Insert it into the router and stop.
            router.insert_instance(
                &format!("{}/{{*p}}", str_path_sanitised(&root.display().to_string())),
                root,
                self.config.as_deref(),
            )
        } else {
            // Otherwise: Traverse directories to find repositories.
            root.read_dir()?.try_for_each(|dir_entry| {
                let dir_entry = dir_entry?;
                match dir_entry.file_type()?.is_dir() {
                    true => self.update_repositories(router, &dir_entry.path()),
                    false => Ok(()),
                }
            })
        }
    }

    pub(crate) fn update_router(&mut self) -> anyhow::Result<()> {
        let mut router = Router::new();
        for folder in self.workspace_folders.iter() {
            let path = folder
                .uri
                .to_file_path()
                .map_err(|_| anyhow!("Cannot convert uri {} to file path", folder.uri))?;

            // Look for repositories and insert them as config search paths.
            // Log but otherwise ignore any error.
            if let Err(error) = self.update_repositories(&mut router, &path) {
                tracing::error!(
                    "An error occurred while updating repositories {}: {error}",
                    path.display()
                );
            }

            // Finally insert the workspace directory as config search path:
            router.insert_instance(
                &format!("{}/{{*p}}", url_path_sanitised(&folder.uri)),
                &path,
                self.config.as_deref(),
            )?;
        }
        self.router = router;

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

pub fn str_path_sanitised(path: &str) -> String {
    // windows paths (eg: /C:/Users/..) may not be percent-encoded by some clients
    // and therefore contain colons, see
    // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#uri
    //
    // and because matchit treats colons as a wildcard we need to strip them
    path.replace(':', "%3A")
}

pub fn url_path_sanitised(url: &Url) -> String {
    str_path_sanitised(url.path())
}
