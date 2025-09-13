# Vim LSP Settings

## Install

Add the following plugins to vimrc using [vim-plug](https://github.com/junegunn/vim-plug):

```vim
Plug 'prabirshrestha/vim-lsp'
Plug 'mattn/vim-lsp-settings'
```

By default [vim-lsp-settings](https://github.com/mattn/vim-lsp-settings) disables typos-lsp, preventing it from being installed or used. To enable it in vimrc:

```vim
let g:lsp_settings = { 'typos-lsp': {'disabled': v:false} }
```

Install typos-lsp:

```
:LspInstallServer! typos-lsp
```

## Configure

To make use of typos.toml config files, typos-lsp needs to know the [workspace folder](https://github.com/prabirshrestha/vim-lsp/blob/0094b5e0fc7db02e0a35d942e5e7f2069ccdbbfc/doc/vim-lsp.txt#L2259) (aka `root_uri`). To enable workspace folders (experimental):

```vim
let g:lsp_experimental_workspace_folders = 1
```

You can configure the following [settings](https://github.com/mattn/vim-lsp-settings/blob/master/settings/typos-lsp.vim):

### Configuration Options

- **`cmd`**: Absolute path to the typos-lsp executable. Defaults to the location of the bundled version in vim-lsp-settings.

- **`env`**: Environment variables to set when running typos-lsp. Use `RUST_LOG` to change the log level (defaults to error), eg: `{ 'RUST_LOG': 'info' }`.

- **`initialization_options.config`**: Custom configuration file path. This is used together with any config file found in the workspace or its parents, with this custom config taking precedence for settings declared in both. Equivalent to the typos `--config` CLI argument.

- **`initialization_options.diagnosticSeverity`**: How typos are rendered in the editor. Can be one of `Error`, `Warning`, `Info`, or `Hint`. Defaults to `Info` if not specified.

Example:

```vim
let g:lsp_settings = {
\  'typos-lsp': {
\    'disabled' : v:false,
\    'env': { 'RUST_LOG': 'info' },
\    'cmd': ['/absolute/path/typos-lsp'],
\    'initialization_options': {
\      'config': '~/code/typos-lsp/crates/typos-lsp/tests/typos.toml',
\      'diagnosticSeverity': 'Info'
\    }
\  }
\}
```

## Logging

To enable LSP logging:

```vim
let g:lsp_log_verbose = 1
let g:lsp_log_file = expand('~/.vim/lsp.log')
```

Logs from typos-lsp will appear in the LSP log file as coming from stderr.
