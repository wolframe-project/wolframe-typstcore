use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, OnceLock},
};

use atomic_refcell::AtomicRefCell;
use typst::{
    foundations::Bytes, html::HtmlDocument, layout::PagedDocument, syntax::{FileId, VirtualPath}, text::{Font, FontBook}, utils::LazyHash
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::{
    console_log, typst::{source_file::SourceFile, wasm::structs::{diagnostics::TypstCoreDiagnostics, range::{MonacoPosition, MonacoRange}}, TypstCore}, typst_error
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

            sources: Arc::new(AtomicRefCell::new(HashMap::new())),

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
                                let diag = match TypstCoreDiagnostics::from_diagnostics(err, &self.sources.borrow()) {
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
                            let diag = TypstCoreDiagnostics::from_diagnostics(err, &self.sources.borrow())?;
                            diagnostics.push(diag);
                        }
                        Err(TypstCoreError::CompileError(diagnostics))
                    }
                }
            }
            _ => {
                match typst::compile::<PagedDocument>(self).output {
                    Ok(doc) => {
                        *self.last_doc.lock().unwrap() = Some(doc.clone());
        
                        Ok(Output::Svg(doc.pages.iter().map(typst_svg::svg).collect()))
                    }
                    Err(error) => {
                        let mut diagnostics = Vec::new();
                        for err in error {
                            let diag = TypstCoreDiagnostics::from_diagnostics(err, &self.sources.borrow())?;
                            diagnostics.push(diag);
                        }
                        Err(TypstCoreError::CompileError(diagnostics))
                    }
                }
            }
        }
    }

    pub fn set_root(&mut self, path: String) -> Result<(), TypstCoreError> {
        let sources = self.sources.borrow();
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
        self.sources.borrow_mut().insert(id, source);
    }

    pub fn remove_source(&mut self, path: String) {
        let id = FileId::new(None, VirtualPath::new(&path));
        self.sources.borrow_mut().remove(&id);
    }

    pub fn edit_source(
        &mut self,
        path: String,
        content: String,
        monaco_range: MonacoRange,
    ) -> Result<(), TypstCoreError> {
        let id = FileId::new(None, VirtualPath::new(&path));
        let mut sources = self.sources.borrow_mut();
        if let Some(source) = sources.get_mut(&id) {
            let typst_range = monaco_range.to_typst_range(&source.source);

            let range = source.source.edit(typst_range, &content);
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
        let sources = self.sources.borrow();
        if let Some(source) = sources.get(&id) {
            Ok(source.source.text().to_string())
        } else {
            Err(typst_error!(format!(
                "Failed to get source, source not found for path: {:?}",
                path
            )))
        }
    }

    pub fn auto_complete(
        &self,
        path: String,
        line: usize,
        column: usize,
    ) -> Result<Vec<JsValue>, TypstCoreError> {
        let id = FileId::new(None, VirtualPath::new(&path));
        let sources = self.sources.borrow();
        if let Some(source) = sources.get(&id) {
            let typst_position = MonacoPosition::new(line, column).to_typst_position(&source.source);
            if let Some(typst_position) = typst_position {
                let doc = self.last_doc.lock().unwrap();
                
                match typst_ide::autocomplete(self, doc.as_ref(), &source.source, typst_position, true) {
                    Some(completions) => {
                        let mut result = Vec::new();
                        for completion in completions.1 {
                            if let Ok(completion) = serde_wasm_bindgen::to_value(&completion) {
                                result.push(completion);
                            }
                        }
                        Ok(result)
                    }
                    None => Ok(Vec::new()),
                }
            } else {
                Err(typst_error!(format!(
                    "Failed to convert Monaco position to Typst position for path: {:?}",
                    path
                )))
            }
        } else {
            Err(typst_error!(format!(
                "Failed to get source, source not found for path: {:?}",
                path
            )))
        }
    }
}
