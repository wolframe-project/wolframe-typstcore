use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

use flate2::read::GzDecoder;
use typst::{
    diag::{FileError, FileResult},
    syntax::{package::PackageSpec, FileId, VirtualPath},
};

use crate::{console_log, utils::fetch};

use super::{source_file::SourceFile, TypstCore};

/// Extracts the path from a tar entry, handling potential errors.
fn extract_entry_path(entry: &tar::Entry<GzDecoder<Cursor<Vec<u8>>>>) -> FileResult<String> {
    Ok(entry
        .path()
        .map_err(|e| {
            FileError::Other(Some(
                format!("Failed to read package entry path: {}", e).into(),
            ))
        })?
        .to_string_lossy()
        .into_owned())
}

/// Reads the contents of a tar entry into a String, handling potential errors.
fn read_entry_contents(entry: &mut tar::Entry<GzDecoder<Cursor<Vec<u8>>>>) -> FileResult<String> {
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes).map_err(|e| {
        FileError::Other(Some(
            format!("Failed to read package entry bytes: {}", e).into(),
        ))
    })?;

    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// Fetches a Typst package from the official repository and extracts its source files.
///
/// This function:
/// 1. Downloads the package tarball from packages.typst.org
/// 2. Extracts all files from the archive
/// 3. Loads the content as SourceFile objects
///
/// Returns a map of FileId to SourceFile for all files in the package.
fn fetch_package(package: &PackageSpec) -> FileResult<HashMap<FileId, SourceFile>> {
    // Build the URL for the package tarball
    let url = format!(
        "https://packages.typst.org/preview/{}-{}.tar.gz",
        package.name, package.version
    );

    // Download and prepare the archive for extraction
    let response = fetch(&url);
    let cursor = Cursor::new(response);
    let gz_decoder = GzDecoder::new(cursor);
    let mut archive = tar::Archive::new(gz_decoder);

    // Get all entries from the archive, handling potential errors
    let entries = archive.entries().map_err(|e| {
        FileError::Other(Some(
            format!("Failed to read package archive: {}", e).into(),
        ))
    })?;

    // Process each file in the archive
    let mut sources = HashMap::new();
    for entry_result in entries.filter(|e| {
        // Only process actual files (not directories)
        e.is_ok() && e.as_ref().unwrap().header().entry_type().is_file()
    }) {
        // Extract file metadata and contents
        let mut entry = entry_result.map_err(|e| {
            FileError::Other(Some(format!("Failed to read package entry: {}", e).into()))
        })?;

        // Get the path within the archive
        let path = extract_entry_path(&entry)?;

        // Create a file identifier for this package file
        let file_id = FileId::new(Some(package.clone()), VirtualPath::new(path));

        // Read the file contents
        let text = read_entry_contents(&mut entry)?;

        // Create and store the source file
        let source_file = SourceFile::new(file_id, text);
        sources.insert(file_id, source_file);
    }

    Ok(sources)
}

impl TypstCore {
    pub fn retrieve_source(&self, id: FileId) -> FileResult<SourceFile> {
        match id.package() {
            Some(package) => {
                console_log!("Retrieving source for id: {:?}, {:?}", id, package);
                if self.packages.lock().unwrap().contains(&(package.clone().into())) {
                    let sources = self.sources.borrow();
                    if let Some(source) = sources.get(&id) {
                        Ok(source.clone())
                    } else {
                        Err(typst::diag::FileError::NotFound(
                            id.vpath().as_rooted_path().to_path_buf(),
                        ))
                    }
                } else {
                    let fetched_sources = fetch_package(package)?;
                    {
                        let mut sources = self.sources.borrow_mut();
                        for (id, source) in fetched_sources {
                            sources.insert(id, source.clone());
                        }
                        self.packages.lock().unwrap().insert(package.clone().into());
                    }
                    if let Some(source) = self.sources.borrow().get(&id) {
                        Ok(source.clone())
                    } else {
                        Err(typst::diag::FileError::NotFound(
                            id.vpath().as_rooted_path().to_path_buf(),
                        ))
                    }
                }
            }
            None => {
                let sources = self.sources.borrow();
                if let Some(source) = sources.get(&id) {
                    Ok(source.clone())
                } else {
                    Err(typst::diag::FileError::NotFound(
                        id.vpath().as_rooted_path().to_path_buf(),
                    ))
                }
            }
        }
    }
}
