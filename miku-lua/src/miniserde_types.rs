use miniserde::json::{Number as JSONNumber, Value as JSONValue};
use miniserde_miku as miniserde;
use mlua::prelude::*;

pub(crate) struct WrappedJSONValue(JSONValue);

impl From<JSONValue> for WrappedJSONValue {
    fn from(val: JSONValue) -> WrappedJSONValue {
        WrappedJSONValue(val)
    }
}

impl<'lua> ToLua<'lua> for WrappedJSONValue {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(match self.0 {
            JSONValue::Null => LuaValue::Nil,
            JSONValue::Bool(b) => LuaValue::Boolean(b),
            JSONValue::Number(i) => match i {
                JSONNumber::U64(i) => i.to_lua(lua)?,
                JSONNumber::I64(i) => i.to_lua(lua)?,
                JSONNumber::F64(i) => i.to_lua(lua)?,
            },
            JSONValue::String(ref s) => LuaValue::String(lua.create_string(s)?),
            JSONValue::Array(s) => LuaValue::Table(
                lua.create_sequence_from(
                    s.into_iter()
                        .map(|v| WrappedJSONValue(v).to_lua(lua))
                        .collect::<LuaResult<Vec<LuaValue<'lua>>>>()?,
                )?,
            ),
            JSONValue::Object(s) => LuaValue::Table(
                lua.create_table_from(
                    s.into_iter()
                        .map(|(k, v)| Ok((k, WrappedJSONValue(v).to_lua(lua)?)))
                        .collect::<LuaResult<Vec<(String, LuaValue<'lua>)>>>()?,
                )?,
            ),
        })
    }
}
