use chrono::{Datelike, Local, Timelike};
use typst::{
    diag::FileResult,
    foundations::{Bytes, Datetime},
    syntax::{FileId, Source},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, World,
};

use super::TypstCore;

impl World for TypstCore {
    #[doc = " The standard library."]
    #[doc = ""]
    #[doc = " Can be created through `Library::build()`."]
    fn library(&self) -> &LazyHash<Library> {
        self.library
            .get_or_init(|| LazyHash::new(Library::builder().build()))
    }

    #[doc = " Metadata about all known fonts."]
    fn book(&self) -> &LazyHash<FontBook> {
        self.book
            .get_or_init(|| LazyHash::new(FontBook::from_fonts(self.fonts.lock().as_slice())))
    }

    #[doc = " Get the file id of the main source file."]
    fn main(&self) -> FileId {
        *self.root.as_ref().expect("Root path is not set")
    }

    #[doc = " Try to access the specified source file."]
    fn source(&self, id: FileId) -> FileResult<Source> {
        let sources = self.sources.read();
        if let Some(source) = sources.get(&id) {
            Ok(source.source())
        } else {
            Err(typst::diag::FileError::NotFound(
                id.vpath().as_rooted_path().to_path_buf(),
            ))
        }
    }

    #[doc = " Try to access the specified file."]
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let sources = self.sources.read();
        if let Some(source) = sources.get(&id) {
            Ok(source.bytes())
        } else {
            Err(typst::diag::FileError::NotFound(
                id.vpath().as_rooted_path().to_path_buf(),
            ))
        }
    }

    #[doc = " Try to access the font with the given index in the font book."]
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.lock().get(index).cloned()
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
            dt.second() as u8,
        )
    }
}
