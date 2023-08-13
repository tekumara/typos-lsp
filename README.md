# typos-vscode

> **Source code spell checker for Visual Studio Code**

[typos](https://github.com/crate-ci/typos) is a low false-positive source code spell checker. This Visual Studio Code extension provides a fast, low memory, in-editor spell checker by integrating with typos through the Language Server Protocol (LSP).

## Features

- Identify misspellings and provide a Quick Fix with suggested corrections:

<img width="373" alt="Diagnostics example with Quick Fix" src="https://user-images.githubusercontent.com/125105/232224205-eb9c6123-0d38-4d60-ac93-0990016453e0.png">

## Usage

Once installed `typos` will automatically execute when you open or edit any file.

To disable `typos` per workspace, see [disable this extension](https://code.visualstudio.com/docs/editor/extension-marketplace#_disable-an-extension).

## Config file support

Supports [config fields](https://github.com/crate-ci/typos/blob/master/docs/reference.md) in `typos.toml`, `_typos.toml`, or `.typos.toml`, except:

- `files.ignore*` - have no effect.
- `default.check-filename` - file names are never spell checked.
- `default.check-file` - files are always checked.
- `*.binary` - binary files are always checked.

Config files will be read from the workspace folder or its parents. If there is no workspace folder, then no config file will be read and the typos defaults will be used.

## Settings

This extension contributes the following settings:

- `typos.logLevel`: Logging level of the language server.
- `typos.path`: Path to the `typos-lsp` binary. If empty the bundled binary will be used.
- `typos.trace.server`: Traces the communication between VS Code and the language server.

## Commands

| Command        | Description         |
| -------------- | ------------------- |
| Typos: Restart | Restart the server. |

## Caveats

- File names are not spell checked.
- Server must be restarted after changing the config files (ie: typos.toml).
