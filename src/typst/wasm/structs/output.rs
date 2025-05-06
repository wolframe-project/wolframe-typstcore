use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::{convert::{FromWasmAbi, IntoWasmAbi}, prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub enum OutputFormat {
    Svg = "svg",
    Html = "html",
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
pub enum Output {
    Svg(Vec<String>),
    Html(String),
}

#[wasm_bindgen]
impl Output {
    pub fn svg(self) -> Option<Vec<String>> {
        if let Output::Svg(svg) = self {
            Some(svg)
        } else {
            None
        }
    }

    pub fn html(self) -> Option<String> {
        if let Output::Html(html) = self {
            Some(html)
        } else {
            None
        }
    }
}

impl From<Output> for JsValue {
    fn from(val: Output) -> Self {
        serde_wasm_bindgen::to_value(&val).unwrap()
    }
}

impl wasm_bindgen::describe::WasmDescribe for Output {
    fn describe() {
        JsValue::describe()
    }
}

impl wasm_bindgen::convert::IntoWasmAbi for Output {
    type Abi = <JsValue as IntoWasmAbi>::Abi;

    fn into_abi(self) -> Self::Abi {
        serde_wasm_bindgen::to_value(&self).unwrap().into_abi()
    }
}

impl wasm_bindgen::convert::FromWasmAbi for Output {
    type Abi = <JsValue as FromWasmAbi>::Abi;

    unsafe fn from_abi(js: Self::Abi) -> Self {
        serde_wasm_bindgen::from_value(JsValue::from_abi(js)).unwrap()
    }
}