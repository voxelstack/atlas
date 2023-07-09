use atlas_comms_derive::Shareable;
use port::Shareable;
use std::panic;
use wasm_bindgen::prelude::*;

pub mod client;
pub mod port;
pub mod server;

#[derive(Debug, Shareable)]
pub struct Payload<T>
where
    T: Shareable,
{
    #[shareable(repr = "serde")]
    pub id: u8,
    pub message: T,
}

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
    use crate::port::ShareableError;

    use super::*;
    use atlas_comms_derive::Shareable;
    use std::fmt::Debug;
    use wasm_bindgen_test::*;
    use web_sys::{OffscreenCanvas, Worker};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_worker);

    #[derive(Debug, PartialEq, Eq, Shareable)]
    enum PlainEnum {
        Ping,
    }

    #[wasm_bindgen_test]
    fn plain_enum() {
        let (data, transfer) = PlainEnum::Ping.try_into().unwrap();
        let recovered: Result<PlainEnum, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(recovered, PlainEnum::Ping);
        assert_eq!(transfer, None);
    }

    #[wasm_bindgen_test]
    fn invalid_ident() {
        let payload = js_sys::Array::new();
        payload.push(&"PlainEnum".into());
        payload.push(&"invalid".into());
        let payload: JsValue = payload.into();

        let recovered: Result<PlainEnum, _> = payload.try_into();
        assert!(recovered.is_err())
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    enum TupleEnum {
        Ping,
        Attach(#[shareable(repr = "raw")] OffscreenCanvas),
        Wrap(
            #[shareable(repr = "raw")] Worker,
            #[shareable(repr = "raw")] OffscreenCanvas,
        ),
    }

    #[wasm_bindgen_test]
    fn value_enum() {
        let value = OffscreenCanvas::new(0, 0).unwrap();
        let (data, transfer) = TupleEnum::Attach(value.clone()).try_into().unwrap();
        let recovered: Result<TupleEnum, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(recovered, TupleEnum::Attach(value));
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    enum StructEnum {
        Wrap {
            #[shareable(repr = "raw")]
            server: OffscreenCanvas,
            #[shareable(repr = "raw")]
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
        .try_into()
        .unwrap();
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
    enum AttrEnum {
        Draw(#[shareable(repr = "raw", transfer)] OffscreenCanvas),
        Wrap {
            #[shareable(repr = "raw")]
            server: OffscreenCanvas,
            #[shareable(repr = "raw", transfer)]
            surface: OffscreenCanvas,
        },
    }

    #[wasm_bindgen_test]
    fn attr_enum_tuple() {
        let value_a = OffscreenCanvas::new(0, 0).unwrap();
        let (_, transfer) = AttrEnum::Draw(value_a.clone()).try_into().unwrap();

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
        .try_into()
        .unwrap();

        assert!(transfer.is_some());
        let transfer: js_sys::Array = transfer.unwrap().into();
        let recovered: OffscreenCanvas = transfer.get(0).into();
        assert_eq!(recovered, value_b);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    struct PlainStruct;

    #[wasm_bindgen_test]
    fn plain_struct() {
        let (data, transfer) = PlainStruct.try_into().unwrap();
        let recovered: Result<PlainStruct, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(recovered, PlainStruct);
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    struct TupleStruct(#[shareable(repr = "raw")] OffscreenCanvas);

    #[wasm_bindgen_test]
    fn tuple_struct() {
        let value = OffscreenCanvas::new(0, 0).unwrap();
        let (data, transfer) = TupleStruct(value.clone()).try_into().unwrap();
        let recovered: Result<TupleStruct, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(recovered, TupleStruct(value));
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    struct StructStruct {
        #[shareable(repr = "raw")]
        worker: OffscreenCanvas,
        #[shareable(repr = "raw")]
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
        .try_into()
        .unwrap();
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
    struct AttrTupleStruct(#[shareable(repr = "raw", transfer)] OffscreenCanvas);

    #[wasm_bindgen_test]
    fn attr_tuple_struct() {
        let value_a = OffscreenCanvas::new(0, 0).unwrap();
        let (_, transfer) = AttrTupleStruct(value_a.clone()).try_into().unwrap();

        assert!(transfer.is_some());
        let transfer: js_sys::Array = transfer.unwrap().into();
        let recovered: OffscreenCanvas = transfer.get(0).into();
        assert_eq!(recovered, value_a);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    struct AttrStructStruct {
        #[shareable(repr = "raw")]
        worker: OffscreenCanvas,
        #[shareable(repr = "raw", transfer)]
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
        .try_into()
        .unwrap();

        assert!(transfer.is_some());
        let transfer: js_sys::Array = transfer.unwrap().into();
        let recovered: OffscreenCanvas = transfer.get(0).into();
        assert_eq!(recovered, value_b);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    enum SerdeTupleEnum {
        Message(
            #[shareable(repr = "serde")] String,
            #[shareable(repr = "serde")] Option<u32>,
        ),
        Ping,
    }

    #[wasm_bindgen_test]
    fn serde_tuple_enum() {
        let (data, transfer) = SerdeTupleEnum::Message("voxelstack.me".into(), Some(314))
            .try_into()
            .unwrap();
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
    enum SerdeStructEnum {
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
        .try_into()
        .unwrap();
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
    struct SerdeTupleStruct(
        #[shareable(repr = "serde")] String,
        #[shareable(repr = "serde")] Option<u32>,
        #[shareable(repr = "raw")] OffscreenCanvas,
    );

    #[wasm_bindgen_test]
    fn serde_tuple_struct() {
        let value_a = OffscreenCanvas::new(0, 0).unwrap();
        let (data, transfer) = SerdeTupleStruct("voxelstack.me".into(), Some(314), value_a.clone())
            .try_into()
            .unwrap();
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
    struct SerdeStructStruct {
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
        .try_into()
        .unwrap();
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

    #[derive(Debug, PartialEq, Eq, Shareable)]
    struct Generic<T>
    where
        T: Into<JsValue> + From<JsValue> + Debug,
    {
        #[shareable(repr = "raw")]
        value: T,
    }

    #[wasm_bindgen_test]
    fn generic() {
        let value = OffscreenCanvas::new(0, 0).unwrap();
        let (data, transfer) = Generic {
            value: value.clone(),
        }
        .try_into()
        .unwrap();
        let recovered: Result<Generic<OffscreenCanvas>, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(recovered, Generic { value });
        assert_eq!(transfer, None);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    struct Child {
        #[shareable(repr = "serde")]
        id: String,
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    enum Parent {
        Ping,
        Attach(Child),
    }

    #[wasm_bindgen_test]
    fn nested() {
        let (data, _) = Parent::Attach(Child {
            id: "surface".into(),
        })
        .try_into()
        .unwrap();
        let recovered: Result<Parent, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(
            recovered,
            Parent::Attach(Child {
                id: "surface".into(),
            })
        );
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    struct ChildTransfer {
        #[shareable(repr = "serde")]
        id: String,
        #[shareable(repr = "raw", transfer)]
        canvas: OffscreenCanvas,
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    enum ParentTransfer {
        Ping,
        Attach(ChildTransfer),
        Transfer(
            ChildTransfer,
            #[shareable(repr = "raw", transfer)] OffscreenCanvas,
        ),
    }

    #[wasm_bindgen_test]
    fn nested_transfer() {
        let value = OffscreenCanvas::new(0, 0).unwrap();
        let (data, transfer) = ParentTransfer::Attach(ChildTransfer {
            id: "surface".into(),
            canvas: value.clone(),
        })
        .try_into()
        .unwrap();
        let recovered: Result<ParentTransfer, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(
            recovered,
            ParentTransfer::Attach(ChildTransfer {
                id: "surface".into(),
                canvas: value.clone()
            })
        );
        assert!(transfer.is_some());
        let transfer: js_sys::Array = transfer.unwrap().into();
        let recovered: OffscreenCanvas = transfer.get(0).into();
        assert_eq!(recovered, value);
    }

    #[wasm_bindgen_test]
    fn nested_transfer_multiple() {
        let value_a = OffscreenCanvas::new(0, 0).unwrap();
        let value_b = OffscreenCanvas::new(0, 0).unwrap();
        let (data, transfer) = ParentTransfer::Transfer(
            ChildTransfer {
                id: "surface".into(),
                canvas: value_a.clone(),
            },
            value_b.clone(),
        )
        .try_into()
        .unwrap();
        let recovered: Result<ParentTransfer, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(
            recovered,
            ParentTransfer::Transfer(
                ChildTransfer {
                    id: "surface".into(),
                    canvas: value_a.clone()
                },
                value_b.clone()
            )
        );
        assert!(transfer.is_some());
        let transfer: js_sys::Array = transfer.unwrap().into();
        let recovered: OffscreenCanvas = transfer.get(0).into();
        assert_eq!(recovered, value_a);
        let recovered: OffscreenCanvas = transfer.get(1).into();
        assert_eq!(recovered, value_b);
    }

    #[derive(Debug, PartialEq, Eq, Shareable)]
    struct ParentGeneric<T>
    where
        T: Shareable,
    {
        value: T,
    }

    #[wasm_bindgen_test]
    fn nested_generic() {
        let (data, transfer) = ParentGeneric {
            value: Child {
                id: "surface".into(),
            },
        }
        .try_into()
        .unwrap();
        let recovered: Result<ParentGeneric<Child>, _> = data.try_into();

        assert!(recovered.is_ok());
        let recovered = recovered.unwrap();

        assert_eq!(
            recovered,
            ParentGeneric {
                value: Child {
                    id: "surface".into()
                }
            }
        );
        assert_eq!(transfer, None);
    }

    #[wasm_bindgen_test]
    fn incompatible_type() {
        let (data, _) = PlainEnum::Ping.try_into().unwrap();
        let recovered: Result<PlainStruct, _> = data.try_into();

        assert!(recovered.is_err());
        assert_eq!(recovered, Err(ShareableError::IncompatibleType));
    }
}
