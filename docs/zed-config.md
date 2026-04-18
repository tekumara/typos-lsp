# Zed Settings

Zed configuration for the typos-lsp server. It's entirely optional and only needed if you want to customise typos-lsp.

Everything under `initialization_options` is passed to the server during initialization.

The `binary` section can be used to choose the executable, pass extra argv, or set process environment variables. See Zed’s [Configuring Languages](https://zed.dev/docs/configuring-languages) documentation.

Example:

```javascript
{
    "lsp": {
        "typos": {
            // Optional. Omit the entire "binary" object to use Zed’s default typos-lsp discovery.
            "binary": {
                // Prefer your install instead of auto-download when applicable.
                "ignore_system_version": false,
                "path": "/absolute/path/to/typos-lsp",
                "arguments": [],
                "env": {
                    // Rust tracing; shows in Zed's log when debugging the server process.
                    "RUST_LOG": "typos_lsp=debug"
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
                // Minimum logging level for the LSP, displayed in Zed's logs. "info" by default, can be:
                // "debug", "error", "info", "off", "trace", "warn"
                "logLevel": "info",
                // Traces the communication between ZED and the language server. Recommended for debugging only. "off" by default, can be:
                // "messages", "off", "verbose"
                "trace.server": "off"
            }
        }
    }
}
```
