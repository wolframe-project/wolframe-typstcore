use serde::{Deserialize, Serialize};
use tsify::Tsify;
use typst::diag::FileError;
use wasm_bindgen::{convert::{FromWasmAbi, IntoWasmAbi}, JsValue};

use super::diagnostics::TypstCoreDiagnostics;

#[macro_export]
macro_rules! typst_error {
    ($message:expr) => {
        TypstCoreError::DefaultError(format!(
            "{} (at {}:{}:{})",
            $message,
            file!(),
            line!(),
            column!()
        ))
    };
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
pub enum TypstCoreError {
    CompileError(Vec<TypstCoreDiagnostics>),
    DefaultError(String),
}

impl From<FileError> for TypstCoreError {
    fn from(value: FileError) -> Self {
        match value {
            FileError::NotFound(path_buf) => TypstCoreError::DefaultError(format!(
                "File not found: {}",
                path_buf.display()
            )),
            FileError::AccessDenied => TypstCoreError::DefaultError("FileError: Access denied".to_string()),
            FileError::IsDirectory => TypstCoreError::DefaultError("FileError: File a directory".to_string()),
            FileError::NotSource => TypstCoreError::DefaultError("FileError: Not a source file".to_string()),
            FileError::InvalidUtf8 => TypstCoreError::DefaultError("FileError: Content is invalid UTF-8".to_string()),
            FileError::Package(package_error) => {
                match package_error {
                    typst::diag::PackageError::NotFound(package_spec) => 
                        TypstCoreError::DefaultError(format!(
                            "FileError: Package not found: {}",
                            package_spec
                        )),
                    typst::diag::PackageError::VersionNotFound(package_spec, package_version) => 
                        TypstCoreError::DefaultError(format!(
                            "FileError: Package version not found: {}@{}",
                            package_spec, package_version
                        )),
                    typst::diag::PackageError::NetworkFailed(eco_string) => 
                        TypstCoreError::DefaultError(format!(
                            "FileError: Network failed: {:?}",
                            eco_string
                        )),
                    typst::diag::PackageError::MalformedArchive(eco_string) => 
                        TypstCoreError::DefaultError(format!(
                            "FileError: Malformed archive: {:?}",
                            eco_string
                        )),
                    typst::diag::PackageError::Other(eco_string) => 
                        TypstCoreError::DefaultError(format!(
                            "FileError: Other package error: {:?}",
                            eco_string
                        ))
                }
            },
            FileError::Other(eco_string) => TypstCoreError::DefaultError(format!(
                "FileError: Other error: {:?}",
                eco_string
            )),
        }
    }
}


/// Wasm Bindgen

impl From<TypstCoreError> for JsValue {
    fn from(val: TypstCoreError) -> Self {
        serde_wasm_bindgen::to_value(&val).unwrap()
    }
}

impl wasm_bindgen::describe::WasmDescribe for TypstCoreError {
    fn describe() {
        JsValue::describe()
    }
}

impl wasm_bindgen::convert::IntoWasmAbi for TypstCoreError {
    type Abi = <JsValue as IntoWasmAbi>::Abi;

    fn into_abi(self) -> Self::Abi {
        serde_wasm_bindgen::to_value(&self).unwrap().into_abi()
    }
}

impl wasm_bindgen::convert::FromWasmAbi for TypstCoreError {
    type Abi = <JsValue as FromWasmAbi>::Abi;

    unsafe fn from_abi(js: Self::Abi) -> Self {
        serde_wasm_bindgen::from_value(JsValue::from_abi(js)).unwrap()
    }
}