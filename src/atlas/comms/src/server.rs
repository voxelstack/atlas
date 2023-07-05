use crate::port::{Shareable, ShareableError};
use wasm_bindgen::JsValue;

// TODO #[derive(Shareable)]
#[derive(Debug)]
pub enum ServerMessage {
    Ok,
}

// TODO #[derive(Shareable)]
#[derive(Debug)]
pub enum ServerError {
    Unknown,
}

// TODO #[derive(Shareable)]
#[derive(Debug)]
pub enum ServerResponse {
    Ok(ServerMessage),
    Err(ServerError),
}

impl Into<(JsValue, Option<JsValue>)> for ServerMessage {
    fn into(self) -> (JsValue, Option<JsValue>) {
        match self {
            ServerMessage::Ok => {
                let data = js_sys::Array::new_with_length(1);
                data.set(0, "ok".into());

                (data.into(), None)
            }
        }
    }
}
impl TryFrom<JsValue> for ServerMessage {
    type Error = ShareableError;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        let value: js_sys::Array = value.into();
        let request = value
            .get(0)
            .as_string()
            .expect("Message id should be a string.");

        match request.as_ref() {
            "ok" => Ok(ServerMessage::Ok),
            _ => Err(ShareableError::InvalidIdentifier(request)),
        }
    }
}
impl Shareable for ServerMessage {}

impl Into<(JsValue, Option<JsValue>)> for ServerError {
    fn into(self) -> (JsValue, Option<JsValue>) {
        match self {
            ServerError::Unknown => {
                let data = js_sys::Array::new_with_length(1);
                data.set(0, "unknown".into());

                (data.into(), None)
            }
        }
    }
}
impl TryFrom<JsValue> for ServerError {
    type Error = ShareableError;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        let value: js_sys::Array = value.into();
        let request = value
            .get(0)
            .as_string()
            .expect("Message id should be a string.");

        match request.as_ref() {
            "unknown" => Ok(ServerError::Unknown),
            _ => Err(ShareableError::InvalidIdentifier(request)),
        }
    }
}
impl Shareable for ServerError {}

impl Into<(JsValue, Option<JsValue>)> for ServerResponse {
    fn into(self) -> (JsValue, Option<JsValue>) {
        let payload = js_sys::Array::new_with_length(2);

        match self {
            ServerResponse::Ok(message) => {
                let (data, transfer) = message.into();
                payload.set(0, "ok".into());
                payload.set(1, data);

                (payload.into(), transfer)
            }
            ServerResponse::Err(message) => {
                let (data, transfer) = message.into();
                payload.set(0, "err".into());
                payload.set(1, data);

                (payload.into(), transfer)
            }
        }
    }
}
impl TryFrom<JsValue> for ServerResponse {
    type Error = ShareableError;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        let value: js_sys::Array = value.into();
        let payload_type = value
            .get(0)
            .as_string()
            .expect("Payload type should be a string.");
        let payload = value.get(1);

        match payload_type.as_ref() {
            "ok" => Ok(ServerResponse::Ok(payload.try_into()?)),
            "err" => Ok(ServerResponse::Err(payload.try_into()?)),
            _ => Err(ShareableError::InvalidIdentifier(payload_type)),
        }
    }
}
impl Shareable for ServerResponse {}
