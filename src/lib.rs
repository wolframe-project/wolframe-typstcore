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

    use crate::typst::wasm::structs::error::TypstCoreError;
    use crate::typst::TypstCore;

    use super::*;
    use ::typst::World;
    use wasm_bindgen_test::__rt::console_log;
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

    #[wasm_bindgen_test]
    fn test_render_0() {
        let mut core = TypstCore::construct();

        core.add_source(
            "/main.typ".to_owned(),
            r#"Hello World
#let x = 1+2;
#x
"#
            .to_owned(),
        );
        let result = core.set_root("/main.typ".to_owned());
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);

        let result = core.compile();
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);

        console_log!("Result: {:#?}", result.unwrap()[0]);
    }
}
