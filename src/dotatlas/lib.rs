use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get() -> String {
    console_error_panic_hook::set_once();

    "pong".into()
}
