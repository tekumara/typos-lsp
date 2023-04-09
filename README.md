# typos-vscode

> **Source code spell checker for Visual Studio Code**

A Visual Studio Code Extension for [typos](https://github.com/crate-ci/typos) a low false-positive source code spell checker.

## Features

Identification of typos for any file type:

<img width="394" alt="Diagnostics example" src="https://user-images.githubusercontent.com/125105/230765737-230c92d7-c5db-4179-a22d-bb6aaf6d0aad.png">

## Usage

Once installed `typos` will automatically execute when you open or edit any file.

To disable typos per workspace, see [disable this extension](https://code.visualstudio.com/docs/editor/extension-marketplace#_disable-an-extension).

## Settings

This extension contributes the following settings:

- `typos.path`: Path to the `typos` binary.
- `typos.logLevel`: Logging level of the language server.
- `typos.trace.server`: Traces the communication between VS Code and the language server.

## Commands

| Command        | Description         |
| -------------- | ------------------- |
| Typos: Restart | Restart the server. |
