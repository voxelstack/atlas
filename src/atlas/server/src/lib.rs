use atlas_comms::{
    client::ClientMessage,
    port::Port,
    server::{ServerEvent, ServerMessage, ServerResponse},
    Payload,
};
use log::trace;
use tokio::sync::mpsc::unbounded_channel;
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

pub use atlas_comms::init_output;
pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub struct AtlasServer {
    _scope: DedicatedWorkerGlobalScope,
    counter: u8,
    wires: Vec<Port>,
    port: Port,
}

#[wasm_bindgen]
impl AtlasServer {
    #[wasm_bindgen(constructor)]
    pub fn new(scope: DedicatedWorkerGlobalScope) -> Self {
        Self {
            _scope: scope.clone(),
            counter: 0,
            wires: Vec::new(),
            port: Port::wrap(Box::new(scope)),
        }
    }

    pub async fn listen(&mut self) {
        let (tx, mut rx) = unbounded_channel();

        let listener = self
            .port
            .add_listener(Closure::new(move |event: MessageEvent| {
                tx.send(event).expect("Server channel should be open.");
            }));

        while let Some(event) = rx.recv().await {
            let payload: Payload<ClientMessage> = event.data().try_into().unwrap();
            trace!("client->[server]: {:?}", payload);

            let Payload { id, message } = payload;
            let res = match message {
                ClientMessage::Ping => ServerResponse::Ok(ServerMessage::Ok),
                ClientMessage::Query => {
                    self.push_event(ServerEvent::Count(self.counter));
                    ServerResponse::Ok(ServerMessage::Ok)
                }
                ClientMessage::Inc => {
                    self.counter += 1;

                    self.push_event(ServerEvent::Count(self.counter));
                    ServerResponse::Ok(ServerMessage::Ok)
                }
                ClientMessage::Dec => {
                    self.counter -= 1;

                    self.push_event(ServerEvent::Count(self.counter));
                    ServerResponse::Ok(ServerMessage::Ok)
                }
                ClientMessage::Attach(surface) => {
                    atlas_graphics::list_adapters(surface).await;
                    ServerResponse::Ok(ServerMessage::Ok)
                }
                ClientMessage::WireUp(port) => {
                    self.wires.push(Port::wrap(Box::new(port)));
                    ServerResponse::Ok(ServerMessage::Ok)
                }
            };

            self.port.send(Payload { id, message: res });
        }

        listener.clear();
    }

    fn push_event(&self, event: ServerEvent) {
        for wire in &self.wires {
            wire.send(event.clone());
        }
    }
}
