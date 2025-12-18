use std::path::Path;

use std::path::PathBuf;

/// Represents a path to a typos_cli config file and, if it contains a configuration file, the file
/// contents.
///
/// When reading a config from a directory, many configuration files are supported, and only one is
/// chosen in a given order. Shows the name of the config file that is used ("typos.toml",
/// "_typos.toml", ".typos.toml", "pyproject.toml"). This information is useful when we want to
/// modify the config file later on.
#[derive(Debug, Clone)]
pub struct ConfigFileLocation {
    pub path: PathBuf,
    pub config: Option<typos_cli::config::Config>,
}

impl PartialEq for ConfigFileLocation {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && format!("{:?}", self.config) == format!("{:?}", other.config)
    }
}

impl ConfigFileLocation {
    pub fn from_file_path_or_default(path: &Path) -> ConfigFileLocation {
        let config = typos_cli::config::Config::from_file(path).ok().flatten();

        ConfigFileLocation {
            config,
            path: path.to_path_buf(),
        }
    }

    pub fn from_dir_or_default(path: &Path) -> ConfigFileLocation {
        let directory = if path.is_dir() {
            path
        } else {
            path.parent().unwrap()
        };
        ConfigFileLocation::from_dir(directory).unwrap_or_else(|_| ConfigFileLocation {
            path: path.join("typos.toml").to_path_buf(),
            config: None,
        })
    }

    // copied from typos_cli::config::Config::from_dir
    fn from_dir(dir: &Path) -> anyhow::Result<ConfigFileLocation> {
        assert!(
            dir.is_dir(),
            "Expected a directory that might contain a configuration file, got {:?}",
            dir.is_dir()
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

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    use super::ConfigFileLocation;

    #[test]
    fn test_from_dir_or_default_with_exact_path() -> anyhow::Result<()> {
        // when given a path to a configuration file, should resolve it to the same file

        // create a temporary directory on disk
        let dir = tempdir()?;

        let file_path = dir.path().join("typos.toml");
        let mut file = File::create(&file_path)?;
        writeln!(file, "#")?;
        let config_file_location = ConfigFileLocation::from_dir_or_default(&file_path);

        assert_eq!(
            config_file_location,
            ConfigFileLocation {
                path: file_path.to_path_buf(),
                config: Some(typos_cli::config::Config::default()),
            }
        );

        Ok(())
    }

    #[test]
    fn test_from_dir_or_default_with_directory() -> anyhow::Result<()> {
        // when given a path to a directory, should resolve it to the first configuration file
        // found in the directory. This should support all of the supported file names, although
        // this test only tests one of them.

        // NOTE when `dir` is dropped, the temporary directory is deleted from disk
        let dir = tempdir()?;
        let dir_path = dir.path();

        let config_file_location = ConfigFileLocation::from_dir_or_default(dir.path());

        assert_eq!(
            config_file_location,
            ConfigFileLocation {
                path: dir_path.join("typos.toml").to_path_buf(),
                config: None,
            }
        );

        Ok(())
    }
}
