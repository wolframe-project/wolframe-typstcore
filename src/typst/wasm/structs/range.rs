use std::{collections::HashMap, ops::Range};

use serde::{Deserialize, Serialize};
use typst::syntax::{FileId, Source, Span};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::{console_log, typst::source_file::SourceFile, typst_error};

use super::error::TypstCoreError;


#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypstCoreRange {
    pub path: String,
    pub monaco_range: MonacoRange,
}

impl TypstCoreRange {
    pub fn with_source(span: Span, source: &Source) -> Result<Self, TypstCoreError> {
        if span.is_detached() {
            Ok(Self {
                path: String::new(),
                monaco_range: MonacoRange::default(),
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
                monaco_range: MonacoRange::from_typst_range(range, source)
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
                monaco_range: MonacoRange::default(),
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

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MonacoRange {
    pub begin_line_number: usize,
    pub begin_column: usize,
    pub end_line_number: usize,
    pub end_column: usize,
}

#[wasm_bindgen]
impl MonacoRange {
    #[wasm_bindgen(constructor)]
    pub fn new(
        begin_line_number: usize,
        begin_column: usize,
        end_line_number: usize,
        end_column: usize,
    ) -> Self {
        MonacoRange {
            begin_line_number,
            begin_column,
            end_line_number,
            end_column,
        }
    }
}

impl MonacoRange {
    pub fn from_typst_range(
        range: Range<usize>,
        source: &Source,
    ) -> Self {
        fn typst_offset_to_monaco_pos(
            source: &Source,
            offset: usize,
        ) -> Option<(usize, usize)> {
            let line = source.byte_to_line(offset)?;
            let column = source.byte_to_column(offset)?;
            Some((line, column))
        }

        let begin = typst_offset_to_monaco_pos(source, range.start);
        let end = typst_offset_to_monaco_pos(source, range.end);
        if let (Some((begin_line_number, begin_column)), Some((end_line_number, end_column))) =
            (begin, end)
        {
            MonacoRange {
                begin_line_number: begin_line_number + 1,
                begin_column: begin_column + 1,
                end_line_number: end_line_number + 1,
                end_column: end_column + 1,
            }
        } else {
            MonacoRange::default()
        }
    }

    pub fn to_typst_range(&self, source: &Source) -> Range<usize> {
        // https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist-analysis/src/location.rs#L87
        fn monaco_pos_to_typst_offset(
            source: &Source,
            line: usize,
            column: usize,
        ) -> Option<usize> {
            let byte_line_offset = source.line_to_byte(line)?;
            let utf16_line_offset = source.byte_to_utf16(byte_line_offset)?;
            let utf16_offset = utf16_line_offset + column;
            let byte_offset = source.utf16_to_byte(utf16_offset)?;
            Some(byte_offset)
        }

        console_log!("Range: {} {} {} {}", self.begin_line_number, self.begin_column, self.end_line_number, self.end_column);

        let begin = monaco_pos_to_typst_offset(source, self.begin_line_number.saturating_sub(1), self.begin_column.saturating_sub(1));
        let end = monaco_pos_to_typst_offset(source, self.end_line_number.saturating_sub(1), self.end_column.saturating_sub(1));
        if let (Some(begin), Some(end)) = (begin, end) {
            begin..end
        } else {
            0..0
        }
    } 
}