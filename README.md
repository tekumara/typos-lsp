# typos-lsp

[![ci](https://github.com/tekumara/typos-lsp/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/tekumara/typos-lsp/actions/workflows/ci.yml)
[![release](https://github.com/tekumara/typos-lsp/actions/workflows/release.yml/badge.svg?event=release)](https://github.com/tekumara/typos-lsp/actions/workflows/release.yml)
![downloads](https://img.shields.io/github/downloads/tekumara/typos-lsp/total)

> **Source code spell checker for Visual Studio Code and LSP clients**

[typos](https://github.com/crate-ci/typos) is a low false-positive source code spell checker. This project exposes `typos` via a Language Server Protocol (LSP) server and Visual Studio Code extension to provide a fast, low memory, in-editor spell checker.

## Install

- VS Code: Install [Typos spell checker](https://marketplace.visualstudio.com/items?itemName=tekumara.typos-vscode) from the VSCode Marketplace.
- VSCodium: Install [Typos spell checker](https://open-vsx.org/extension/tekumara/typos-vscode) from the Open VSX Registry.
- Neovim: Install `typos-lsp` using [mason](https://mason-registry.dev/registry/list#typos-lsp).
- Zed: Install [Typos](https://zed.dev/extensions?query=typos) from Zed's extension marketplace.

### Other clients

For homebrew users:

```
brew install typos-lsp
```

For [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) users:

```
cargo binstall --git https://github.com/tekumara/typos-lsp typos-lsp
```

For Linux users (via Cargo):

```sh
cargo install --git https://github.com/tekumara/typos-lsp typos-lsp
```

Or manually download `typos-lsp` from the [releases page](https://github.com/tekumara/typos-lsp/releases).

## Configuration

For configuration see:

- [VS Code Settings](docs/vscode-settings.md)
- [Neovim LSP config](docs/neovim-lsp-config.md)
- [Helix config](docs/helix-config.md)
- [Zed config](docs/zed-config.md)

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

Restart the server after changing the config file for the new changes to take affect.

## Caveats

- File names are not spell checked.

## Why aren't my misspellings being corrected?

To minimise false-positives `typos` only suggests corrections for known misspellings, rather than unknown words like a traditional spell-checker. For more info see [Why was ... not corrected?](https://github.com/crate-ci/typos?tab=readme-ov-file#why-was--not-corrected).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) to get started.
