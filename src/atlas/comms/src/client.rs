use atlas_comms_derive::Shareable;
use wasm_bindgen::JsValue;
use web_sys::{MessagePort, OffscreenCanvas};

#[derive(Debug, Shareable)]
pub enum ClientMessage {
    Ping,
    Query,
    Inc,
    Dec,
    Attach(#[shareable(repr = "raw", transfer)] OffscreenCanvas),
    WireUp(#[shareable(repr = "raw", transfer)] MessagePort),
}
