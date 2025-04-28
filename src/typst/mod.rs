use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use chrono::{DateTime, Datelike, Local, Timelike};
use parking_lot::{Mutex, RwLock};
use source_file::SourceFile;
use typst::{
    diag::FileResult,
    foundations::{Bytes, Datetime},
    layout::PagedDocument,
    syntax::{FileId, Source},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, World,
};
use wasm_bindgen::prelude::wasm_bindgen;

mod source_file;
mod tidy;

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

impl World for TypstCore {
    #[doc = " The standard library."]
    #[doc = ""]
    #[doc = " Can be created through `Library::build()`."]
    fn library(&self) -> &LazyHash<Library> {
        todo!()
    }

    #[doc = " Metadata about all known fonts."]
    fn book(&self) -> &LazyHash<FontBook> {
        todo!()
    }

    #[doc = " Get the file id of the main source file."]
    fn main(&self) -> FileId {
        todo!()
    }

    #[doc = " Try to access the specified source file."]
    fn source(&self, id: FileId) -> FileResult<Source> {
        todo!()
    }

    #[doc = " Try to access the specified file."]
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        todo!()
    }

    #[doc = " Try to access the font with the given index in the font book."]
    fn font(&self, index: usize) -> Option<Font> {
        todo!()
    }

    #[doc = " Get the current date."]
    #[doc = ""]
    #[doc = " If no offset is specified, the local date should be chosen. Otherwise,"]
    #[doc = " the UTC date should be chosen with the corresponding offset in hours."]
    #[doc = ""]
    #[doc = " If this function returns `None`, Typst\'s `datetime` function will"]
    #[doc = " return an error."]
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let dt = if let Some(offset) = offset {
            self.now.get_or_init(|| {
                let now = Local::now();
                let offset = chrono::Duration::hours(offset);
                now + offset
            })
        } else {
            self.now.get_or_init(Local::now)
        };

        Datetime::from_ymd_hms(
            dt.year(),
            dt.month() as u8,
            dt.day() as u8,
            dt.hour() as u8,
            dt.minute() as u8,
            dt.second() as u8
        )
    }
}
