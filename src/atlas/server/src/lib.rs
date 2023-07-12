use atlas_comms::{
    client::ClientMessage,
    port::Port,
    server::{ServerError, ServerEvent, ServerMessage, ServerResponse},
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
    wire: Option<Port>,
    port: Port,
}

#[wasm_bindgen]
impl AtlasServer {
    #[wasm_bindgen(constructor)]
    pub fn new(scope: DedicatedWorkerGlobalScope) -> Self {
        Self {
            _scope: scope.clone(),
            counter: 0,
            wire: None,
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
                    if let Some(wire) = &self.wire {
                        wire.send(ServerEvent::Count(self.counter));
                    }
                    ServerResponse::Ok(ServerMessage::Ok)
                }
                ClientMessage::Inc => {
                    self.counter += 1;

                    if let Some(wire) = &self.wire {
                        wire.send(ServerEvent::Count(self.counter));
                    }
                    ServerResponse::Ok(ServerMessage::Ok)
                }
                ClientMessage::Dec => {
                    self.counter -= 1;

                    if let Some(wire) = &self.wire {
                        wire.send(ServerEvent::Count(self.counter));
                    }
                    ServerResponse::Ok(ServerMessage::Ok)
                }
                ClientMessage::Attach(_) => ServerResponse::Err(ServerError::Unknown),
                ClientMessage::WireUp(port) => {
                    self.wire = Some(Port::wrap(Box::new(port)));
                    ServerResponse::Ok(ServerMessage::Ok)
                }
            };

            self.port.send(Payload { id, message: res });
        }

        listener.clear();
    }
}
