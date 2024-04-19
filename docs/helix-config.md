# Helix config

To enable typos for all files, add the following to your [languages.toml](https://docs.helix-editor.com/languages.html) file:

```toml
[language-server.typos]
# typos-lsp must be on your PATH, or otherwise change this to an absolute path to typos-lsp
command = "typos-lsp"
# Logging level of the language server. Defaults to error.
# Run with helix -v to output LSP logs to the editor log (:log-open)
environment = {"RUST_LOG" = "error"}
# Custom config. Used together with any workspace config files, taking precedence for
# settings declared in both. Equivalent to the typos `--config` cli argument.
config.config = "~/code/typos-lsp/crates/typos-lsp/tests/typos.toml"
# How typos are rendered in the editor, can be one of an Error, Warning, Information or Hint.
# Defaults to Warning.
config.diagnosticSeverity = "Warning"

[[language]]
name = "all-files"
scope = ""
file-types = [{glob="*"}]
language-servers = ["typos"]
```
