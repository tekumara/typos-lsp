use std::path::PathBuf;
use tower_lsp::lsp_types::Url;
mod common;
use common::TestServer;

#[test_log::test(tokio::test)]
async fn test_initialize_e2e() {
    let mut server = TestServer::new();

    similar_asserts::assert_eq!(
        server
            .request(
                r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}"#
            )
            .await,
        format!(
            r#"{{"jsonrpc":"2.0","result":{{"capabilities":{{"codeActionProvider":{{"codeActionKinds":["quickfix"],"workDoneProgress":false}},"textDocumentSync":1,"workspace":{{"workspaceFolders":{{"changeNotifications":true,"supported":true}}}}}},"serverInfo":{{"name":"typos","version":"{}"}}}},"id":1}}"#,
            env!("CARGO_PKG_VERSION")
        )
    )
}

#[test_log::test(tokio::test)]
async fn test_e2e() {
    let initialize = r#"{
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
      }
    "#;

    let did_open = r#"{
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
              "textDocument": {
                "uri": "file:///diagnostics.txt",
                "languageId": "plaintext",
                "version": 1,
                "text": "this is an apropriate test\nfo typos\n"
              }
            }
          }
        "#;

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
    let _ = server.request(initialize).await;

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
async fn test_config_file_e2e() {
    let workspace_folder_uri =
        Url::from_file_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")).unwrap();

    let initialize = format!(
        r#"{{
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {{
          "initializationOptions": {{
            "diagnosticSeverity": "Warning"
          }},
          "capabilities": {{
            "textDocument": {{ "publishDiagnostics": {{ "dataSupport": true }} }}
          }},
          "workspaceFolders": [
            {{
              "uri": "{}",
              "name": "tests"
            }}
          ]
        }},
        "id": 1
      }}
    "#,
        workspace_folder_uri
    );

    let did_open_diag_txt = format!(
        r#"{{
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {{
              "textDocument": {{
                "uri": "{}/diagnostics.txt",
                "languageId": "plaintext",
                "version": 1,
                "text": "this is an apropriate test\nfo typos\n"
              }}
            }}
          }}
        "#,
        workspace_folder_uri
    );

    let did_open_changelog = format!(
        r#"{{
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {{
              "textDocument": {{
                "uri": "{}/CHANGELOG.md",
                "languageId": "plaintext",
                "version": 1,
                "text": "this is an apropriate test\nfo typos\n"
              }}
            }}
          }}
        "#,
        workspace_folder_uri
    );

    let mut server = TestServer::new();
    let _ = server.request(&initialize).await;

    // check "fo" is corrected to "of" because of default.extend-words
    similar_asserts::assert_eq!(
        server.request(&did_open_diag_txt).await,
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"diagnostics":[{{"data":{{"corrections":["appropriate"]}},"message":"`apropriate` should be `appropriate`","range":{{"end":{{"character":21,"line":0}},"start":{{"character":11,"line":0}}}},"severity":2,"source":"typos"}},{{"data":{{"corrections":["of"]}},"message":"`fo` should be `of`","range":{{"end":{{"character":2,"line":1}},"start":{{"character":0,"line":1}}}},"severity":2,"source":"typos"}}],"uri":"{}/diagnostics.txt","version":1}}}}"#,
            workspace_folder_uri
        ),
    );

    // check changelog is excluded because of files.extend-exclude
    similar_asserts::assert_eq!(
        server.request(&did_open_changelog).await,
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"diagnostics":[],"uri":"{}/CHANGELOG.md","version":1}}}}"#,
            workspace_folder_uri
        ),
    );
}

#[test_log::test(tokio::test)]
async fn test_custom_config_file_e2e() {
    let workspace_folder_uri =
        Url::from_file_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")).unwrap();

    let custom_config = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("custom_typos.toml");

    let initialize = format!(
        r#"{{
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {{
          "initializationOptions": {{
            "diagnosticSeverity": "Warning",
            "config": "{}"
          }},
          "capabilities": {{
            "textDocument": {{ "publishDiagnostics": {{ "dataSupport": true }} }}
          }},
          "workspaceFolders": [
            {{
              "uri": "{}",
              "name": "tests"
            }}
          ]
        }},
        "id": 1
      }}
    "#,
        custom_config.to_string_lossy().replace("\\", "\\\\"), // escape windows path separators to make valid json
        workspace_folder_uri,
    );

    let did_open_diag_txt = format!(
        r#"{{
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {{
              "textDocument": {{
                "uri": "{}/diagnostics.txt",
                "languageId": "plaintext",
                "version": 1,
                "text": "this is an apropriate test\nfo typos\n"
              }}
            }}
          }}
        "#,
        workspace_folder_uri
    );

    let mut server = TestServer::new();
    let _ = server.request(&initialize).await;

    // check "fo" is corrected to "go" because of default.extend-words
    // in custom_typos.toml which overrides typos.toml
    similar_asserts::assert_eq!(
        server.request(&did_open_diag_txt).await,
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"diagnostics":[{{"data":{{"corrections":["appropriate"]}},"message":"`apropriate` should be `appropriate`","range":{{"end":{{"character":21,"line":0}},"start":{{"character":11,"line":0}}}},"severity":2,"source":"typos"}},{{"data":{{"corrections":["go"]}},"message":"`fo` should be `go`","range":{{"end":{{"character":2,"line":1}},"start":{{"character":0,"line":1}}}},"severity":2,"source":"typos"}}],"uri":"{}/diagnostics.txt","version":1}}}}"#,
            workspace_folder_uri
        ),
    );
}

#[test_log::test(tokio::test)]
async fn test_unicode_diagnostics() {
    let initialize = r#"{
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
      }
    "#;

    let did_open = r#"{
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
          "textDocument": {
            "uri": "file:///diagnostics.txt",
            "languageId": "plaintext",
            "version": 1,
            "text": "¿Qué hace él?"
          }
        }
      }
    "#;

    let mut server = TestServer::new();
    let _ = server.request(&initialize).await;

    // start position should count graphemes with multiple code points as one visible character
    similar_asserts::assert_eq!(
        server.request(&did_open).await,
        r#"{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"diagnostics":[{"data":{"corrections":["have"]},"message":"`hace` should be `have`","range":{"end":{"character":9,"line":0},"start":{"character":5,"line":0}},"severity":2,"source":"typos"}],"uri":"file:///diagnostics.txt","version":1}}"#,
    );
}
