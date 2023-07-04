use crate::port::Shareable;
use wasm_bindgen::JsValue;
use web_sys::OffscreenCanvas;

// TODO #[derive(Shareable)]
#[derive(Debug)]
pub enum ClientMessage {
    Ping,
    Attach(OffscreenCanvas),
}

impl Into<(JsValue, Option<JsValue>)> for ClientMessage {
    fn into(self) -> (JsValue, Option<JsValue>) {
        match self {
            ClientMessage::Ping => {
                let data = js_sys::Array::new_with_length(1);
                data.set(0, "ping".into());

                (data.into(), None)
            }
            ClientMessage::Attach(canvas) => {
                let data = js_sys::Array::new_with_length(2);
                data.set(0, "attach".into());
                data.set(1, canvas.clone().into());

                let transfer = js_sys::Array::new_with_length(1);
                transfer.set(0, canvas.into());

                (data.into(), Some(transfer.into()))
            }
        }
    }
}

impl From<JsValue> for ClientMessage {
    fn from(value: JsValue) -> Self {
        let value: js_sys::Array = value.into();
        let request = value
            .get(0)
            .as_string()
            .expect("Message id should be a string.");

        match request.as_ref() {
            "ping" => ClientMessage::Ping,
            "attach" => {
                let canvas = value.get(1).into();
                ClientMessage::Attach(canvas)
            }
            _ => panic!("Message id should be valid."),
        }
    }
}

impl Shareable for ClientMessage {}
