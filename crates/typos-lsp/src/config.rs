use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use toml_edit::DocumentMut;

/// Represents a path to a typos_cli config file and, if it contains a configuration file, the file
/// contents.
///
/// When reading a config from a directory, many configuration files are supported, and only one is
/// chosen in a given order. Shows the name of the config file that is used ("typos.toml",
/// "_typos.toml", ".typos.toml", "pyproject.toml"). This information is useful when we want to
/// modify the config file later on.
#[derive(Debug, Clone)]
pub struct ConfigPath {
    pub path: PathBuf,
    pub config: Option<typos_cli::config::Config>,
}

impl PartialEq for ConfigPath {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && format!("{:?}", self.config) == format!("{:?}", other.config)
    }
}

impl ConfigPath {
    pub fn from_file_path_or_default(path: &Path) -> ConfigPath {
        let config = typos_cli::config::Config::from_file(path).ok().flatten();

        ConfigPath {
            path: path.to_path_buf(),
            config,
        }
    }

    pub fn from_dir_or_default(path: &Path) -> ConfigPath {
        let directory = if path.is_dir() {
            path
        } else {
            path.parent().unwrap()
        };
        ConfigPath::from_dir(directory).unwrap_or_else(|_| ConfigPath {
            path: path.join("typos.toml").to_path_buf(),
            config: None,
        })
    }

    // copied from typos_cli::config::Config::from_dir
    fn from_dir(dir: &Path) -> anyhow::Result<ConfigPath> {
        assert!(
            dir.is_dir(),
            "Expected a directory that might contain a configuration file, got {:?}",
            dir.is_dir()
        );

        for file in typos_cli::config::SUPPORTED_FILE_NAMES {
            let path = dir.join(file);
            if let Ok(Some(config)) = typos_cli::config::Config::from_file(path.as_path()) {
                return Ok(ConfigPath {
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

pub fn add_ignore(config_file_path: &Path, typo: &str) -> anyhow::Result<()> {
    let input = match std::fs::read_to_string(config_file_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            return Err(anyhow::Error::new(e).context(format!(
                "Cannot read config file at {}",
                config_file_path.display()
            )))
        }
    };

    // preserve comments and formatting
    let mut document = input
        .parse::<DocumentMut>()
        .with_context(|| anyhow!("Cannot parse config file at {}", config_file_path.display()))?;
    let extend_words = document
        .entry("default")
        .or_insert(toml_edit::table())
        .as_table_mut()
        .context("Cannot get 'default' table")?
        .entry("extend-words")
        .or_insert(toml_edit::table())
        .as_table_mut()
        .context("Cannot get 'extend-words' table")?;
    extend_words[typo] = toml_edit::value(typo);

    std::fs::write(config_file_path, document.to_string())
        .with_context(|| anyhow!("Cannot write config file to {}", config_file_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_from_dir_or_default_with_exact_path() -> anyhow::Result<()> {
        // when given a path to a configuration file, should resolve it to the same file

        // create a temporary directory on disk
        let dir = tempdir()?;

        let file_path = dir.path().join("typos.toml");
        let mut file = File::create(&file_path)?;
        writeln!(file, "#")?;

        assert_eq!(
            ConfigPath::from_dir_or_default(&file_path),
            ConfigPath {
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

        assert_eq!(
            ConfigPath::from_dir_or_default(dir.path()),
            ConfigPath {
                path: dir_path.join("typos.toml").to_path_buf(),
                config: None,
            }
        );

        Ok(())
    }

    #[test]
    fn test_add_ignore_to_new_file() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.toml");

        add_ignore(&file_path, "typo")?;

        let content = std::fs::read_to_string(&file_path)?;
        similar_asserts::assert_eq!(
            content,
            [
                "[default]",
                "",
                "[default.extend-words]",
                "typo = \"typo\"",
                ""
            ]
            .join("\n")
        );
        Ok(())
    }

    #[test]
    fn test_add_ignore_to_existing_file() -> anyhow::Result<()> {
        // should preserve comments and formatting

        let existing_file = [
            "[files] # comment",
            "# comment",
            "extend-exclude = [\"CHANGELOG.md\", \"crates/typos-lsp/tests/integration_test.rs\"]",
        ]
        .join("\n");

        // make sure the config is valid (so the test makes sense)
        let _ = typos_cli::config::Config::from_toml(&existing_file)?;

        let dir = tempdir()?;
        let file_path = dir.path().join("test.toml");
        std::fs::write(&file_path, existing_file)?;

        add_ignore(&file_path, "typo")?;

        let content = std::fs::read_to_string(&file_path)?;
        similar_asserts::assert_eq!(
            content,
            [
                "[files] # comment",
                "# comment",
                "extend-exclude = [\"CHANGELOG.md\", \"crates/typos-lsp/tests/integration_test.rs\"]",
                "",
                "[default]",
                "",
                "[default.extend-words]",
                "typo = \"typo\"",
                ""
            ]
            .join("\n")
        );

        Ok(())
    }
}
