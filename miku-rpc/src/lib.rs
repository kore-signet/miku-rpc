//! A crate implementing the OpenComputers 2 HLApi interface.

mod bus;
pub use bus::DeviceBus;

/// Type definitions for commonly used responses.
pub mod types;
/// Wrappers around specific HLApi devices and their methods.
#[cfg(feature = "wrappers")]
pub mod wrappers;
use miniserde_miku::Serialize;

mod response;
pub use response::*;

#[derive(Serialize)]
/// A HLApi call, composed of a type and some json serializable data.
pub struct Call<T: Serialize> {
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub data: T,
}

impl Call<()> {
    pub fn list() -> Call<()> {
        Call {
            msg_type: MessageType::List,
            data: (),
        }
    }
}

impl Call<&str> {
    pub fn methods(device_id: &str) -> Call<&str> {
        Call {
            msg_type: MessageType::Methods,
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

impl Call<InvokeCall<'_>> {
    pub fn invoke<'a>(
        device_id: &'a str,
        method_name: &'a str,
        parameters: &'a [&'a dyn Serialize],
    ) -> Call<InvokeCall<'a>> {
        Call {
            msg_type: MessageType::Invoke,
            data: InvokeCall {
                device_id,
                method_name,
                parameters,
            },
        }
    }
}
