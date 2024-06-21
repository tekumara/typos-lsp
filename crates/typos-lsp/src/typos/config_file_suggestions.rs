use std::path::PathBuf;

/// Represents the paths to typos_cli config files that could be used when adding a new ignore
/// rule. The config files may or may not exist.
pub struct ConfigFileSuggestions {
    /// The path to a (possible) configuration file in the directory where the LSP server was
    /// started. This is always included as the default suggestion.
    pub project_root: PathBuf,

    /// The explicit configuration file that was given to the LSP server at startup.
    pub explicit: Option<PathBuf>,
}
