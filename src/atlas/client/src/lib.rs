use atlas_comms::{
    client::ClientMessage,
    port::{Listener, Port},
    server::{ServerEvent, ServerMessage, ServerResponse},
    Payload,
};
use log::trace;
use tokio::sync::mpsc::channel;
use wasm_bindgen::prelude::*;
use web_sys::{BroadcastChannel, MessageChannel, MessageEvent, OffscreenCanvas, Worker};

pub use atlas_comms::init_output;

const BUS_PREFIX: &str = "atlas_bus";

#[wasm_bindgen]
pub struct AtlasClient {
    _server: Worker,
    pipe: Port,
    wire: Option<(Port, Listener<'static>)>,
    bus_id: String,
}

#[wasm_bindgen]
impl AtlasClient {
    #[wasm_bindgen(constructor)]
    pub fn new(server: Worker) -> Self {
        Self {
            _server: server.clone(),
            wire: None,
            pipe: Port::wrap(Box::new(server)),
            bus_id: format!("{}#{}", BUS_PREFIX, rand::random::<u8>()),
        }
    }

    pub async fn listen(&mut self) {
        let channel = MessageChannel::new().unwrap();
        let (rx, tx) = (channel.port1(), channel.port2());

        let res = self.request(ClientMessage::WireUp(tx)).await;
        if let ServerResponse::Ok(ServerMessage::Ok) = res {
            let wire = Port::wrap(Box::new(rx));
            let bus_id = self.bus_id.clone();
            let handle = wire.add_listener(Closure::new(move |event: MessageEvent| {
                let event: ServerEvent = event.data().try_into().unwrap();
                trace!("[··wire]<-server: {:?}", event);

                let channel = BroadcastChannel::new(&bus_id).unwrap();

                // TODO This shouldn't be manual.
                // The event data should probably be generic over #[wasm_bindgen]
                // structs so observables can subscribe with a function that
                // receives a typed payload.
                let payload = js_sys::Array::new();
                match event {
                    ServerEvent::Count(value) => {
                        payload.push(&JsValue::from(stringify!(ServerEvent::Count)));
                        payload.push(&JsValue::from(value));
                    }
                }

                channel.post_message(&payload).unwrap();
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

    pub async fn query(&self) {
        self.request(ClientMessage::Query).await;
    }

    pub async fn inc(&self) {
        self.request(ClientMessage::Inc).await;
    }

    pub async fn dec(&self) {
        self.request(ClientMessage::Dec).await;
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

    pub fn observe(&mut self, observable: String) -> Observable {
        Observable {
            id: observable,
            channel: BroadcastChannel::new(&self.bus_id).unwrap(),
            listeners: Vec::new(),
        }
    }
}

#[wasm_bindgen]
pub struct Observable {
    id: String,
    channel: BroadcastChannel,
    listeners: Vec<Closure<dyn Fn(MessageEvent)>>,
}

#[wasm_bindgen]
impl Observable {
    pub fn subscribe(&mut self, on_change: js_sys::Function) -> Result<JsValue, JsValue> {
        let id = self.id.clone();
        let listener = Closure::<dyn Fn(MessageEvent)>::new(move |event: MessageEvent| {
            let event: js_sys::Array = event.data().into();

            let event_id = event.get(0).as_string().unwrap();
            if event_id == id {
                on_change
                    .call1(&JsValue::undefined(), &event.get(1))
                    .unwrap();
            }
        });
        self.channel
            .add_event_listener_with_callback("message", listener.as_ref().unchecked_ref())
            .unwrap();
        self.listeners.push(listener);

        let listener = self.listeners.last().unwrap();
        let listener_handle: &js_sys::Function = listener.as_ref().unchecked_ref();
        let listener_handle = listener_handle.clone();
        let channel_handle = self.channel.clone();
        let unsubscribe = Closure::<dyn Fn()>::new(move || {
            channel_handle
                .remove_event_listener_with_callback("message", &listener_handle)
                .unwrap();
        });
        Ok(unsubscribe.into_js_value())
    }
}
