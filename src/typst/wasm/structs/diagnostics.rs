use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typst::{diag::SourceDiagnostic, syntax::FileId};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::typst::source_file::SourceFile;

use super::{error::TypstCoreError, range::TypstCoreRange};


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