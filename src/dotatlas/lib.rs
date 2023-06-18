use std::panic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    alert("Hello from dotatlas!");
}
