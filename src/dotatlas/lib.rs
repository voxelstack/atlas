use std::panic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    web_sys::window()
        .expect("Should have a window.")
        .alert_with_message("Hello from dotatlas with web-sys!")?;

    Ok(())
}
