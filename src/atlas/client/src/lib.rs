use atlas_comms::{client::ClientMessage, port::Port, server::ServerResponse, Payload};
use log::trace;
use tokio::sync::mpsc::channel;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, OffscreenCanvas, Worker};

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

    pub async fn attach(&self, surface: OffscreenCanvas) {
        self.request(ClientMessage::Attach(surface)).await;
    }

    pub async fn ping(&self) {
        self.request(ClientMessage::Ping).await;
    }

    async fn request(&self, message: ClientMessage) -> ServerResponse {
        let (tx, mut rx) = channel::<ServerResponse>(1);
        let id = js_sys::Number::from(rand::random::<u8>());

        let id_clone = id.clone();
        let listener = self
            .port
            .add_listener(Closure::new(move |event: MessageEvent| {
                let payload: Payload<ServerResponse> = event.data().into();
                trace!("client got back: {:?}", payload);

                if payload.id == id_clone {
                    tx.try_send(payload.message)
                        .expect("Request channel should not be closed.");
                }
            }));

        let payload = Payload { id, message };
        self.port.send(payload);

        let response = rx.recv().await.expect("Should post a valid response.");
        listener.clear();

        response
    }
}
