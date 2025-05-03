mod utils;
mod typst;


pub fn add(left: u64, right: u64) -> u64 {
    console_log!("Adding {} + {}", left, right);
    left + right
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;
    use super::*;

    #[wasm_bindgen_test]
    fn it_works() {
        let result = add(2, 2);
        println!("Result: {}", result);
        assert_eq!(result, 4);
    }
}
