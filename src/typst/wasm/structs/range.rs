use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typst::syntax::{FileId, Source, Span};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{typst::source_file::SourceFile, typst_error};

use super::error::TypstCoreError;


#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypstCoreRange {
    pub path: String,
    pub start: usize,
    pub end: usize,
}

impl TypstCoreRange {
    pub fn with_source(span: Span, source: &Source) -> Result<Self, TypstCoreError> {
        if span.is_detached() {
            Ok(Self {
                path: String::new(),
                start: 0,
                end: 0,
            })
        } else {
            let range = source
                .range(span)
                .ok_or_else(|| typst_error!(format!("Failed to get range for span: {:?}", span)))?;

            Ok(Self {
                path: source
                    .id()
                    .vpath()
                    .as_rooted_path()
                    .to_string_lossy()
                    .to_string(),
                start: source.byte_to_utf16(range.start).unwrap(),
                end: source.byte_to_utf16(range.end).unwrap(),
            })
        }
    }

    pub fn with_sources(
        span: Span,
        sources: &HashMap<FileId, SourceFile>,
    ) -> Result<Self, TypstCoreError> {
        if span.is_detached() {
            Ok(Self {
                path: String::new(),
                start: 0,
                end: 0,
            })
        } else {
            let id = span
                .id()
                .ok_or_else(|| typst_error!(format!("Failed to get id for span: {:?}", span)))?;

            let source_file = sources
                .get(&id)
                .ok_or_else(|| typst_error!(format!("Failed to get source for id: {:?}", id)))?;

            let source = source_file.source();

            TypstCoreRange::with_source(span, &source)
        }
    }
}