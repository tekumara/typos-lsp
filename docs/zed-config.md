# Zed Settings

Example Zed configuration for the typos-lsp server:

```javascript
{
    "lsp": {
        "typos": {
            "initialization_options": {
                // Path to your typos config file, .typos.toml by default.
                "config": ".typos.toml",
                // Path to the typos-lsp binary, can be on $PATH or be an absolute path.
                // If empty the bundled binary will be used.
                "path": "typos-lsp",
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

You do not need to reload when editing Zed's `settings.json`.
