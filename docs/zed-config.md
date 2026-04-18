# Zed Settings

Zed configuration for the typos-lsp server. It's entirely optional and only needed if you want to customise typos-lsp.

Everything under `initialization_options` is passed to the server during initialization.

Example:

```javascript
{
    "lsp": {
        "typos": {
            // Optional. Omit the entire "binary" object to use Zed’s default typos-lsp discovery.
            // See https://zed.dev/docs/configuring-languages
            "binary": {
                // Prefer your install instead of auto-downloaded version
                "ignore_system_version": false,
                "path": "/absolute/path/to/typos-lsp",
                "arguments": [],
                "env": {
                    // Logging level for the raw language server logs (defaults to error).
                    // Raw logs appear in the LSP Logs under Server Logs when Log level = Log
                    "RUST_LOG": "typos_lsp=error"
                }
            },
            "initialization_options": {
                // Custom config. Used together with a config file found in the workspace or its parents,
                // taking precedence for settings declared in both.
                // Equivalent to the typos `--config` cli argument.
                "config": "~/code/typos-lsp/crates/typos-lsp/tests/typos.toml",
                // Diagnostic severity within Zed. "Information" by default, can be:
                // "Error", "Hint", "Information", "Warning"
                "diagnosticSeverity": "Information",
            }
        }
    }
}
```
