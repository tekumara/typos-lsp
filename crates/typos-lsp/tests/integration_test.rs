use serde_json::{json, Value};
use std::{path::PathBuf, str::FromStr};
use tower_lsp::lsp_types::Url;
mod common;
use common::TestServer;
use {once_cell::sync::Lazy, regex::Regex};

#[test_log::test(tokio::test)]
async fn test_initialize_e2e() {
    let mut server = TestServer::new();

    similar_asserts::assert_eq!(
        server.request(&initialize()).await,
        json!(
          {
            "jsonrpc": "2.0",
            "result": {
              "capabilities": {
                "codeActionProvider": {
                  "codeActionKinds": ["quickfix"],
                  "workDoneProgress": false
                },
                "positionEncoding": "utf-16",
                "textDocumentSync": 1,
                "workspace": {
                  "workspaceFolders": { "changeNotifications": true, "supported": true }
                }
              },
              "serverInfo": { "name": "typos", "version": env!("CARGO_PKG_VERSION") }
            },
            "id": 1
          }
        )
    )
}

#[test_log::test(tokio::test)]
async fn test_code_action() {
    let did_open = did_open("this is an apropriate test\nfo typos\n");

    let code_action = json!(
      {
        "jsonrpc": "2.0",
        "method": "textDocument/codeAction",
        "params": {
          "textDocument": {
            "uri": "file:///C%3A/diagnostics.txt"
          },
          "range": range(1, 0, 2),
          "context": {
            "diagnostics": [ diag("`fo` should be `of`, `for`", 1, 0, 2) ],
            "only": ["quickfix"],
            "triggerKind": 1
          }
        },
        "id": 2
      }
    )
    .to_string();

    let code_action_insertion = json!(
      {
        "jsonrpc": "2.0",
        "method": "textDocument/codeAction",
        "params": {
          "textDocument": {
            "uri": "file:///C%3A/diagnostics.txt"
          },
          "range": range(0, 11, 21),
          "context": {
            "diagnostics": [ diag("`apropriate` should be `appropriate`", 0, 11, 21) ],
            "only": ["quickfix"],
            "triggerKind": 1
          }
        },
        "id": 3
      }
    )
    .to_string();

    let mut server = TestServer::new();
    let _ = server.request(&initialize()).await;

    similar_asserts::assert_eq!(
        server.request(&did_open).await,
        publish_diagnostics(&[
            diag("`apropriate` should be `appropriate`", 0, 11, 21),
            diag("`fo` should be `of`, `for`, `do`, `go`, `to`", 1, 0, 2)
        ])
    );

    similar_asserts::assert_eq!(
        server.request(&code_action).await,
        json!(
          {
            "jsonrpc": "2.0",
            "result": [
              {
                "diagnostics": [ diag("`fo` should be `of`, `for`", 1, 0, 2) ],
                "edit": {
                  "changes": {
                    "file:///C%3A/diagnostics.txt": [
                      {
                        "newText": "of",
                        "range": range(1, 0, 2)
                      }
                    ]
                  }
                },
                "kind": "quickfix",
                "title": "of"
              },
              {
                "diagnostics": [ diag("`fo` should be `of`, `for`", 1, 0, 2) ],
                "edit": {
                  "changes": {
                    "file:///C%3A/diagnostics.txt": [
                      {
                        "newText": "for",
                        "range": range(1, 0, 2)
                      }
                    ]
                  }
                },
                "kind": "quickfix",
                "title": "for"
              }
            ],
            "id": 2
          }
        ),
    );

    similar_asserts::assert_eq!(
        server.request(&code_action_insertion).await,
        json!(
          {
            "jsonrpc": "2.0",
            "result": [
              {
                "diagnostics": [ diag("`apropriate` should be `appropriate`", 0, 11, 21) ],
                "edit": {
                  "changes": {
                    "file:///C%3A/diagnostics.txt": [
                      {
                        "newText": "appropriate",
                        "range": range(0, 11, 21)
                      }
                    ]
                  }
                },
                "isPreferred": true,
                "kind": "quickfix",
                "title": "appropriate"
              }
            ],
            "id": 3
          }
        ),
    );
}

#[test_log::test(tokio::test)]
async fn test_config_file() {
    let workspace_folder_uri =
        Url::from_file_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")).unwrap();
    let diag_txt = workspace_folder_uri.join("tests/diagnostics.txt").unwrap();
    let changelog_md = workspace_folder_uri.join("tests/CHANGELOG.md").unwrap();

    let did_open_diag_txt = &did_open_with("fo typos", Some(&diag_txt));

    let did_open_changelog_md = &did_open_with("fo typos", Some(&changelog_md));

    let mut server = TestServer::new();
    let _ = server
        .request(&initialize_with(Some(&workspace_folder_uri), None))
        .await;

    // check "fo" is corrected to "of" because of default.extend-words
    similar_asserts::assert_eq!(
        server.request(&did_open_diag_txt).await,
        publish_diagnostics_with(&[diag("`fo` should be `of`", 0, 0, 2)], Some(&diag_txt))
    );

    // check changelog is excluded because of files.extend-exclude
    similar_asserts::assert_eq!(
        server.request(&did_open_changelog_md).await,
        publish_diagnostics_with(&[], Some(&changelog_md)),
    );
}

#[test_log::test(tokio::test)]
async fn test_custom_config_file() {
    let custom_config = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("custom_typos.toml");

    let workspace_folder_uri =
        Url::from_file_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")).unwrap();

    let diag_txt = workspace_folder_uri.join("tests/diagnostics.txt").unwrap();

    let did_open_diag_txt = &did_open_with("fo typos", Some(&diag_txt));

    let mut server = TestServer::new();
    let _ = server
        .request(&initialize_with(
            Some(&workspace_folder_uri),
            Some(&custom_config),
        ))
        .await;

    // check "fo" is corrected to "go" because of default.extend-words
    // in custom_typos.toml which overrides typos.toml
    similar_asserts::assert_eq!(
        server.request(&did_open_diag_txt).await,
        publish_diagnostics_with(&[diag("`fo` should be `go`", 0, 0, 2)], Some(&diag_txt))
    );
}

#[test_log::test(tokio::test)]
async fn test_custom_config_no_workspace_folder() {
    // mimics Neovim opening a file outside the root dir
    let custom_config = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("custom_typos.toml");

    let workspace_folder_uri =
        Url::from_file_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")).unwrap();

    let diag_txt = workspace_folder_uri.join("tests/diagnostics.txt").unwrap();

    let did_open_diag_txt = &did_open_with("fo typos", Some(&diag_txt));

    let mut server = TestServer::new();
    let _ = server
        .request(&initialize_with(None, Some(&custom_config)))
        .await;

    // check "fo" is corrected to "go" because of default.extend-words
    // in custom_typos.toml which overrides typos.toml
    similar_asserts::assert_eq!(
        server.request(&did_open_diag_txt).await,
        publish_diagnostics_with(&[diag("`fo` should be `go`", 0, 0, 2)], Some(&diag_txt))
    );
}

#[test_log::test(tokio::test)]
async fn test_non_file_uri() {
    // a Neovim toggleterm uri
    let term = Url::from_str("term://~/code/typos-lsp//59317:/bin/zsh;#toggleterm#1").unwrap();

    let did_open_diag_txt = &did_open_with("apropriate", Some(&term));

    let mut server = TestServer::new();
    let _ = server.request(&initialize_with(None, None)).await;

    similar_asserts::assert_eq!(
        server.request(&did_open_diag_txt).await,
        publish_diagnostics_with(
            &[diag("`apropriate` should be `appropriate`", 0, 0, 10)],
            Some(&term)
        )
    );
}

#[test_log::test(tokio::test)]
async fn test_empty_file_uri() {
    // eg: when using nvim telescope
    let term = Url::from_str("file:///").unwrap();

    let did_open_diag_txt = &did_open_with("apropriate", Some(&term));

    let mut server = TestServer::new();
    let _ = server.request(&initialize_with(None, None)).await;

    similar_asserts::assert_eq!(
        server.request(&did_open_diag_txt).await,
        publish_diagnostics_with(
            &[diag("`apropriate` should be `appropriate`", 0, 0, 10)],
            Some(&term)
        )
    );
}

#[test_log::test(tokio::test)]
async fn test_position_with_unicode_text() {
    let mut server = TestServer::new();
    let _ = server.request(&initialize()).await;

    // ¿ and é are two-byte code points in utf-8
    let unicode_text = &did_open("¿Qué hace él?");
    similar_asserts::assert_eq!(
        server.request(&unicode_text).await,
        publish_diagnostics(&[diag("`hace` should be `have`", 0, 5, 9)])
    );

    // ẽ has two code points U+0065 U+0303 (latin small letter e, combining tilde)
    let unicode_text = &did_open("ẽ hace");
    similar_asserts::assert_eq!(
        server.request(&unicode_text).await,
        publish_diagnostics(&[diag("`hace` should be `have`", 0, 3, 7)])
    );
}

#[test_log::test(tokio::test)]
async fn test_ignore_typos_in_config_files() {
    let term = Url::from_str("file:///C%3A/.typos.toml").unwrap();

    let did_open = &did_open_with("apropriate", Some(&term));

    let mut server = TestServer::new();
    let _ = server.request(&initialize_with(None, None)).await;

    similar_asserts::assert_eq!(
        server.request(&did_open).await,
        publish_diagnostics_with(&[], Some(&term))
    );
}

fn initialize() -> String {
    initialize_with(None, None)
}

fn initialize_with(workspace_folder_uri: Option<&Url>, custom_config: Option<&PathBuf>) -> String {
    let mut v = json!(
    {
      "jsonrpc": "2.0",
      "method": "initialize",
      "params": {
        "initializationOptions": {
          "diagnosticSeverity": "Warning"
        },
        "capabilities": {
          "textDocument": { "publishDiagnostics": { "dataSupport": true } }
        }
      },
      "id": 1
    });

    if let Some(uri) = workspace_folder_uri {
        v["params"]["workspaceFolders"] = json!([{ "uri": uri, "name": "tests" }]);
    }

    if let Some(config) = custom_config {
        v["params"]["initializationOptions"]["config"] = json!(config);
    }

    v.to_string()
}

fn did_open(text: &str) -> String {
    did_open_with(text, None)
}

fn did_open_with(text: &str, uri: Option<&Url>) -> String {
    json!(
    {
      "jsonrpc": "2.0",
      "method": "textDocument/didOpen",
      "params": {
        "textDocument": {
          "uri": uri.unwrap_or(&Url::parse("file:///C%3A/diagnostics.txt").unwrap()),
          "languageId": "plaintext",
          "version": 1,
          "text": text
        }
      }
    })
    .to_string()
}

fn diag(message: &str, line: u32, start: u32, end: u32) -> Value {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"`[^`]+` should be (.*)").unwrap());

    let caps = RE.captures(message).unwrap();

    let corrections: Vec<&str> = caps[1].split(", ").map(|s| s.trim_matches('`')).collect();

    json!({
      "data": { "corrections": corrections },
      "message": message,
      "range": range(line,start,end),
      "severity": 2,
      "source": "typos"
    })
}

fn range(line: u32, start: u32, end: u32) -> Value {
    json!({
      "end": { "character": end, "line": line },
      "start": { "character": start, "line": line }
    })
}

fn publish_diagnostics(diags: &[Value]) -> Value {
    publish_diagnostics_with(diags, None)
}

fn publish_diagnostics_with(diags: &[Value], uri: Option<&Url>) -> Value {
    json!({
      "jsonrpc": "2.0",
      "method": "textDocument/publishDiagnostics",
      "params": {
        "uri": uri.unwrap_or(&Url::parse("file:///C%3A/diagnostics.txt").unwrap()),
        "diagnostics": diags,
        "version": 1
      }
    })
}
