use config_file_location::ConfigFileLocation;

use super::config_file_location;

/// Represents the paths to typos_cli config files that could be used when adding a new ignore
/// rule. The config files may or may not exist.
pub struct ConfigFileSuggestions {
    /// The path to a (possible) configuration file in the directory where the LSP server was
    /// started. This is always included as the default suggestion.
    pub project_root: ConfigFileLocation,

    /// Other configuration files that currently exist in the project. The order is from the closest
    /// to the currently open file to the project root. Only existing files are included.
    pub config_files: Vec<ConfigFileLocation>,
}
