use std::fmt::{self, Debug};

use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, MessagePort, Worker};

pub trait RawPort {
    fn send_raw(&self, message: JsValue);
    fn transfer_raw(&self, message: JsValue, transfer: JsValue);
    fn add_raw_listener(&self, listener: &js_sys::Function);
    fn remove_raw_listener(&self, listener: &js_sys::Function);
    fn start(&self) {}
}

impl RawPort for Worker {
    fn send_raw(&self, message: JsValue) {
        self.post_message(&message)
            .expect("Should have error handling.");
    }

    fn transfer_raw(&self, message: JsValue, transfer: JsValue) {
        self.post_message_with_transfer(&message, &transfer)
            .expect("Should have error handling.");
    }

    fn add_raw_listener(&self, listener: &js_sys::Function) {
        self.add_event_listener_with_callback("message", listener)
            .expect("Should have error handling.");
    }

    fn remove_raw_listener(&self, listener: &js_sys::Function) {
        self.remove_event_listener_with_callback("message", listener)
            .expect("Should have error handling.");
    }
}

impl RawPort for DedicatedWorkerGlobalScope {
    fn send_raw(&self, message: JsValue) {
        self.post_message(&message)
            .expect("Should have error handling.");
    }

    fn transfer_raw(&self, message: JsValue, transfer: JsValue) {
        self.post_message_with_transfer(&message, &transfer)
            .expect("Should have error handling.");
    }

    fn add_raw_listener(&self, listener: &js_sys::Function) {
        self.add_event_listener_with_callback("message", listener)
            .expect("Should have error handling.");
    }

    fn remove_raw_listener(&self, listener: &js_sys::Function) {
        self.remove_event_listener_with_callback("message", listener)
            .expect("Should have error handling.");
    }
}

impl RawPort for MessagePort {
    fn send_raw(&self, message: JsValue) {
        self.post_message(&message)
            .expect("Should have error handling.");
    }

    fn transfer_raw(&self, message: JsValue, transfer: JsValue) {
        self.post_message_with_transferable(&message, &transfer)
            .expect("Should have error handling.");
    }

    fn add_raw_listener(&self, listener: &js_sys::Function) {
        self.add_event_listener_with_callback("message", listener)
            .expect("Should have error handling.");
    }

    fn remove_raw_listener(&self, listener: &js_sys::Function) {
        self.remove_event_listener_with_callback("message", listener)
            .expect("Should have error handling.");
    }

    fn start(&self) {
        self.start();
    }
}

pub struct Listener<'a> {
    owner: &'a dyn RawPort,
    inner: Closure<dyn Fn(MessageEvent)>,
}

impl<'a> Listener<'a> {
    pub fn clear(self) {
        self.remove_listener();
    }

    fn remove_listener(&self) {
        self.owner
            .remove_raw_listener(self.inner.as_ref().unchecked_ref());
    }
}

impl<'a> Drop for Listener<'a> {
    fn drop(&mut self) {
        self.remove_listener();
    }
}

// TODO Reconsider the + Debug bound, right now it's there so I can unwrap tokio
//  channel errors.
pub trait Shareable:
    TryInto<(JsValue, Option<JsValue>), Error = ShareableError>
    + TryFrom<JsValue, Error = ShareableError>
    + Debug
{
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ShareableError {
    IncompatibleType,
    BadPayload,
    SerdeFailure,
}

impl fmt::Display for ShareableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShareableError::IncompatibleType => {
                write!(
                    f,
                    "the type of the payload doesn't match the type being read"
                )
            }
            ShareableError::BadPayload => {
                write!(f, "invalid payload format")
            }
            ShareableError::SerdeFailure => {
                write!(f, "serde failed while sharing")
            }
        }
    }
}

pub struct Port(Box<dyn RawPort>);

impl Port {
    pub fn wrap(raw_port: Box<dyn RawPort>) -> Self {
        raw_port.start();
        Self(raw_port)
    }

    pub fn send<M>(&self, message: M)
    where
        M: Shareable,
    {
        let (data, transfer) = message.try_into().unwrap();
        match transfer {
            Some(transfer) => self.0.transfer_raw(data, transfer),
            None => self.0.send_raw(data),
        }
    }

    pub fn add_listener(&self, listener: Closure<dyn Fn(MessageEvent)>) -> Listener {
        self.0.add_raw_listener(listener.as_ref().unchecked_ref());

        Listener {
            owner: self.0.as_ref(),
            inner: listener,
        }
    }
}
