
# Zed Settings

The Typos extension can be configured through a `.typos.toml` configuration file, which reference can be found [here](https://github.com/crate-ci/typos/blob/master/docs/reference.md).

Additionally, you can configure it in Zed's settings with the following:

```javascript
{
    "lsp": {
        "typos": {
            "initialization_options": {
                // Path to your typos config file, .typos.toml by default.
                "config": ".typos.toml",
                // Path to your typos-lsp executable, takes $PATH into account.
                "path": "typos-lsp",
                // Diagnostic severity within Zed. "Error" by default, can be:
                // "Error", "Hint", "Information", "Warning"
                "diagnosticSeverity": "Error",
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

**WARNING**: When modifying your Typos configuration either in `.typos.toml` or `Cargo.toml` you will need to reload the workspace to take them into account.

You do not need to reload when editing Zed's `settings.json`.