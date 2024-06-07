# typos-lsp

[![ci](https://github.com/tekumara/typos-lsp/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/tekumara/typos-lsp/actions/workflows/ci.yml)
[![release](https://github.com/tekumara/typos-lsp/actions/workflows/release.yml/badge.svg?event=release)](https://github.com/tekumara/typos-lsp/actions/workflows/release.yml)
![downloads](https://img.shields.io/github/downloads/tekumara/typos-lsp/total)

> **Source code spell checker for Visual Studio Code and LSP clients**

[typos](https://github.com/crate-ci/typos) is a low false-positive source code spell checker. This project exposes `typos` via a Language Server Protocol (LSP) server and Visual Studio Code extension to provide a fast, low memory, in-editor spell checker.

## Install

- Vscode: Install [Typos spell checker](https://marketplace.visualstudio.com/items?itemName=tekumara.typos-vscode) from the VSCode Marketplace.
- VSCodium: Install [Typos spell checker](https://open-vsx.org/extension/tekumara/typos-vscode) from the Open VSX Registry.
- Neovim: Install `typos-lsp` using [mason](https://mason-registry.dev/registry/list#typos-lsp) or download `typos-lsp` from the [releases page](https://github.com/tekumara/typos-lsp/releases).
- Helix: Download `typos-lsp` from the [releases page](https://github.com/tekumara/typos-lsp/releases) and place it on your PATH.
- Other clients: Download `typos-lsp` from the [releases page](https://github.com/tekumara/typos-lsp/releases).

For configuration see:

- [VS Code Settings](docs/vscode-settings.md)
- [Neovim LSP config](docs/neovim-lsp-config.md)
- [Helix config](docs/helix-config.md)

## Features

<!-- markdownlint-disable-file MD033 -->

- Identify misspellings and provide a Quick Fix with suggested corrections:

    <img width="373" alt="Diagnostics example with Quick Fix" src="https://user-images.githubusercontent.com/125105/232224205-eb9c6123-0d38-4d60-ac93-0990016453e0.png">

## Usage

Once installed `typos` will automatically execute when you open or edit any file.

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

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) to get started.
