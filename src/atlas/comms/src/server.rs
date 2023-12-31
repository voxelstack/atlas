use atlas_comms_derive::Shareable;
use wasm_bindgen::JsValue;

#[derive(Debug, Shareable)]
pub enum ServerMessage {
    Ok,
}

#[derive(Debug, Shareable)]
pub enum ServerError {
    Unknown,
}

#[derive(Debug, Shareable)]
pub enum ServerResponse {
    Ok(ServerMessage),
    Err(ServerError),
}

#[derive(Clone, Copy, Debug, Shareable)]
pub enum ServerEvent {
    Count(#[shareable(repr = "serde")] u8),
}
