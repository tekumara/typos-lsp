# typos-vscode

[![ci](https://github.com/tekumara/typos-vscode/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/tekumara/typos-vscode/actions/workflows/ci.yml)
[![release](https://github.com/tekumara/typos-vscode/actions/workflows/release.yml/badge.svg?event=release)](https://github.com/tekumara/typos-vscode/actions/workflows/release.yml)

> **Source code spell checker for Visual Studio Code**

[typos](https://github.com/crate-ci/typos) is a low false-positive source code spell checker. This Visual Studio Code extension provides a fast, low memory, in-editor spell checker by exposing typos via the Language Server Protocol (LSP).

## Install

Install [Typos spell checker](https://marketplace.visualstudio.com/items?itemName=tekumara.typos-vscode) from the VSCode Marketplace.

To use the LSP server `typos-lsp` independently of VS Code download it from the [releases page](https://github.com/tekumara/typos-vscode/releases).

For a Neovim LSP config see [neovim/nvim-lspconfig](https://github.com/neovim/nvim-lspconfig).

## Features

<!-- markdownlint-disable-file MD033 -->

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

- `typos.config`: Custom config. Used together with any workspace config files, taking precedence for settings declared in both. Equivalent to the typos `--config` [cli argument](https://github.com/crate-ci/typos/blob/master/docs/reference.md).
- `typos.diagnosticSeverity`: How typos are rendered in the editor, eg: as errors, warnings, information, or hints.
- `typos.logLevel`: Logging level of the language server. Logs appear in the _Output -> Typos_ pane.
- `typos.path`: Path to the `typos-lsp` binary. If empty the bundled binary will be used.
- `typos.trace.server`: Traces the communication between VS Code and the language server. Recommended for debugging only.

## Commands

| Command        | Description         |
| -------------- | ------------------- |
| Typos: Restart | Restart the server. |

## Caveats

- File names are not spell checked.
- Server must be restarted after changing the config files (ie: typos.toml).
