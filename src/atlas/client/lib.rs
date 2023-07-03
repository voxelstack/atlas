use atlas_comms::{port::Port, Payload};
use wasm_bindgen::prelude::*;
use web_sys::Worker;

pub use atlas_comms::init_output;

#[wasm_bindgen]
pub struct AtlasClient {
    _server: Worker,
    port: Port,
}

#[wasm_bindgen]
impl AtlasClient {
    #[wasm_bindgen(constructor)]
    pub fn new(server: Worker) -> Self {
        Self {
            _server: server.clone(),
            port: Port::wrap(Box::new(server)),
        }
    }

    pub fn send(&self, message: String) {
        self.port.send(Payload(message));
    }
}
