use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower_lsp::{lsp_types::Url, LspService, Server};
use typos_lsp::lsp::*;

#[test_log::test(tokio::test)]
async fn test_initialize_e2e() {
    let req_init =
        req(r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}"#);

    let mut output = Vec::new();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(req_init.as_ref(), &mut output, socket)
        .serve(service)
        .await;

    similar_asserts::assert_eq!(
        body(&output).unwrap(),
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

    let (mut req_client, mut resp_client) = start_server();
    let mut buf = vec![0; 1024];

    req_client
        .write_all(req(initialize).as_bytes())
        .await
        .unwrap();
    let _ = resp_client.read(&mut buf).await.unwrap();

    tracing::debug!("{}", did_open);
    req_client
        .write_all(req(did_open).as_bytes())
        .await
        .unwrap();
    let n = resp_client.read(&mut buf).await.unwrap();

    similar_asserts::assert_eq!(
        body(&buf[..n]).unwrap(),
        r#"{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"diagnostics":[{"data":{"corrections":["appropriate"]},"message":"`apropriate` should be `appropriate`","range":{"end":{"character":21,"line":0},"start":{"character":11,"line":0}},"severity":2,"source":"typos"},{"data":{"corrections":["of","for","do","go","to"]},"message":"`fo` should be `of`, `for`, `do`, `go`, `to`","range":{"end":{"character":2,"line":1},"start":{"character":0,"line":1}},"severity":2,"source":"typos"}],"uri":"file:///diagnostics.txt","version":1}}"#,
    );

    tracing::debug!("{}", code_action);
    req_client
        .write_all(req(code_action).as_bytes())
        .await
        .unwrap();
    let n = resp_client.read(&mut buf).await.unwrap();

    similar_asserts::assert_eq!(
        body(&buf[..n]).unwrap(),
        r#"{"jsonrpc":"2.0","result":[{"diagnostics":[{"data":{"corrections":["of","for"]},"message":"`fo` should be `of`, `for`","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}},"severity":2,"source":"typos"}],"edit":{"changes":{"file:///diagnostics.txt":[{"newText":"of","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}}}]}},"kind":"quickfix","title":"of"},{"diagnostics":[{"data":{"corrections":["of","for"]},"message":"`fo` should be `of`, `for`","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}},"severity":2,"source":"typos"}],"edit":{"changes":{"file:///diagnostics.txt":[{"newText":"for","range":{"end":{"character":7,"line":1},"start":{"character":5,"line":1}}}]}},"kind":"quickfix","title":"for"}],"id":2}"#,
    );

    tracing::debug!("{}", code_action_insertion);
    req_client
        .write_all(req(code_action_insertion).as_bytes())
        .await
        .unwrap();
    let n = resp_client.read(&mut buf).await.unwrap();

    similar_asserts::assert_eq!(
        body(&buf[..n]).unwrap(),
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

    let (mut req_client, mut resp_client) = start_server();
    let mut buf = vec![0; 1024];

    req_client
        .write_all(req(initialize).as_bytes())
        .await
        .unwrap();
    let _ = resp_client.read(&mut buf).await.unwrap();

    // check "fo" is corrected to "of" because of default.extend-words
    tracing::debug!("{}", did_open_diag_txt);
    req_client
        .write_all(req(did_open_diag_txt).as_bytes())
        .await
        .unwrap();
    let n = resp_client.read(&mut buf).await.unwrap();

    similar_asserts::assert_eq!(
        body(&buf[..n]).unwrap(),
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"diagnostics":[{{"data":{{"corrections":["appropriate"]}},"message":"`apropriate` should be `appropriate`","range":{{"end":{{"character":21,"line":0}},"start":{{"character":11,"line":0}}}},"severity":2,"source":"typos"}},{{"data":{{"corrections":["of"]}},"message":"`fo` should be `of`","range":{{"end":{{"character":2,"line":1}},"start":{{"character":0,"line":1}}}},"severity":2,"source":"typos"}}],"uri":"{}/diagnostics.txt","version":1}}}}"#,
            workspace_folder_uri
        ),
    );

    // check changelog is excluded because of files.extend-exclude
    tracing::debug!("{}", did_open_changelog);
    req_client
        .write_all(req(did_open_changelog).as_bytes())
        .await
        .unwrap();
    let n = resp_client.read(&mut buf).await.unwrap();

    similar_asserts::assert_eq!(
        body(&buf[..n]).unwrap(),
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{{"diagnostics":[],"uri":"{}/CHANGELOG.md","version":1}}}}"#,
            workspace_folder_uri
        ),
    );
}

// TODO refactor and extract the boilerplate
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

    let (mut req_client, mut resp_client) = start_server();
    let mut buf = vec![0; 10240];

    req_client
        .write_all(req(initialize).as_bytes())
        .await
        .unwrap();
    let _ = resp_client.read(&mut buf).await.unwrap();

    // check "fo" is corrected to "go" because of default.extend-words
    // in custom_typos.toml which overrides typos.toml
    tracing::debug!("{}", did_open_diag_txt);
    req_client
        .write_all(req(did_open_diag_txt).as_bytes())
        .await
        .unwrap();
    let n = resp_client.read(&mut buf).await.unwrap();

    similar_asserts::assert_eq!(
        body(&buf[..n]).unwrap(),
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

    let (mut req_client, mut resp_client) = start_server();
    let mut buf = vec![0; 1024];

    req_client
        .write_all(req(initialize).as_bytes())
        .await
        .unwrap();
    let _ = resp_client.read(&mut buf).await.unwrap();

    tracing::debug!("{}", did_open);
    req_client
        .write_all(req(did_open).as_bytes())
        .await
        .unwrap();
    let n = resp_client.read(&mut buf).await.unwrap();

    // start position should count graphemes with multiple code points as one visible character
    similar_asserts::assert_eq!(
        body(&buf[..n]).unwrap(),
        r#"{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"diagnostics":[{"data":{"corrections":["have"]},"message":"`hace` should be `have`","range":{"end":{"character":9,"line":0},"start":{"character":5,"line":0}},"severity":2,"source":"typos"}],"uri":"file:///diagnostics.txt","version":1}}"#,
    );
}

fn start_server() -> (tokio::io::DuplexStream, tokio::io::DuplexStream) {
    let (req_client, req_server) = tokio::io::duplex(1024);
    let (resp_server, resp_client) = tokio::io::duplex(1024);

    let (service, socket) = LspService::new(Backend::new);

    // start server as concurrent task
    tokio::spawn(Server::new(req_server, resp_server, socket).serve(service));

    (req_client, resp_client)
}

fn req<T: AsRef<str>>(msg: T) -> String {
    format!(
        "Content-Length: {}\r\n\r\n{}",
        msg.as_ref().len(),
        msg.as_ref()
    )
}

fn body(src: &[u8]) -> Result<&str, anyhow::Error> {
    // parse headers to get headers length
    let mut dst = [httparse::EMPTY_HEADER; 2];

    let (headers_len, _) = match httparse::parse_headers(src, &mut dst)? {
        httparse::Status::Complete(output) => output,
        httparse::Status::Partial => return Err(anyhow::anyhow!("partial headers")),
    };

    // skip headers
    let skipped = &src[headers_len..];

    // return the rest (ie: the body) as &str
    std::str::from_utf8(skipped).map_err(anyhow::Error::from)
}
