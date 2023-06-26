use async_trait::async_trait;
use std::{cell::RefCell, panic, rc::Rc};
use tokio::sync::mpsc;
use wasm_bindgen::{prelude::*, JsValue};
use web_sys::{MessageChannel, MessageEvent, Worker};

#[wasm_bindgen(js_name = setPanicHook)]
pub fn set_panic_hook() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[derive(Debug)]
pub enum CommError {
    Unknown,
}

pub trait Dispatch {
    fn into_transfer_pair(&self) -> (JsValue, js_sys::Array);
}

pub trait Dispatcher {
    fn post_message(&self, message: JsValue, transferable: js_sys::Array) -> Result<(), CommError>;
}

#[async_trait(?Send)]
pub trait Wire<SenderMessage, ReceiverMessage, ReceiverError>
where
    SenderMessage: Dispatch + Send,
    ReceiverMessage: From<JsValue> + std::fmt::Debug + Send,
    ReceiverError: From<JsValue> + std::fmt::Debug + Send,
{
    fn dispatcher(&self) -> &dyn Dispatcher;

    async fn send(
        &self,
        message: SenderMessage,
    ) -> Result<Result<ReceiverMessage, ReceiverError>, CommError>
    where
        SenderMessage: 'async_trait,
        ReceiverMessage: 'static,
        ReceiverError: 'static,
    {
        let (message, transferable) = message.into_transfer_pair();
        // tx needs to be moved into both onmessage and onerror. This is used as a oneshot channel
        // but we need mpsc since tokio::sync::oneshot::Sender is not clone.
        let (tx, mut rx) = mpsc::channel::<Result<ReceiverMessage, ReceiverError>>(1);

        let channel = MessageChannel::new().map_err(|_| CommError::Unknown)?;
        let onmessage_handle = Rc::new(RefCell::new(None));

        let tx_clone = tx.clone();
        let port1_clone = channel.port1();
        let onmessage_handle_clone = onmessage_handle.clone();
        let onmessage = Closure::once(move |event: MessageEvent| {
            let data: js_sys::Array = event.data().into();
            let result = data.get(0).as_string().unwrap();
            let payload = data.get(1);

            // TODO This should notify the channel that the connection might have been closed. The
            // same should happen on the onerror handle.
            tx_clone
                .try_send(match result.as_ref() {
                    "Ok" => Ok(payload.into()),
                    "Err" => Err(payload.into()),
                    _ => unreachable!(),
                })
                .expect("Should be able to send event data.");

            port1_clone.close();
            drop(onmessage_handle_clone);
        });

        channel
            .port1()
            .set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage_handle.borrow_mut().replace(onmessage);

        transferable.push(&channel.port2().into());
        self.dispatcher()
            .post_message(message, transferable)
            .map_err(|_| CommError::Unknown)?;

        rx.recv()
            .await
            .map_or(Err(CommError::Unknown), |res| Ok(res))
    }
}

// Ideally messages and errors would be string enums but there's an open issue with wasm-bindgen and
// those don't work.
// https://github.com/rustwasm/wasm-bindgen/issues/3057
#[derive(Debug)]
pub enum ClientMessage {
    Ping,
}

impl Dispatch for ClientMessage {
    fn into_transfer_pair(&self) -> (JsValue, js_sys::Array) {
        match self {
            ClientMessage::Ping => (JsValue::from("0"), js_sys::Array::new()),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub enum ServerMessage {
    Pong,
}

impl From<JsValue> for ServerMessage {
    fn from(value: JsValue) -> Self {
        let value: js_sys::Array = value.into();
        let message = value.get(0).as_string().unwrap();

        match message.as_ref() {
            "0" => ServerMessage::Pong,
            _ => unreachable!(),
        }
    }
}

impl From<ServerMessage> for JsValue {
    fn from(value: ServerMessage) -> Self {
        match value {
            ServerMessage::Pong => JsValue::from(0),
        }
    }
}

#[derive(Debug)]
pub enum ServerError {
    Unknown,
}

impl From<JsValue> for ServerError {
    fn from(value: JsValue) -> Self {
        let value: js_sys::Array = value.into();
        let message = value.get(0).as_string().unwrap();

        match message.as_ref() {
            "0" => ServerError::Unknown,
            _ => unreachable!(),
        }
    }
}

impl From<ServerError> for JsValue {
    fn from(value: ServerError) -> Self {
        match value {
            ServerError::Unknown => JsValue::from(0),
        }
    }
}

pub struct ServerProxy {
    handle: Worker,
}

impl Dispatcher for ServerProxy {
    fn post_message(&self, message: JsValue, transferable: js_sys::Array) -> Result<(), CommError> {
        self.handle
            .post_message_with_transfer(&message, &transferable)
            .map_err(|_| CommError::Unknown)
    }
}

impl Wire<ClientMessage, ServerMessage, ServerError> for ServerProxy {
    fn dispatcher(&self) -> &dyn Dispatcher {
        self
    }
}

impl ServerProxy {
    pub fn wrap(handle: Worker) -> Self {
        Self { handle }
    }

    pub async fn ping(&self) -> Result<ServerMessage, ServerError> {
        self.send(ClientMessage::Ping).await.unwrap()
    }
}
