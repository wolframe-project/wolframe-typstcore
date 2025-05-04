use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use typst::{
    diag::SourceDiagnostic,
    syntax::{package::PackageSpec, FileId, Source, Span},
};
use wasm_bindgen::{
    convert::{FromWasmAbi, IntoWasmAbi},
    prelude::wasm_bindgen,
    JsValue,
};

use crate::typst::source_file::SourceFile;

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

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct TypstCorePackage {
    pub name: String,
    pub namespace: String,
    pub version: String,
}

impl PartialEq<PackageSpec> for TypstCorePackage {
    fn eq(&self, other: &PackageSpec) -> bool {
        self.name == other.name
            && self.namespace == other.namespace
            && self.version == format!("{:?}", other.version)
    }
}

impl From<PackageSpec> for TypstCorePackage {
    fn from(val: PackageSpec) -> Self {
        TypstCorePackage {
            name: val.name.to_string(),
            namespace: val.namespace.to_string(),
            version: format!("{:?}", val.version),
        }
    }
}

impl PartialEq<TypstCorePackage> for PackageSpec {
    fn eq(&self, other: &TypstCorePackage) -> bool {
        self.name == other.name
            && self.namespace == other.namespace
            && format!("{:?}", self.version) == other.version
    }
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TypstCoreSeverity {
    Error = "error",
    Warning = "warning",
}

impl From<typst::diag::Severity> for TypstCoreSeverity {
    fn from(severity: typst::diag::Severity) -> Self {
        match severity {
            typst::diag::Severity::Error => Self::Error,
            typst::diag::Severity::Warning => Self::Warning,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypstCoreRange {
    pub path: String,
    pub start: usize,
    pub end: usize,
}

impl TypstCoreRange {
    pub fn with_source(span: Span, source: &Source) -> Result<Self, TypstCoreError> {
        if span.is_detached() {
            Ok(Self {
                path: String::new(),
                start: 0,
                end: 0,
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
                start: range.start,
                end: range.end,
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
                start: 0,
                end: 0,
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

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypstCoreDiagnostics {
    pub severity: TypstCoreSeverity,
    pub message: String,
    pub range: TypstCoreRange,
}

impl TypstCoreDiagnostics {
    pub fn from_diagnostics(
        err: SourceDiagnostic,
        sources: &HashMap<FileId, SourceFile>,
    ) -> Result<Self, TypstCoreError> {
        let severity = TypstCoreSeverity::from(err.severity);

        let message = err.message.to_string();

        let range = TypstCoreRange::with_sources(err.span, sources)?;

        Ok(Self {
            severity,
            message,
            range,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
pub enum TypstCoreError {
    CompileError(Vec<TypstCoreDiagnostics>),
    DefaultError(String),
}

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
