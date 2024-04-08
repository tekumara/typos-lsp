use std::path::Path;

use bstr::ByteSlice;
use ignore::overrides::{Override, OverrideBuilder};
use typos_cli::policy;
pub struct Instance<'s> {
    pub ignores: Override,
    pub engine: policy::ConfigEngine<'s>,
}

impl Instance<'_> {
    pub fn new<'s>(
        path: &Path,
        config: Option<&Path>,
    ) -> anyhow::Result<Instance<'s>, anyhow::Error> {
        // leak to get a 'static which is needed to satisfy the 's lifetime
        // but does mean memory will grow unbounded
        let storage = Box::leak(Box::new(policy::ConfigStorage::new()));
        let mut engine = typos_cli::policy::ConfigEngine::new(storage);

        // TODO: currently mimicking typos here but do we need to create and update
        // a default config?
        let mut c = typos_cli::config::Config::default();
        if let Some(config_path) = config {
            let custom = typos_cli::config::Config::from_file(config_path)?;
            if let Some(custom) = custom {
                c.update(&custom);
                engine.set_overrides(c);
            }
        }

        // initialise an engine and overrides using the config file from path or its parent
        engine.init_dir(path)?;
        let walk_policy = engine.walk(path);

        let mut ignores = OverrideBuilder::new(path);
        // always ignore the config files like typos cli does
        for f in typos_cli::config::SUPPORTED_FILE_NAMES {
            ignores.add(&format!("!{}", f))?;
        }

        // add any explicit excludes
        for pattern in walk_policy.extend_exclude.iter() {
            ignores.add(&format!("!{}", pattern))?;
        }
        let ignore = ignores.build()?;

        Ok(Instance {
            ignores: ignore,
            engine,
        })
    }
}

// copied from https://github.com/crate-ci/typos/blob/c15b28fff9a814f9c12bd24cb1cfc114037e9187/crates/typos-cli/src/file.rs#L741
#[derive(Clone, Debug)]
pub(crate) struct Ignores {
    blocks: Vec<std::ops::Range<usize>>,
}

impl Ignores {
    pub(crate) fn new(content: &[u8], ignores: &[regex::Regex]) -> Self {
        let mut blocks = Vec::new();
        if let Ok(content) = std::str::from_utf8(content) {
            for ignore in ignores {
                for mat in ignore.find_iter(content) {
                    blocks.push(mat.range());
                }
            }
        }
        Self { blocks }
    }

    pub(crate) fn is_ignored(&self, span: std::ops::Range<usize>) -> bool {
        let start = span.start;
        let end = span.end.saturating_sub(1);
        self.blocks
            .iter()
            .any(|block| block.contains(&start) || block.contains(&end))
    }
}

pub struct AccumulatePosition {
    line_num: usize,
    line_pos: usize,
    last_offset: usize,
}

impl AccumulatePosition {
    pub fn new() -> Self {
        Self {
            // LSP ranges are 0-indexed see https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#range
            line_num: 0,
            line_pos: 0,
            last_offset: 0,
        }
    }

    pub fn pos(&mut self, buffer: &[u8], byte_offset: usize) -> (usize, usize) {
        assert!(self.last_offset <= byte_offset);
        let slice = &buffer[self.last_offset..byte_offset];
        let newlines = slice.find_iter(b"\n").count();
        let line_num = self.line_num + newlines;

        let line_start = buffer[0..byte_offset]
            .rfind_byte(b'\n')
            // Skip the newline
            .map(|s| s + 1)
            .unwrap_or(0);

        let before_typo = String::from_utf8_lossy(&buffer[line_start..byte_offset]);

        // count UTF-16 code units as per
        // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocuments
        // UTF-16 is the only position encoding we support for now
        let line_pos = before_typo.chars().map(char::len_utf16).sum();

        self.line_num = line_num;
        self.line_pos = line_pos;
        self.last_offset = byte_offset;

        (self.line_num, self.line_pos)
    }
}
