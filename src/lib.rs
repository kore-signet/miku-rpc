pub mod bus;
pub mod types;
use miniserde::{Deserialize, Serialize};

#[derive(Serialize)]
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
pub struct InvokeCall<'a> {
    #[serde(rename = "deviceId")]
    device_id: &'a str,
    #[serde(rename = "name")]
    method_name: &'a str,
    parameters: Vec<&'a dyn Serialize>,
}

impl Call<'_, InvokeCall<'_>> {
    pub fn invoke<'a>(
        device_id: &'a str,
        method_name: &'a str,
        parameters: Vec<&'a dyn Serialize>,
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
pub struct Response<T: Deserialize> {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: T,
}
