use std::path::PathBuf;
use tower_lsp::lsp_types::Url;
mod common;
use common::TestServer;

fn initialize() -> String {
    initialize_with(None, None)
}

fn initialize_with(workspace_folder_uri: Option<&Url>, custom_config: Option<&PathBuf>) -> String {
    let workspace_folders = workspace_folder_uri.map_or(String::default(), |v| {
        format!(
            r#",
            "workspaceFolders": [
              {{
                "uri": "{}",
                "name": "tests"
              }}
            ]"#,
            v
        )
    });

    let config = custom_config.map_or(String::default(), |v| {
        format!(
            r#", "config": "{}""#,
            v.to_string_lossy().replace("\\", "\\\\") // escape windows path separators to make valid json
        )
    });

    format!(
        r#"{{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {{
    "initializationOptions": {{
      "diagnosticSeverity": "Warning"{}
    }},
    "capabilities": {{
      "textDocument": {{ "publishDiagnostics": {{ "dataSupport": true }} }}
    }}{}
  }},
  "id": 1
}}
"#,
        config, workspace_folders,
    )
}

fn did_open(text: &str) -> String {
    did_open_with(text, None)
}

fn did_open_with(text: &str, uri: Option<&Url>) -> String {
    format!(
        r#"{{
    "jsonrpc": "2.0",
    "method": "textDocument/didOpen",
    "params": {{
      "textDocument": {{
        "uri": "{}",
        "languageId": "plaintext",
        "version": 1,
        "text": "{}"
      }}
    }}
  }}"#,
        uri.unwrap_or(&Url::parse("file:///diagnostics.txt").unwrap()),
        text.replace("\n", "\\n")
    )
}

#[test_log::test(tokio::test)]
async fn test_initialize_e2e() {
    let mut server = TestServer::new();

    similar_asserts::assert_eq!(
        server.request(&initialize()).await,
        format!(
            r#"{{"jsonrpc":"2.0","result":{{"capabilities":{{"codeActionProvider":{{"codeActionKinds":["quickfix"],"workDoneProgress":false}},"textDocumentSync":1,"workspace":{{"workspaceFolders":{{"changeNotifications":true,"supported":true}}}}}},"serverInfo":{{"name":"typos","version":"{}"}}}},"id":1}}"#,
            env!("CARGO_PKG_VERSION")
        )
    )
}

#[test_log::test(tokio::test)]
async fn test_code_action() {
    let did_open = &did_open("this is an apropriate test\nfo typos\n");

    let code_action = r#"
    {
        "jsonrpc": "2.0",
        "method": "textDocument/codeAction",
        "params": {
          "textDocument": {
            "uri": "file:///diagnostics.txt"
          },
          "range": {
            "start": {
              "line": 1,
              "character": 5
            },
            "end": {
              "line": 1,
              "character": 7
            }
          },
          "context": {
            "diagnostics": [
              {
                "range": {
                  "start": {
                    "line": 1,
                    "character": 5
                  },
                  "end": {
                    "line": 1,
                    "character": 7
                  }
                },
                "message": "`fo` should be `of`, `for`",
                "data": {
                    "corrections": ["of", "for"]
                },
                "severity": 2,
                "source": "typos"
              }
            ],
            "only": ["quickfix"],
            "triggerKind": 1
          }
        },
        "id": 2
      }
    "#;

    let code_action_insertion = r#"
    {
        "jsonrpc": "2.0",
        "method": "textDocument/codeAction",
        "params": {
          "textDocument": {
            "uri": "file:///diagnostics.txt"
          },
          "range": {
            "start": {
              "line": 0,
              "character": 11
            },
            "end": {
              "line": 0,
              "character": 21
            }
          },
          "context": {
            "diagnostics": [
              {
                "range": {
                  "start": {
                    "line": 0,
                    "character": 11
                  },
                  "end": {
                    "line": 0,
                    "character": 21
                  }
                },
                "message": "`apropriate` should be `appropriate`",
                "data": {
                  "corrections": ["appropriate"]
                },
                "severity": 2,
                "source": "typos"
              }
            ],
            "only": ["quickfix"],
            "triggerKind": 1
          }
        },
        "id": 3
      }
    "#;

    let mut server = TestServer::new();
    let _ = server.request(&initialize()).await;

    similar_asserts::assert_eq!(
        server.request(did_open).await,
        r#"{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"diagnostics":[{"data":{"corrections":["appropriate"]},"message":"`apropriate` should be `appropriate`","range":{"end":{"character":21,"line":0},"start":{"character":11,"line":0}},"severity":2,"source":"typos"},{"data":{"corrections":["of","for","do","go","to"]},"message":"`fo` should be `of`, `for`, `do`, `go`, `to`","range":{"end":{"character":2,"line":1},"start":{"character":0,"line":1}},"severity":2,"source":"typos"}],"uri":"file:///diagnostics.txt","version":1}}"#,
    );

    similar_asserts::assert_eq!(
        server.request(code_action).await,
        r#"{"jsonrpc":"2.0","result":[{"diagnostics":[{"data":{"corrections":["of","for"]},"message":"`fo` should be `of`, `for`","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}},"severity":2,"source":"typos"}],"edit":{"changes":{"file:///diagnostics.txt":[{"newText":"of","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}}}]}},"kind":"quickfix","title":"of"},{"diagnostics":[{"data":{"corrections":["of","for"]},"message":"`fo` should be `of`, `for`","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}},"severity":2,"source":"typos"}],"edit":{"changes":{"file:///diagnostics.txt":[{"newText":"for","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}}}]}},"kind":"quickfix","title":"for"}],"id":2}"#,
    );

    similar_asserts::assert_eq!(
        server.request(code_action_insertion).await,
        r#"{"jsonrpc":"2.0","result":[{"diagnostics":[{"data":{"corrections":["appropriate"]},"message":"`apropriate` should be `appropriate`","range":{"end":{"character":21,"line":0},"start":{"character":11,"line":0}},"severity":2,"source":"typos"}],"edit":{"changes":{"file:///diagnostics.txt":[{"newText":"appropriate","range":{"end":{"character":21,"line":0},"start":{"character":11,"line":0}}}]}},"isPreferred":true,"kind":"quickfix","title":"appropriate"}],"id":3}"#,
    );
}

#[test_log::test(tokio::test)]
async fn test_config_file() {
    let workspace_folder_uri =
        Url::from_file_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")).unwrap();
    let diag_txt = workspace_folder_uri.join("tests/diagnostics.txt").unwrap();
    let changelog_md = workspace_folder_uri.join("tests/CHANGELOG.md").unwrap();

    let did_open_diag_txt =
        &did_open_with("this is an apropriate test\nfo typos\n", Some(&diag_txt));

    let did_open_changelog_md = &did_open_with(
        "this is an apropriate test\nfo typos\n",
        Some(&changelog_md),
    );

    let mut server = TestServer::new();
    let _ = server
        .request(&initialize_with(Some(&workspace_folder_uri), None))
        .await;

    // check "fo" is corrected to "of" because of default.extend-words
    similar_asserts::assert_eq!(
        server.request(&did_open_diag_txt).await,
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"diagnostics":[{{"data":{{"corrections":["appropriate"]}},"message":"`apropriate` should be `appropriate`","range":{{"end":{{"character":21,"line":0}},"start":{{"character":11,"line":0}}}},"severity":2,"source":"typos"}},{{"data":{{"corrections":["of"]}},"message":"`fo` should be `of`","range":{{"end":{{"character":2,"line":1}},"start":{{"character":0,"line":1}}}},"severity":2,"source":"typos"}}],"uri":"{}","version":1}}}}"#,
            diag_txt
        ),
    );

    // check changelog is excluded because of files.extend-exclude
    similar_asserts::assert_eq!(
        server.request(&did_open_changelog_md).await,
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"diagnostics":[],"uri":"{}","version":1}}}}"#,
            changelog_md
        ),
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

    let did_open_diag_txt =
        &did_open_with("this is an apropriate test\nfo typos\n", Some(&diag_txt));

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
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"diagnostics":[{{"data":{{"corrections":["appropriate"]}},"message":"`apropriate` should be `appropriate`","range":{{"end":{{"character":21,"line":0}},"start":{{"character":11,"line":0}}}},"severity":2,"source":"typos"}},{{"data":{{"corrections":["go"]}},"message":"`fo` should be `go`","range":{{"end":{{"character":2,"line":1}},"start":{{"character":0,"line":1}}}},"severity":2,"source":"typos"}}],"uri":"{}","version":1}}}}"#,
            diag_txt
        ),
    );
}

#[test_log::test(tokio::test)]
async fn test_unicode_diagnostics() {
    let did_open = &did_open("¿Qué hace él?");

    let mut server = TestServer::new();
    let _ = server.request(&initialize()).await;

    // start position should count graphemes with multiple code points as one visible character
    similar_asserts::assert_eq!(
        server.request(&did_open).await,
        r#"{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"diagnostics":[{"data":{"corrections":["have"]},"message":"`hace` should be `have`","range":{"end":{"character":9,"line":0},"start":{"character":5,"line":0}},"severity":2,"source":"typos"}],"uri":"file:///diagnostics.txt","version":1}}"#,
    );
}
