use std::sync::OnceLock;

use typst::{foundations::Bytes, syntax::{FileId, Source}};

#[derive(Clone)]
pub struct SourceFile {
    bytes: OnceLock<Bytes>,
    pub source: Source,
}

impl SourceFile {
    pub fn new(id: FileId, text: String) -> Self {
        Self {
            bytes: OnceLock::new(),
            source: Source::new(id, text),
        }
    }

    pub fn source(&self) -> Source {
        self.source.clone()
    }

    pub fn bytes(&self) -> Bytes {
        self.bytes
            .get_or_init(|| Bytes::from_string(self.source.text().to_string()))
            .clone()
    }
}