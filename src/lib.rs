#![feature(assert_matches)]

pub mod typst;
mod utils;
pub mod ast;

pub fn add(left: u64, right: u64) -> u64 {
    console_log!("Adding {} + {}", left, right);
    left + right
}

#[cfg(test)]
mod tests {

    use std::assert_matches::assert_matches;

    use crate::ast::debug_print_ast;
    use crate::typst::wasm::structs::definition;
    use crate::typst::wasm::structs::error::TypstCoreError;
    use crate::typst::wasm::structs::output::OutputFormat;
    use crate::typst::wasm::structs::range::MonacoRange;
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
            MonacoRange {
                begin_column: 1,
                begin_line_number: 1,
                end_column: 11,
                end_line_number: 1,
            });
        
        let result = core.compile(OutputFormat::Html);
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);
        let result = result.unwrap();
        console_log!("Result: {:#?}", result.html().unwrap());
        console_log!("Result: {:#?}", core.get_source("/main.typ".to_owned()).unwrap());
    }

    #[wasm_bindgen_test]
    fn test_definition() {
        let mut core = TypstCore::construct();

        core.add_source(
            "/main.typ".to_owned(),
            "#let x = 1 + 2;".to_owned(),
        );
        let result = core.set_root("/main.typ".to_owned());
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);
        let result = core.compile(OutputFormat::Html);
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);
        let src = core.get_source_file("/main.typ".to_owned()).unwrap();
        debug_print_ast(src.source());

        let definition = core.definition("/main.typ".to_owned(), MonacoRange { begin_line_number: 1, begin_column: 6, end_line_number: 1, end_column: 7 });
        assert!(definition.is_ok(), "Expected a definition, but got none");
        console_log!("Definition: {:?}", definition);
    }


    #[wasm_bindgen_test]
    fn test_definition_std() {
        let mut core = TypstCore::construct();

        core.add_source(
            "/main.typ".to_owned(),
            "#block()[]".to_owned(),
        );
        let result = core.set_root("/main.typ".to_owned());
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);
        let result = core.compile(OutputFormat::Html);
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);

        let definition = core.definition("/main.typ".to_owned(), MonacoRange { begin_line_number: 1, begin_column: 2, end_line_number: 1, end_column: 6 });
        assert!(definition.is_ok(), "Expected a definition, but got none");
        console_log!("Definition: {:?}", definition);
    }

    #[wasm_bindgen_test]
    fn test_definition_tidy() {
        let mut core = TypstCore::construct();

        core.add_source(
            "/main.typ".to_owned(),
            "//Comment\n///Comment2 \n \n \r\n/// Pi is cool\r\n/// -> float\n#let pi = 3.1415\n\n/// LOL!!!!\n\t\n/// Function \n/// -> bool \r\n#let is_ok(var) = {var == pi}".to_owned(),
        );
        let result = core.set_root("/main.typ".to_owned());
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);
        let result = core.compile(OutputFormat::Html);
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);
        let src = core.get_source_file("/main.typ".to_owned()).unwrap();
        debug_print_ast(src.source());

        let definition = core.definition("/main.typ".to_owned(), MonacoRange { begin_line_number: 7, begin_column: 6, end_line_number: 7, end_column: 7 });
        assert!(definition.is_ok(), "Expected a definition, but got none");
        console_log!("Definition: {:?}", definition);


        let definition = core.definition("/main.typ".to_owned(), MonacoRange { begin_line_number: 13, begin_column: 6, end_line_number: 13, end_column: 10 });
        assert!(definition.is_ok(), "Expected a definition, but got none");
        console_log!("Definition: {:?}", definition);
    }

    #[wasm_bindgen_test]
    fn test_definition_tidy_1() {
        let mut core = TypstCore::construct();

        core.add_source(
            "/main.typ".to_owned(),
            "/// Function \n/// -> bool \r\n#let is_ok(\n/// variable\n/// -> float\nvar) = {var == pi}".to_owned(),
        );
        let result = core.set_root("/main.typ".to_owned());
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);
        let result = core.compile(OutputFormat::Html);
        assert!(result.is_ok(), "Expected no error, but got: {:?}", result);
        let src = core.get_source_file("/main.typ".to_owned()).unwrap();
        debug_print_ast(src.source());

        let definition = core.definition("/main.typ".to_owned(), MonacoRange { begin_line_number: 3, begin_column: 6, end_line_number: 3, end_column: 11 });
        assert!(definition.is_ok(), "Expected a definition, but got none");
        console_log!("Definition: {:?}", definition);
    }
}
