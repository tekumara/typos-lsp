# typos-vscode

> **Source code spell checker for Visual Studio Code**

A Visual Studio Code Extension and LSP server for [typos](https://github.com/crate-ci/typos) a low false-positive source code spell checker.

## Features

- Identify misspellings and provide a Quick Fix with suggested corrections:

<img width="373" alt="Diagnostics example with Quick Fix" src="https://user-images.githubusercontent.com/125105/232224205-eb9c6123-0d38-4d60-ac93-0990016453e0.png">

## Usage

Once installed `typos` will automatically execute when you open or edit any file.

To disable `typos` per workspace, see [disable this extension](https://code.visualstudio.com/docs/editor/extension-marketplace#_disable-an-extension).

## Settings

This extension contributes the following settings:

- `typos.logLevel`: Logging level of the language server.
- `typos.path`: Path to the `typos-lsp` binary. If empty the bundled binary will be used.
- `typos.trace.server`: Traces the communication between VS Code and the language server.

## Commands

| Command        | Description         |
| -------------- | ------------------- |
| Typos: Restart | Restart the server. |
