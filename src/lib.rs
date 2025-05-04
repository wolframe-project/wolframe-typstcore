#![feature(assert_matches)]

pub mod typst;
mod utils;

pub fn add(left: u64, right: u64) -> u64 {
    console_log!("Adding {} + {}", left, right);
    left + right
}

#[cfg(test)]
mod tests {

    use std::assert_matches::assert_matches;

    use crate::typst::wasm::structs::TypstCoreError;
    use crate::typst::TypstCore;

    use super::*;
    use ::typst::World;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn fonts_ok() {
        let core = TypstCore::construct();

        assert!(core.font(0).is_some()); // Check if the first font is available, should be if the typst_assets crate is included with features = ["fonts"]
    }

    #[wasm_bindgen_test]
    fn no_root_on_compile() {
        let core = TypstCore::construct();

        let result = core.compile();
        assert!(
            result.is_err(),
            "Expected an error when root is not set, but got: {:?}",
            result
        );

        let err = result.unwrap_err();

        console_log!("Error: {:?}", err);

        assert_matches!(err, TypstCoreError::DefaultError(_));
    }
}
