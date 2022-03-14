use miku_rpc::DeviceBus;
use mlua::prelude::*;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

mod miniserde_types;

pub struct BusRegistry(HashMap<PathBuf, Rc<Mutex<DeviceBus>>>);

pub struct BusCreator;

impl LuaUserData for BusCreator {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("new", |lua, _, path: String| {
            let mut registry = {
                if let Some(r) = lua.app_data_mut::<BusRegistry>() {
                    r
                } else {
                    lua.set_app_data(BusRegistry(HashMap::new()));
                    lua.app_data_mut::<BusRegistry>().unwrap()
                }
            };

            let path = PathBuf::from(path).canonicalize()?;
            if !registry.0.contains_key(&path) {
                registry
                    .0
                    .insert(path.clone(), Rc::new(Mutex::new(DeviceBus::new(&path)?)));
            }

            Ok(BusHandle(Rc::clone(&registry.0[&path])))
        })
    }
}

#[derive(Clone)]
pub struct BusHandle(Rc<Mutex<DeviceBus>>);

impl BusHandle {
    #[inline(always)]
    fn make_call<T: Serialize>(&self, call: &T) -> LuaResult<miniserde_types::WrappedJSONValue> {
        let mut bus = self.0.lock();
        let mut serialized = serde_json::to_vec(call).map_err(LuaError::external)?;
        let mut serialized_call = Vec::with_capacity(serialized.len() + 2);
        serialized_call.push(0);
        serialized_call.append(&mut serialized);
        serialized_call.push(0);

        bus.call_preserialized::<miniserde_miku::json::Value>(&serialized_call)
            .map(|v| miniserde_types::WrappedJSONValue::from(v.data))
            .map_err(LuaError::external)
    }
}

impl LuaUserData for BusHandle {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "invoke",
            |_,
             this,
             (device_id, method_name, values): (
                LuaValue<'lua>,
                LuaValue<'lua>,
                LuaMultiValue<'lua>,
            )| {
                let call = Call {
                    msg_type: "invoke",
                    data: FullLuaInvoke {
                        device_id: device_id,
                        method_name: method_name,
                        parameters: values.into_vec(),
                    },
                };

                this.make_call(&call)
            },
        );

        methods.add_method("find", |_, this, name: String| {
            let mut bus = this.0.lock();
            if let Some(id) = bus.find(&name).map_err(LuaError::external)? {
                Ok(Some(DeviceHandle {
                    id,
                    handle: this.clone(),
                }))
            } else {
                Ok(None)
            }
        });

        methods.add_method("list", |_, this, _: ()| {
            let mut bus = this.0.lock();

            Ok(bus
                .call_preserialized::<miniserde_miku::json::Value>(b"\0{\"type\":\"list\"}\0")
                .map(|v| miniserde_types::WrappedJSONValue::from(v.data))
                .unwrap())
        });
    }
}

pub struct DeviceHandle {
    id: String,
    handle: BusHandle,
}

impl DeviceHandle {
    // function to return for calls
    fn raw_invoke(
        lua: &Lua,
        (method_name, possibly_this, values): (String, LuaAnyUserData<'_>, LuaMultiValue<'_>),
    ) -> LuaResult<miniserde_types::WrappedJSONValue> {
        let this = possibly_this.borrow::<DeviceHandle>()?;

        let call = Call {
            msg_type: "invoke",
            data: Invoke {
                device_id: &this.id,
                method_name: method_name.to_lua(lua)?,
                parameters: values.into_vec(),
            },
        };

        this.handle.make_call(&call)
    }
}

impl LuaUserData for DeviceHandle {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "invoke",
            |_, this, (method_name, values): (LuaValue<'lua>, LuaMultiValue<'lua>)| {
                let call = Call {
                    msg_type: "invoke",
                    data: Invoke {
                        device_id: &this.id,
                        method_name: method_name,
                        parameters: values.into_vec(),
                    },
                };

                this.handle.make_call(&call)
            },
        );

        methods.add_meta_method(LuaMetaMethod::Index, |lua, _, method_name: String| {
            lua.create_function(DeviceHandle::raw_invoke)?
                .bind(method_name)
        });
    }
}

#[derive(Serialize, Deserialize)]
pub struct Call<T: serde::Serialize> {
    #[serde(rename = "type")]
    msg_type: &'static str,
    data: T,
}

#[derive(Serialize)]
pub struct FullLuaInvoke<'a> {
    #[serde(rename = "deviceId")]
    device_id: LuaValue<'a>,
    #[serde(rename = "name")]
    method_name: LuaValue<'a>,
    parameters: Vec<LuaValue<'a>>,
}

#[derive(Serialize)]
pub struct Invoke<'a, 'b> {
    #[serde(rename = "deviceId")]
    device_id: &'b str,
    #[serde(rename = "name")]
    method_name: LuaValue<'a>,
    parameters: Vec<LuaValue<'a>>,
}

#[mlua::lua_module]
fn libmiku(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    Ok(lua.create_userdata(BusCreator)?)
}
