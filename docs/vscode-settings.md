# VS Code Settings

This extension contributes the following settings:

- `typos.config`: Custom config. Used together with a config file found in the workspace or its parents, taking precedence for settings declared in both. Equivalent to the typos `--config` [cli argument](https://github.com/crate-ci/typos/blob/master/docs/reference.md).
- `typos.diagnosticSeverity`: How typos are rendered in the editor, can be one of an Error, Warning, Information or Hint.
- `typos.logLevel`: Logging level of the language server. Logs appear in the _Output -> Typos_ pane.
- `typos.path`: Path to the `typos-lsp` binary. If empty the bundled binary will be used.
- `typos.trace.server`: Traces the communication between VS Code and the language server. Recommended for debugging only.

To disable `typos` per workspace, see [disable this extension](https://code.visualstudio.com/docs/editor/extension-marketplace#_disable-an-extension).

## VS Code Commands

| Command        | Description         |
| -------------- | ------------------- |
| Typos: Restart | Restart the server. |
