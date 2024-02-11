# typos-lsp

[![ci](https://github.com/tekumara/typos-lsp/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/tekumara/typos-lsp/actions/workflows/ci.yml)
[![release](https://github.com/tekumara/typos-lsp/actions/workflows/release.yml/badge.svg?event=release)](https://github.com/tekumara/typos-lsp/actions/workflows/release.yml)

> **Source code spell checker for Visual Studio Code and LSP clients**

[typos](https://github.com/crate-ci/typos) is a low false-positive source code spell checker. This project exposes `typos` via a Language Server Protocol (LSP) server and Visual Studio Code extension to provide a fast, low memory, in-editor spell checker.

## Install

- Vscode: Install [Typos spell checker](https://marketplace.visualstudio.com/items?itemName=tekumara.typos-vscode) from the VSCode Marketplace.
- Other clients: Download `typos-lsp` from the [releases page](https://github.com/tekumara/typos-lsp/releases).
- Neovim: for a LSP config see [neovim/nvim-lspconfig](https://github.com/neovim/nvim-lspconfig/blob/master/doc/server_configurations.md#typos_lsp).

## Features

<!-- markdownlint-disable-file MD033 -->

- Identify misspellings and provide a Quick Fix with suggested corrections:

    <img width="373" alt="Diagnostics example with Quick Fix" src="https://user-images.githubusercontent.com/125105/232224205-eb9c6123-0d38-4d60-ac93-0990016453e0.png">

## Usage

Once installed `typos` will automatically execute when you open or edit any file.

## VS Code Settings

This extension contributes the following settings:

- `typos.config`: Custom config. Used together with any workspace config files, taking precedence for settings declared in both. Equivalent to the typos `--config` [cli argument](https://github.com/crate-ci/typos/blob/master/docs/reference.md).
- `typos.diagnosticSeverity`: How typos are rendered in the editor, eg: as errors, warnings, information, or hints.
- `typos.logLevel`: Logging level of the language server. Logs appear in the _Output -> Typos_ pane.
- `typos.path`: Path to the `typos-lsp` binary. If empty the bundled binary will be used.
- `typos.trace.server`: Traces the communication between VS Code and the language server. Recommended for debugging only.

To disable `typos` per workspace, see [disable this extension](https://code.visualstudio.com/docs/editor/extension-marketplace#_disable-an-extension).

## VS Code Commands

| Command        | Description         |
| -------------- | ------------------- |
| Typos: Restart | Restart the server. |

## Neovim Settings

Example config when using [neovim/nvim-lspconfig](https://github.com/neovim/nvim-lspconfig/blob/master/doc/server_configurations.md#typos_lsp):

```lua
local lspconfig = require('lspconfig')
require('lspconfig').typos_lsp.setup({
    config = {
        -- Logging level of the language server. Logs appear in :LspLog. Defaults to error.
        cmd_env = { RUST_LOG = "error" }
    },
    init_options = {
        -- Custom config. Used together with any workspace config files, taking precedence for
        -- settings declared in both. Equivalent to the typos `--config` cli argument.
        config = '~/code/typos-lsp/crates/typos-lsp/tests/typos.toml',
        -- How typos are rendered in the editor, eg: as errors, warnings, information, or hints.
        -- Defaults to error.
        diagnosticSeverity = "Error"
    }
})
```

## Config file support

Supports [config fields](https://github.com/crate-ci/typos/blob/master/docs/reference.md) in `typos.toml`, `_typos.toml`, or `.typos.toml`, except:

- `files.ignore*` - have no effect.
- `default.check-filename` - file names are never spell checked.
- `default.check-file` - files are always checked.
- `*.binary` - binary files are always checked.

Config files will be read from the workspace folder or its parents. If there is no workspace folder, then no config file will be read and the typos defaults will be used.

## Caveats

- File names are not spell checked.
- Server must be restarted after changing the config files (ie: typos.toml).

## Why aren't my misspellings being corrected?

To minimise false-positives `typos` only suggests corrections for known misspellings, rather than unknown words like a traditional spell-checker. For more info see [Why was ... not corrected?](https://github.com/crate-ci/typos?tab=readme-ov-file#why-was--not-corrected).
