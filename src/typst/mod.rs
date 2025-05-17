use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, OnceLock},
};

use atomic_refcell::AtomicRefCell;
use chrono::{DateTime, Local};
use source_file::SourceFile;
use typst::{
    layout::PagedDocument,
    syntax::FileId,
    text::{Font, FontBook},
    utils::LazyHash,
    Library,
};
use wasm::structs::package::TypstCorePackage;
use wasm_bindgen::prelude::wasm_bindgen;

mod source_file;
mod tidy;
mod typst_core;
pub mod wasm;
mod world;

#[wasm_bindgen]
pub struct TypstCore {
    library: OnceLock<LazyHash<Library>>,

    book: OnceLock<LazyHash<FontBook>>,

    sources: Arc<AtomicRefCell<HashMap<FileId, SourceFile>>>,

    fonts: Mutex<Vec<Font>>,

    root: Option<FileId>,

    now: OnceLock<DateTime<Local>>,

    last_doc: Mutex<Option<PagedDocument>>,

    packages: Mutex<HashSet<TypstCorePackage>>,
}
