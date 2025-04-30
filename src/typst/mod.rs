use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use chrono::{DateTime, Local};
use parking_lot::{Mutex, RwLock};
use source_file::SourceFile;
use typst::{
    layout::PagedDocument,
    syntax::FileId,
    text::{Font, FontBook},
    utils::LazyHash,
    Library,
};
use wasm_bindgen::prelude::wasm_bindgen;

mod source_file;
mod tidy;
mod world;
mod wasm;

#[wasm_bindgen]
pub struct TypstCore {
    library: OnceLock<LazyHash<Library>>,

    book: OnceLock<LazyHash<FontBook>>,

    sources: Arc<RwLock<HashMap<FileId, SourceFile>>>,

    fonts: Mutex<Vec<Font>>,

    root: PathBuf,

    now: OnceLock<DateTime<Local>>,

    last_doc: Mutex<Option<PagedDocument>>,
}