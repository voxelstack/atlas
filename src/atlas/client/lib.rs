use atlas_comms::{ServerError, ServerMessage, ServerProxy};
use wasm_bindgen::prelude::*;
use web_sys::Worker;

pub use atlas_comms::set_panic_hook;

#[wasm_bindgen]
pub struct AtlasClient {
    server: ServerProxy,
}

#[wasm_bindgen]
impl AtlasClient {
    #[wasm_bindgen(constructor)]
    pub fn new(worker: Worker) -> Self {
        Self {
            server: ServerProxy::wrap(worker),
        }
    }

    pub async fn ping(&self) -> Result<ServerMessage, ServerError> {
        self.server.ping().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_worker);

    #[wasm_bindgen_test]
    fn pass() {
        assert_eq!(0, 0);
    }
}
