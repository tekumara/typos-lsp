# Helix config

In your [languages.toml](https://docs.helix-editor.com/languages.html) file configure the `typos` language server as follows:

```toml
[language-server.typos]
# typos-lsp must be on your PATH, or otherwise change this to an absolute path to typos-lsp
command = "typos-lsp"
# Logging level of the language server. Defaults to error.
# Run with helix -v to output LSP logs to the editor log (:log-open)
environment = {"RUST_LOG" = "error"}
# Custom config. Used together with a config file found in the workspace or its parents,
# taking precedence for settings declared in both. Equivalent to the typos `--config` cli argument.
config.config = "~/code/typos-lsp/crates/typos-lsp/tests/typos.toml"
# How typos are rendered in the editor, can be one of an Error, Warning, Info or Hint.
# Defaults to Info.
config.diagnosticSeverity = "Info"
```

Then add `typos` to one or more languages, as the last entry to avoid taking precedence, eg:

```toml
[[language]]
name = "rust"
language-servers = ["rust-analyzer", "typos"]
```

Currently it doesn't seem possible to add typos to all languages at once, see [this discussion](https://github.com/helix-editor/helix/discussions/8850).
