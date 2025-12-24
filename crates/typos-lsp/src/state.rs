use anyhow::anyhow;
use matchit::Router;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tower_lsp_server::ls_types::{
    DiagnosticSeverity, Position, TextDocumentContentChangeEvent, Uri, WorkspaceFolder,
};

use crate::typos::Instance;

#[derive(Default)]
pub(crate) struct BackendState<'s> {
    pub severity: Option<DiagnosticSeverity>,
    /// The path to the configuration file given to the LSP server. Settings in this configuration
    /// file override the typos.toml settings.
    pub config: Option<PathBuf>,
    pub workspace_folders: Vec<WorkspaceFolder>,

    /// Maps routes (file system paths) to TyposCli instances, so that we can quickly find the
    /// correct instance for a given file path
    pub router: Router<crate::typos::Instance<'s>>,
    pub documents: HashMap<Uri, Document>,
}

pub(crate) struct Document {
    pub version: i32,
    pub text: String,
}

impl Document {
    pub fn new(version: i32, text: String) -> Self {
        Self { version, text }
    }

    pub fn update(&mut self, version: i32, changes: Vec<TextDocumentContentChangeEvent>) {
        for change in changes {
            if let Some(range) = change.range {
                if let (Some(start), Some(end)) = (
                    self.position_to_offset(range.start),
                    self.position_to_offset(range.end),
                ) {
                    self.text.replace_range(start..end, &change.text);
                } else {
                    tracing::warn!("Invalid range in document update: {:?}", range);
                }
            } else {
                self.text = change.text;
            }
        }
        self.version = version;
    }

    fn position_to_offset(&self, position: Position) -> Option<usize> {
        // translates an LSP Position (0-indexed line and UTF-16 character offset) into a byte offset in the document string.
        // the reverse operation of AccumulatePosition::pos
        let mut offset = 0;
        // Split inclusive ensures we keep newlines, which counts towards offset.
        let mut lines = self.text.split_inclusive('\n');

        for _ in 0..position.line {
            let line = lines.next()?; // if we go out of bounds exit position_to_offset returning None (shouldn't happen)
            offset += line.len();
        }

        // Now we are on the correct line.
        let line = match lines.next() {
            Some(l) => l,
            // We are past the last line
            None => {
                return if position.character == 0 {
                    // Insertion point on new line after end of document (EOF)
                    Some(offset)
                } else {
                    // Invalid
                    None
                };
            }
        };

        // Find char offset
        let mut utf16_pos = 0;
        // iterate over UTF-8 chars in the line
        // i = char index, ie: byte offset
        for (i, c) in line.char_indices() {
            if utf16_pos == position.character {
                return Some(offset + i);
            }
            utf16_pos += c.len_utf16() as u32;
            if utf16_pos > position.character {
                return None;
            }
        }

        // Check if at the end of the line
        if utf16_pos == position.character {
            Some(offset + line.len())
        } else {
            None
        }
    }
}

impl<'s> BackendState<'s> {
    pub(crate) fn set_workspace_folders(
        &mut self,
        workspace_folders: Vec<WorkspaceFolder>,
    ) -> anyhow::Result<(), anyhow::Error> {
        self.workspace_folders = workspace_folders;
        self.update_router()?;
        Ok(())
    }

    pub(crate) fn update_workspace_folders(
        &mut self,
        added: Vec<WorkspaceFolder>,
        removed: Vec<WorkspaceFolder>,
    ) -> anyhow::Result<(), anyhow::Error> {
        self.workspace_folders.extend(added);
        if !removed.is_empty() {
            self.workspace_folders.retain(|x| !removed.contains(x));
        }
        self.update_router()?;
        Ok(())
    }

    pub(crate) fn update_router(&mut self) -> anyhow::Result<(), anyhow::Error> {
        self.router = Router::new();
        for folder in self.workspace_folders.iter() {
            let path = folder
                .uri
                .to_file_path()
                .ok_or_else(|| anyhow!("Cannot convert uri {:?} to file path", folder.uri))?;
            let route = format!("{}{}", uri_path_sanitised(&folder.uri), "/{*p}");
            self.router
                .insert_instance(&route, &path, self.config.as_deref())?;
        }

        // add low priority catch all route used for files outside the workspace, or
        // when there is no workspace folder
        #[cfg(windows)]
        for drive in crate::windows::get_drives() {
            let route = format!("/{}%3A/{{*p}}", &drive);
            self.router.insert_instance(
                &route,
                &PathBuf::from(format!("{}:\\", &drive)),
                self.config.as_deref(),
            )?;
        }

        #[cfg(not(windows))]
        {
            let route = "/{*p}";
            self.router
                .insert_instance(route, &PathBuf::from("/"), self.config.as_deref())?;
        }

        Ok(())
    }
}

trait RouterExt {
    fn insert_instance(
        &mut self,
        route: &str,
        path: &Path,
        config: Option<&Path>,
    ) -> anyhow::Result<(), anyhow::Error>;
}

impl RouterExt for Router<Instance<'_>> {
    // convenience method to insert a new TyposCli into the router
    // implemented as an extension trait to avoid interprocedural conflicts
    fn insert_instance(
        &mut self,
        route: &str,
        path: &Path,
        config: Option<&Path>,
    ) -> anyhow::Result<(), anyhow::Error> {
        tracing::debug!("Adding route {} for path {}", route, path.display());
        let instance = Instance::new(path, config)?;
        self.insert(route, instance)?;
        Ok(())
    }
}

pub fn uri_path_sanitised(uri: &Uri) -> String {
    // windows paths (eg: /C:/Users/..) may not be percent-encoded by some clients
    // and therefore contain colons, see
    // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#uri
    //
    // and because matchit treats colons as a wildcard we need to strip them
    uri.path().to_string().replace(':', "%3A")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp_server::ls_types::Range;

    #[test]
    fn test_update_document_full() {
        let mut doc = Document::new(0, "".to_string());
        let version = 1;
        let text = "hello world";

        let changes = vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: text.to_string(),
        }];

        doc.update(version, changes);

        assert_eq!(doc.text, "hello world");
        assert_eq!(doc.version, 1);
    }

    #[test]
    fn test_update_document_incremental_insert() {
        let mut doc = Document::new(1, "hello world".to_string());

        // Insert "!" at the end
        let changes = vec![TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 0,
                    character: 11,
                },
                end: Position {
                    line: 0,
                    character: 11,
                },
            }),
            range_length: None,
            text: "!".to_string(),
        }];

        doc.update(2, changes);

        assert_eq!(doc.text, "hello world!");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_update_document_incremental_delete() {
        let mut doc = Document::new(1, "hello world".to_string());

        // Delete "world"
        let changes = vec![TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 0,
                    character: 6,
                },
                end: Position {
                    line: 0,
                    character: 11,
                },
            }),
            range_length: None,
            text: "".to_string(),
        }];

        doc.update(2, changes);

        assert_eq!(doc.text, "hello ");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_update_document_incremental_replace() {
        let mut doc = Document::new(1, "hello world".to_string());

        // Replace "world" with "there"
        let changes = vec![TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 0,
                    character: 6,
                },
                end: Position {
                    line: 0,
                    character: 11,
                },
            }),
            range_length: None,
            text: "there".to_string(),
        }];

        doc.update(2, changes);

        assert_eq!(doc.text, "hello there");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_update_document_multiline() {
        let mut doc = Document::new(1, "line 1\nline 2\nline 3".to_string());

        // Replace "line 2" with "line two"
        let changes = vec![TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 1,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 6,
                },
            }),
            range_length: None,
            text: "line two".to_string(),
        }];

        doc.update(2, changes);

        assert_eq!(doc.text, "line 1\nline two\nline 3");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_position_to_offset_complex() {
        // '𐐀' is 4 bytes (0xF0 0x90 0x90 0x80), 2 UTF-16 units.
        let text = "a𐐀b\r\nc";
        let doc = Document::new(1, text.to_string());

        // Line 0: "a𐐀b\r\n"
        // 'a': utf16=0, byte=0
        assert_eq!(
            doc.position_to_offset(Position {
                line: 0,
                character: 0
            }),
            Some(0)
        );
        // '𐐀': utf16=1, byte=1. Takes 2 units.
        assert_eq!(
            doc.position_to_offset(Position {
                line: 0,
                character: 1
            }),
            Some(1)
        );
        // Invalid middle of surrogate pair
        assert_eq!(
            doc.position_to_offset(Position {
                line: 0,
                character: 2
            }),
            None
        );
        // 'b': utf16=3, byte=1+4=5
        assert_eq!(
            doc.position_to_offset(Position {
                line: 0,
                character: 3
            }),
            Some(5)
        );
        // '\r': utf16=4, byte=6
        assert_eq!(
            doc.position_to_offset(Position {
                line: 0,
                character: 4
            }),
            Some(6)
        );
        // '\n': utf16=5, byte=7
        assert_eq!(
            doc.position_to_offset(Position {
                line: 0,
                character: 5
            }),
            Some(7)
        );
        // End of line 0 after \n before next line start,
        // effectively same as start of next line
        assert_eq!(
            doc.position_to_offset(Position {
                line: 0,
                character: 6
            }),
            Some(8)
        );
        // Out of bounds line 0
        assert_eq!(
            doc.position_to_offset(Position {
                line: 0,
                character: 7
            }),
            None
        );

        // Line 1: "c"
        // Start of line 1
        assert_eq!(
            doc.position_to_offset(Position {
                line: 1,
                character: 0
            }),
            Some(8)
        );
        // After 'c', end of line insertion point
        assert_eq!(
            doc.position_to_offset(Position {
                line: 1,
                character: 1
            }),
            Some(9)
        );
        // Out of bounds line 1
        assert_eq!(
            doc.position_to_offset(Position {
                line: 1,
                character: 2
            }),
            None
        );

        // Line 2: (Does not exist, it's EOF)
        // Valid EOF insertion point
        assert_eq!(
            doc.position_to_offset(Position {
                line: 2,
                character: 0
            }),
            Some(9)
        );
        // Invalid char on non-existent line
        assert_eq!(
            doc.position_to_offset(Position {
                line: 2,
                character: 1
            }),
            None
        );
    }
}
