use atlas_comms_derive::Shareable;
use wasm_bindgen::JsValue;
use web_sys::OffscreenCanvas;

#[derive(Debug, Shareable)]
pub enum ClientMessage {
    Ping,
    Attach(#[shareable(repr = "raw", transfer)] OffscreenCanvas),
}
