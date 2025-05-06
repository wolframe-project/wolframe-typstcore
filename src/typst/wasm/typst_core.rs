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
    typst::{source_file::SourceFile, wasm::structs::diagnostics::TypstCoreDiagnostics, TypstCore},
    typst_error,
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
        end: usize,
    ) -> Result<(), TypstCoreError> {
        let id = FileId::new(None, VirtualPath::new(&path));
        let mut sources = self.sources.write();
        if let Some(source) = sources.get_mut(&id) {
            source.source.edit(begin..end, &content);
            Ok(())
        } else {
            Err(typst_error!(format!(
                "Failed to edit source, source not found for path: {:?}",
                path
            )))
        }
    }
}
