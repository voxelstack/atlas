use port::Shareable;
use std::panic;
use wasm_bindgen::prelude::*;

pub mod client;
pub mod port;
pub mod server;

#[derive(Debug)]
pub struct Payload<T>
where
    T: Shareable,
{
    pub id: js_sys::Number,
    pub message: T,
}

impl<T> Into<(JsValue, Option<JsValue>)> for Payload<T>
where
    T: Shareable,
{
    fn into(self) -> (JsValue, Option<JsValue>) {
        let payload = js_sys::Array::new_with_length(2);
        let (data, transfer) = self.message.into();
        payload.set(0, JsValue::from(self.id));
        payload.set(1, data);

        (payload.into(), transfer)
    }
}
impl<T> From<JsValue> for Payload<T>
where
    T: Shareable,
{
    fn from(value: JsValue) -> Self {
        let value: js_sys::Array = value.into();
        let id: js_sys::Number = value.get(0).into();
        let message = value.get(1);

        Self {
            id,
            message: message.into(),
        }
    }
}
impl<T> Shareable for Payload<T> where T: Shareable {}

#[wasm_bindgen(js_name = initOutput)]
pub fn init_output() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    #[cfg(feature = "loggers")]
    {
        if let Err(_) = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{} [{} {}:{}]",
                    message,
                    record.level(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                ))
            })
            .level(log::LevelFilter::Trace)
            .chain(fern::Output::call(console_log::log))
            .apply()
        {
            web_sys::console::warn_1(&"Failed to initialize loggers.".into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_comms_derive::Shareable;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_worker);

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub enum Message {
        Ping,
    }

    #[wasm_bindgen_test]
    fn plain_enum() {
        let (data, transfer) = Message::Ping.into();
        let recovered: Message = data.into();

        assert_eq!(recovered, Message::Ping);
        assert_eq!(transfer, None);
    }

    #[wasm_bindgen_test]
    #[should_panic]
    fn invalid_ident() {
        let _: Message = JsValue::from("invalid").into();
    }
}
