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
    use crate::typst::wasm::structs::output::OutputFormat;
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

        let result = core.compile(OutputFormat::Svg);
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

table(columns:2)[*HI*][$phi * frak(X)^(10/5)$]
"#
            .to_owned(),
        );
        let result = core.set_root("/main.typ".to_owned());
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);

        let result = core.compile(OutputFormat::Html);
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);

        console_log!("Result: {:#?}", result.unwrap().html().unwrap());
    }

    #[wasm_bindgen_test]
    fn test_edit_with_untruncated_lines() {
        let mut core = TypstCore::construct();

        core.add_source(
            "/main.typ".to_owned(),
            r#"Hello World"#
                .to_owned(),
        );
        let result = core.set_root("/main.typ".to_owned());
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);

        let _ = core.edit_source(
            "/main.typ".to_owned(), 
            "Zum einrichten des iBGPs haben wir folgende, wie im Wiki beschriebene Befehle genutzt:\n  ```\n  router bgp 3\n    neighbor 3.[150 + Y].0.1 remote-as 3\n    neighbor 3.[150 + Y].0.1 update-source lo\n  ```\n  Für jeden Router dann dementsprechend sieben Mal, ein Router braucht zu sich selber keine BGP session.\n\n  Für ROME router sieht das dann so aus:\n  ```\nROME_router# conf t\nROME_router(config)# router bgp 3\nROME_router(config-router)# neighbor 3.151.0.1 remote-as 3\nROME_router(config-router)# neighbor 3.152.0.1 remote-as 3\nROME_router(config-router)# neighbor 3.153.0.1 remote-as 3\nROME_router(config-router)# neighbor 3.154.0.1 remote-as 3\nROME_router(config-router)# neighbor 3.155.0.1 remote-as 3\nROME_router(config-router)# neighbor 3.156.0.1 remote-as 3\nROME_router(config-router)# neighbor 3.157.0.1 remote-as 3\nROME_router(config-router)# neighbor 3.151.0.1 update-source lo\nROME_router(config-router)# neighbor 3.152.0.1 update-source lo\nROME_router(config-router)# neighbor 3.153.0.1 update-source lo\nROME_router(config-router)# neighbor 3.154.0.1 update-source lo\nROME_router(config-router)# neighbor 3.155.0.1 update-source lo\nROME_router(config-router)# neighbor 3.156.0.1 update-source lo\nROME_router(config-router)# neighbor 3.157.0.1 update-source lo\n  ```".to_owned(), 
            0, 11);
        
        let result = core.compile(OutputFormat::Html);
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);
        let result = result.unwrap();
        console_log!("Result: {:#?}", result.html().unwrap());
        console_log!("Result: {:#?}", core.get_source("/main.typ".to_owned()).unwrap());
    }
}
