use atlas_comms::Message;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet() -> Message {
    Message { id: 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_worker);

    #[wasm_bindgen_test]
    fn msg_id() {
        assert_eq!(greet().id, 0);
    }
}
