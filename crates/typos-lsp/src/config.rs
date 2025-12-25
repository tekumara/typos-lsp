use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use toml_edit::DocumentMut;

pub fn find_config_file_or_default(directory: &Path) -> PathBuf {
    assert!(
        directory.is_dir(),
        "Expected a directory that might contain a configuration file, got {:?}",
        directory.is_dir()
    );

    // adapted from typos_cli::config::Config::from_dir
    for file in typos_cli::config::SUPPORTED_FILE_NAMES {
        let config_path = directory.join(file);
        if typos_cli::config::Config::from_file(&config_path)
            .ok()
            .flatten()
            .is_some()
        {
            return config_path;
        }
    }

    //  no config file found in the directory, so provide a default typos.toml path
    directory.join("typos.toml")
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
    use tempfile::tempdir;

    #[test]
    fn test_find_config_file_found() -> anyhow::Result<()> {
        // when a configuration file is found in the directory, should return that file path
        let dir = tempdir()?;
        let dir_path = dir.path();
        let config_path = dir_path.join(".typos.toml");
        File::create(&config_path)?;

        assert_eq!(find_config_file_or_default(dir_path), config_path);

        Ok(())
    }

    #[test]
    fn test_find_config_file_missing() -> anyhow::Result<()> {
        // when no configuration file is found in the directory, should return the default typos.toml path
        let dir = tempdir()?;
        let dir_path = dir.path();

        assert_eq!(
            find_config_file_or_default(dir_path),
            dir_path.join("typos.toml").to_path_buf()
        );

        Ok(())
    }

    #[test]
    fn test_add_ignore_to_new_config_file() -> anyhow::Result<()> {
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
    fn test_add_ignore_to_existing_config_file() -> anyhow::Result<()> {
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
