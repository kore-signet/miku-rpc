//! A crate implementing the OpenComputers 2 HLApi interface.

mod bus;
pub use bus::DeviceBus;

/// Type definitions for commonly used responses.
pub mod types;
/// Wrappers around specific HLApi devices and their methods.
#[cfg(feature = "wrappers")]
pub mod wrappers;
use miniserde_miku::{Deserialize, Serialize};

#[derive(Serialize)]
/// A HLApi call, composed of a type and some json serializable data.
pub struct Call<'a, T: Serialize> {
    #[serde(rename = "type")]
    pub msg_type: &'a str,
    pub data: T,
}

impl Call<'_, ()> {
    pub fn list() -> Call<'static, ()> {
        Call {
            msg_type: "list",
            data: (),
        }
    }
}

impl Call<'_, &str> {
    pub fn methods(device_id: &str) -> Call<'static, &str> {
        Call {
            msg_type: "methods",
            data: device_id,
        }
    }
}

#[derive(Serialize)]
/// A HLApi call that invokes a method on a specific device.
pub struct InvokeCall<'a> {
    #[serde(rename = "deviceId")]
    device_id: &'a str,
    #[serde(rename = "name")]
    method_name: &'a str,
    parameters: &'a [&'a dyn Serialize],
}

impl Call<'_, InvokeCall<'_>> {
    pub fn invoke<'a>(
        device_id: &'a str,
        method_name: &'a str,
        parameters: &'a [&'a dyn Serialize],
    ) -> Call<'static, InvokeCall<'a>> {
        Call {
            msg_type: "invoke",
            data: InvokeCall {
                device_id,
                method_name,
                parameters,
            },
        }
    }
}

#[derive(Deserialize)]
/// The response to a HLApi call.
pub struct Response<T: Deserialize> {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: T,
}
