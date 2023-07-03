use atlas_comms::{port::Port, Payload};
use log::trace;
use tokio::sync::mpsc::unbounded_channel;
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

pub use atlas_comms::init_output;
pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub struct AtlasServer {
    _scope: DedicatedWorkerGlobalScope,
    port: Port,
}

#[wasm_bindgen]
impl AtlasServer {
    #[wasm_bindgen(constructor)]
    pub fn new(scope: DedicatedWorkerGlobalScope) -> Self {
        Self {
            _scope: scope.clone(),
            port: Port::wrap(Box::new(scope)),
        }
    }

    pub async fn listen(&self) {
        let (tx, mut rx) = unbounded_channel();

        let listener = self
            .port
            .add_listener(Closure::new(move |event: MessageEvent| {
                let payload: Payload = event.data().into();
                tx.send(payload.0).expect("Server channel should be open.");
            }));

        while let Some(message) = rx.recv().await {
            trace!("wasm server received: {}", message);
        }

        listener.clear();
    }
}
