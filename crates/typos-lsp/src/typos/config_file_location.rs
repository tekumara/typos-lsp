use std::path::Path;

use std::path::PathBuf;

/// Represents a path to a typos_cli config file and, if it contains a configuration file, the file
/// contents
pub struct ConfigFileLocation {
    pub path: PathBuf,
    pub config: Option<typos_cli::config::Config>,
}

impl ConfigFileLocation {
    pub fn from_dir_or_default(path: &Path) -> ConfigFileLocation {
        let directory = if path.is_dir() {
            path
        } else {
            path.parent().unwrap()
        };
        ConfigFileLocation::from_dir(directory).unwrap_or_else(|_| ConfigFileLocation {
            path: path.to_path_buf(),
            config: None,
        })
    }

    // copied from typos_cli::config::Config::from_dir with the difference that it shows which
    // config file was found of the supported ones. This information is useful when we want to
    // modify the config file later on.
    pub fn from_dir(dir: &Path) -> anyhow::Result<ConfigFileLocation> {
        assert!(
            dir.is_dir(),
            "Expected a directory that might contain a configuration file"
        );

        for file in typos_cli::config::SUPPORTED_FILE_NAMES {
            let path = dir.join(file);
            if let Ok(Some(config)) = typos_cli::config::Config::from_file(path.as_path()) {
                return Ok(ConfigFileLocation {
                    path,
                    config: Some(config),
                });
            }
        }

        Err(anyhow::anyhow!(
            "No typos_cli config file found starting from {:?}",
            dir
        ))
    }
}
