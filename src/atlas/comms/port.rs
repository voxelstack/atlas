use std::fmt::Debug;

use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, Worker};

pub trait RawPort {
    fn send_raw(&self, message: JsValue);
    fn transfer_raw(&self, message: JsValue, transfer: JsValue);
    fn add_raw_listener(&self, listener: &js_sys::Function);
    fn remove_raw_listener(&self, listener: &js_sys::Function);
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

pub trait Shareable: Into<(JsValue, Option<JsValue>)> + From<JsValue> + Debug {}

pub struct Port(Box<dyn RawPort>);

impl Port {
    pub fn wrap(raw_port: Box<dyn RawPort>) -> Self {
        Self(raw_port)
    }

    pub fn send<M>(&self, message: M)
    where
        M: Shareable,
    {
        let (data, transfer) = message.into();
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
