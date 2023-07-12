use atlas_comms::{
    client::ClientMessage,
    port::{Listener, Port},
    server::{ServerEvent, ServerMessage, ServerResponse},
    Payload,
};
use log::trace;
use tokio::sync::mpsc::channel;
use wasm_bindgen::prelude::*;
use web_sys::{MessageChannel, MessageEvent, OffscreenCanvas, Worker};

pub use atlas_comms::init_output;

#[wasm_bindgen]
pub struct AtlasClient {
    _server: Worker,
    wire: Option<(Port, Listener<'static>)>,
    pipe: Port,
}

#[wasm_bindgen]
impl AtlasClient {
    #[wasm_bindgen(constructor)]
    pub fn new(server: Worker) -> Self {
        Self {
            _server: server.clone(),
            wire: None,
            pipe: Port::wrap(Box::new(server)),
        }
    }

    pub async fn listen(&mut self) {
        let channel = MessageChannel::new().unwrap();
        let (rx, tx) = (channel.port1(), channel.port2());

        let res = self.request(ClientMessage::WireUp(tx)).await;
        if let ServerResponse::Ok(ServerMessage::Ok) = res {
            let wire = Port::wrap(Box::new(rx));
            let handle = wire.add_listener(Closure::new(move |event: MessageEvent| {
                let event: ServerEvent = event.data().try_into().unwrap();
                trace!("[··wire]<-server: {:?}", event);
            }));

            // From wasm_bindgen:
            // structs with #[wasm_bindgen] cannot have lifetime or type
            // parameters currently
            //
            // Instead of calling mem::forget(handle), transmute the lifetime to
            // 'static so we can still store it. There might be a better way of
            // handling this, but it's not a priority since right now there's
            // only one client (and it's likely that's always gonna be the case)
            let handle = unsafe { std::mem::transmute(handle) };

            self.wire = Some((wire, handle));
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
        let id: u8 = rand::random();

        let listener = self
            .pipe
            .add_listener(Closure::new(move |event: MessageEvent| {
                let payload: Payload<ServerResponse> = event.data().try_into().unwrap();
                trace!("[client]<-server: {:?}", payload);

                if payload.id == id {
                    tx.try_send(payload.message)
                        .expect("Request channel should not be closed.");
                }
            }));

        let payload = Payload { id, message };
        self.pipe.send(payload);

        let response = rx.recv().await.expect("Should post a valid response.");
        listener.clear();

        response
    }
}
