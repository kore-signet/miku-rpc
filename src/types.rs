use crate::RPCMessage;
use miniserde::{Deserialize, Serialize};

pub type DeviceList = RPCMessage<Vec<DeviceData>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceData {
    #[serde(rename = "deviceId")]
    device_id: String,
    #[serde(rename = "typeNames")]
    type_names: Vec<String>,
}
