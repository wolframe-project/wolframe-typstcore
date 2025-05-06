use serde::{Deserialize, Serialize};
use tsify::Tsify;
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
