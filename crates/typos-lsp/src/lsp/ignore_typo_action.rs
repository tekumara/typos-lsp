use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use toml_edit::DocumentMut;

pub(super) const IGNORE_IN_PROJECT: &str = "ignore-in-project";

pub(super) fn ignore_typo_in_config_file(config_file: PathBuf, typo: String) -> anyhow::Result<()> {
    let input = read_to_string(&config_file)
        .with_context(|| anyhow!("Cannot read config file at {}", config_file.display()))
        .unwrap_or("".to_string());

    let document = add_typo(input, typo, &config_file)?;

    std::fs::write(&config_file, document.to_string())
        .with_context(|| anyhow!("Cannot write config file to {}", config_file.display()))?;

    Ok(())
}

fn add_typo(
    input: String,
    typo: String,
    config_file_path: &Path,
) -> Result<DocumentMut, anyhow::Error> {
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
    extend_words[typo.as_str()] = toml_edit::value(typo.clone());
    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_typo_to_empty_file() {
        let empty_file = "";
        let document = add_typo(
            empty_file.to_string(),
            "typo".to_string(),
            PathBuf::from("test.toml").as_path(),
        )
        .unwrap();

        similar_asserts::assert_eq!(
            document.to_string(),
            [
                "[default]",
                "",
                "[default.extend-words]",
                "typo = \"typo\"",
                ""
            ]
            .join("\n")
        );
    }

    #[test]
    fn test_add_typo_to_existing_file() -> anyhow::Result<()> {
        // should preserve comments and formatting

        let existing_file = [
            "[files] # comment",
            "# comment",
            "extend-exclude = [\"CHANGELOG.md\", \"crates/typos-lsp/tests/integration_test.rs\"]",
        ]
        .join("\n");

        // make sure the config is valid (so the test makes sense)
        let _ = typos_cli::config::Config::from_toml(&existing_file)?;

        let document = add_typo(
            existing_file.to_string(),
            "typo".to_string(),
            PathBuf::from("test.toml").as_path(),
        )?;

        similar_asserts::assert_eq!(
            document.to_string(),
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
