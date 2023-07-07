use port::{Shareable, ShareableError};
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
impl<T> TryFrom<JsValue> for Payload<T>
where
    T: Shareable,
{
    type Error = ShareableError;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        let value: js_sys::Array = value.into();
        let id: js_sys::Number = value.get(0).into();
        let message = value.get(1);

        Ok(Self {
            id,
            message: message.try_into()?,
        })
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
    use web_sys::{OffscreenCanvas, Worker};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_worker);

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub enum PlainEnum {
        Ping,
    }

    #[wasm_bindgen_test]
    fn plain_enum() {
        let (data, transfer) = PlainEnum::Ping.into();
        let recovered: Result<PlainEnum, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(recovered, PlainEnum::Ping);
        assert_eq!(transfer, None);
    }

    #[wasm_bindgen_test]
    fn invalid_ident() {
        let payload = js_sys::Array::new();
        payload.push(&"invalid".into());
        let payload: JsValue = payload.into();

        let recovered: Result<PlainEnum, _> = payload.try_into();
        assert!(recovered.is_err())
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub enum TupleEnum {
        Ping,
        Attach(OffscreenCanvas),
        Wrap(Worker, OffscreenCanvas),
    }

    #[wasm_bindgen_test]
    fn value_enum() {
        let value = OffscreenCanvas::new(0, 0).unwrap();
        let (data, transfer) = TupleEnum::Attach(value.clone()).into();
        let recovered: Result<TupleEnum, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(recovered, TupleEnum::Attach(value));
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub enum StructEnum {
        Wrap {
            server: OffscreenCanvas,
            surface: OffscreenCanvas,
        },
    }

    #[wasm_bindgen_test]
    fn struct_enum() {
        let value_a = OffscreenCanvas::new(0, 0).unwrap();
        let value_b = OffscreenCanvas::new(0, 0).unwrap();
        let (data, transfer) = StructEnum::Wrap {
            server: value_a.clone(),
            surface: value_b.clone(),
        }
        .into();
        let recovered: Result<StructEnum, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(
            recovered,
            StructEnum::Wrap {
                server: value_a,
                surface: value_b
            }
        );
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub enum AttrEnum {
        Draw(#[shareable(transfer)] OffscreenCanvas),
        Wrap {
            server: OffscreenCanvas,
            #[shareable(transfer)]
            surface: OffscreenCanvas,
        },
    }

    #[wasm_bindgen_test]
    fn attr_enum_tuple() {
        let value_a = OffscreenCanvas::new(0, 0).unwrap();
        let (_, transfer) = AttrEnum::Draw(value_a.clone()).into();

        assert!(transfer.is_some());
        let transfer: js_sys::Array = transfer.unwrap().into();
        let recovered: OffscreenCanvas = transfer.get(0).into();
        assert_eq!(recovered, value_a);
    }

    #[wasm_bindgen_test]
    fn attr_enum_struct() {
        let value_a = OffscreenCanvas::new(0, 0).unwrap();
        let value_b = OffscreenCanvas::new(0, 0).unwrap();

        let (_, transfer) = AttrEnum::Wrap {
            server: value_a.clone(),
            surface: value_b.clone(),
        }
        .into();

        assert!(transfer.is_some());
        let transfer: js_sys::Array = transfer.unwrap().into();
        let recovered: OffscreenCanvas = transfer.get(0).into();
        assert_eq!(recovered, value_b);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub struct PlainStruct;

    #[wasm_bindgen_test]
    fn plain_struct() {
        let (data, transfer) = PlainStruct.into();
        let recovered: Result<PlainStruct, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(recovered, PlainStruct);
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub struct TupleStruct(OffscreenCanvas);

    #[wasm_bindgen_test]
    fn tuple_struct() {
        let value = OffscreenCanvas::new(0, 0).unwrap();
        let (data, transfer) = TupleStruct(value.clone()).into();
        let recovered: Result<TupleStruct, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(recovered, TupleStruct(value));
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub struct StructStruct {
        worker: OffscreenCanvas,
        canvas: OffscreenCanvas,
    }

    #[wasm_bindgen_test]
    fn struct_struct() {
        let value_a = OffscreenCanvas::new(0, 0).unwrap();
        let value_b = OffscreenCanvas::new(0, 0).unwrap();
        let (data, transfer) = StructStruct {
            worker: value_a.clone(),
            canvas: value_b.clone(),
        }
        .into();
        let recovered: Result<StructStruct, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(
            recovered,
            StructStruct {
                worker: value_a,
                canvas: value_b
            }
        );
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub struct AttrTupleStruct(#[shareable(transfer)] OffscreenCanvas);

    #[wasm_bindgen_test]
    fn attr_tuple_struct() {
        let value_a = OffscreenCanvas::new(0, 0).unwrap();
        let (_, transfer) = AttrTupleStruct(value_a.clone()).into();

        assert!(transfer.is_some());
        let transfer: js_sys::Array = transfer.unwrap().into();
        let recovered: OffscreenCanvas = transfer.get(0).into();
        assert_eq!(recovered, value_a);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub struct AttrStructStruct {
        worker: OffscreenCanvas,
        #[shareable(transfer)]
        canvas: OffscreenCanvas,
    }

    #[wasm_bindgen_test]
    fn attr_struct_struct() {
        let value_a = OffscreenCanvas::new(0, 0).unwrap();
        let value_b = OffscreenCanvas::new(0, 0).unwrap();

        let (_, transfer) = AttrStructStruct {
            worker: value_a.clone(),
            canvas: value_b.clone(),
        }
        .into();

        assert!(transfer.is_some());
        let transfer: js_sys::Array = transfer.unwrap().into();
        let recovered: OffscreenCanvas = transfer.get(0).into();
        assert_eq!(recovered, value_b);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub enum SerdeTupleEnum {
        Message(
            #[shareable(repr = "serde")] String,
            #[shareable(repr = "serde")] Option<u32>,
        ),
        Ping,
    }

    #[wasm_bindgen_test]
    fn serde_tuple_enum() {
        let (data, transfer) = SerdeTupleEnum::Message("voxelstack.me".into(), Some(314)).into();
        let recovered: Result<SerdeTupleEnum, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(
            recovered,
            SerdeTupleEnum::Message("voxelstack.me".into(), Some(314))
        );
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub enum SerdeStructEnum {
        Message {
            #[shareable(repr = "serde")]
            key: String,
            #[shareable(repr = "serde")]
            value: Option<u32>,
        },
        Ping,
    }

    #[wasm_bindgen_test]
    fn serde_struct_enum() {
        let (data, transfer) = SerdeStructEnum::Message {
            key: "voxelstack.me".into(),
            value: Some(314),
        }
        .into();
        let recovered: Result<SerdeStructEnum, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(
            recovered,
            SerdeStructEnum::Message {
                key: "voxelstack.me".into(),
                value: Some(314),
            }
        );
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub struct SerdeTupleStruct(
        #[shareable(repr = "serde")] String,
        #[shareable(repr = "serde")] Option<u32>,
        OffscreenCanvas,
    );

    #[wasm_bindgen_test]
    fn serde_tuple_struct() {
        let value_a = OffscreenCanvas::new(0, 0).unwrap();
        let (data, transfer) =
            SerdeTupleStruct("voxelstack.me".into(), Some(314), value_a.clone()).into();
        let recovered: Result<SerdeTupleStruct, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(
            recovered,
            SerdeTupleStruct("voxelstack.me".into(), Some(314), value_a)
        );
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    pub struct SerdeStructStruct {
        #[shareable(repr = "serde")]
        key: String,
        #[shareable(repr = "serde")]
        value: Option<u32>,
    }

    #[wasm_bindgen_test]
    fn serde_struct_struct() {
        let (data, transfer) = SerdeStructStruct {
            key: "voxelstack.me".into(),
            value: Some(314),
        }
        .into();
        let recovered: Result<SerdeStructStruct, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(
            recovered,
            SerdeStructStruct {
                key: "voxelstack.me".into(),
                value: Some(314),
            }
        );
        assert_eq!(transfer, None);
    }
}
