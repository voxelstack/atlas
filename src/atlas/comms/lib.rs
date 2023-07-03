use port::Shareable;
use std::panic;
use wasm_bindgen::prelude::*;

pub mod port;

pub struct Payload(pub String);
impl Into<(JsValue, Option<JsValue>)> for Payload {
    fn into(self) -> (JsValue, Option<JsValue>) {
        (JsValue::from(self.0), None)
    }
}
impl From<JsValue> for Payload {
    fn from(value: JsValue) -> Self {
        Self(value.as_string().unwrap())
    }
}
impl Shareable for Payload {}

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
