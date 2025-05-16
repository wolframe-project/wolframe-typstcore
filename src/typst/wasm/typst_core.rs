use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, OnceLock},
};

use parking_lot::{Mutex, RwLock};
use typst::{
    foundations::Bytes, html::HtmlDocument, layout::PagedDocument, syntax::{FileId, VirtualPath}, text::{Font, FontBook}, utils::LazyHash
};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    console_log, typst::{source_file::SourceFile, wasm::structs::diagnostics::TypstCoreDiagnostics, TypstCore}, typst_error
};

use super::structs::{error::TypstCoreError, output::{Output, OutputFormat}};

fn gather_internal_fonts() -> Vec<Font> {
    let mut fonts = Vec::new();

    for font in typst_assets::fonts() {
        let buf = Bytes::new(font);
        for font in Font::iter(buf) {
            fonts.push(font);
        }
    }

    fonts
}


// https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist-analysis/src/location.rs#L87
fn monaco_pos_to_typst_offset(
    source: &SourceFile,
    line: usize,
    column: usize,
) -> Option<usize> {
    let byte_line_offset = source.source.line_to_byte(line)?;
    let utf16_line_offset = source.source.byte_to_utf16(byte_line_offset)?;
    let utf16_offset = utf16_line_offset + column;
    let byte_offset = source.source.utf16_to_byte(utf16_offset)?;
    Some(byte_offset)
}

#[wasm_bindgen]
impl TypstCore {
    #[wasm_bindgen(constructor)]
    pub fn construct() -> Self {
        console_error_panic_hook::set_once();
        let fonts = gather_internal_fonts();

        Self {
            library: OnceLock::default(),

            book: OnceLock::from(LazyHash::new(FontBook::from_fonts(&fonts))),

            sources: Arc::new(RwLock::new(HashMap::new())),

            fonts: Mutex::new(fonts),

            root: None,

            last_doc: Mutex::new(None),

            now: OnceLock::new(),

            packages: Mutex::new(HashSet::new()),
        }
    }

    pub fn compile(&self, format: OutputFormat) -> Result<Output, TypstCoreError> {
        if self.root.is_none() {
            return Err(typst_error!("Root path is not set"));
        }

        match format {
            OutputFormat::Html => {
                match typst::compile::<HtmlDocument>(self).output {
                    Ok(doc) => {
                        // is not a paged document, so we don't need to store it
                        let html = typst_html::html(&doc).map_err(|e| {
                            let mut diagnostics = Vec::new();
                            for err in e {
                                let diag = match TypstCoreDiagnostics::from_diagnostics(err, &self.sources.read()) {
                                    Ok(diag) => diag,
                                    Err(e) => {
                                        return e;
                                    }
                                };
                                diagnostics.push(diag);
                            }
                            TypstCoreError::CompileError(diagnostics)
                        })?;

                        Ok(Output::Html(html))
                    }
                    Err(error) => {
                        let mut diagnostics = Vec::new();
                        for err in error {
                            let diag = TypstCoreDiagnostics::from_diagnostics(err, &self.sources.read())?;
                            diagnostics.push(diag);
                        }
                        Err(TypstCoreError::CompileError(diagnostics))
                    }
                }
            }
            _ => {
                match typst::compile::<PagedDocument>(self).output {
                    Ok(doc) => {
                        *self.last_doc.lock() = Some(doc.clone());
        
                        Ok(Output::Svg(doc.pages.iter().map(typst_svg::svg).collect()))
                    }
                    Err(error) => {
                        let mut diagnostics = Vec::new();
                        for err in error {
                            let diag = TypstCoreDiagnostics::from_diagnostics(err, &self.sources.read())?;
                            diagnostics.push(diag);
                        }
                        Err(TypstCoreError::CompileError(diagnostics))
                    }
                }
            }
        }
    }

    pub fn set_root(&mut self, path: String) -> Result<(), TypstCoreError> {
        let sources = self.sources.read();
        let id = FileId::new(None, VirtualPath::new(&path));
        if sources.contains_key(&id) {
            self.root = Some(id);
            Ok(())
        } else {
            Err(typst_error!(format!(
                "Failed to set root, source not found for path: {:?}",
                path
            )))
        }
    }

    pub fn add_source(&mut self, path: String, content: String) {
        let id = FileId::new(None, VirtualPath::new(&path));
        let source = SourceFile::new(id, content);
        self.sources.write().insert(id, source);
    }

    pub fn remove_source(&mut self, path: String) {
        let id = FileId::new(None, VirtualPath::new(&path));
        self.sources.write().remove(&id);
    }

    pub fn edit_source(
        &mut self,
        path: String,
        content: String,
        begin: usize,
        begin_line: usize,
        begin_column: usize,
        end: usize,
        end_line: usize,
        end_column: usize,
    ) -> Result<(), TypstCoreError> {
        let id = FileId::new(None, VirtualPath::new(&path));
        let mut sources = self.sources.write();
        if let Some(source) = sources.get_mut(&id) {
            let utf16_begin = monaco_pos_to_typst_offset(source, begin_line, begin_column);
            let utf16_end = monaco_pos_to_typst_offset(source, end_line, end_column);


            let range = source.source.edit(utf16_begin.unwrap()..utf16_end.unwrap(), &content);
            console_log!("Edited range: {:?}; New text: {:?}", range, source.source.text());
            Ok(())
        } else {
            Err(typst_error!(format!(
                "Failed to edit source, source not found for path: {:?}",
                path
            )))
        }
    }

    pub fn get_source(&self, path: String) -> Result<String, TypstCoreError> {
        let id = FileId::new(None, VirtualPath::new(&path));
        let sources = self.sources.read();
        if let Some(source) = sources.get(&id) {
            Ok(source.source.text().to_string())
        } else {
            Err(typst_error!(format!(
                "Failed to get source, source not found for path: {:?}",
                path
            )))
        }
    }
}
