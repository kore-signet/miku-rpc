use crate::Response;
use miniserde::{Deserialize, Serialize};

pub type DeviceList = Response<Vec<DeviceData>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceData {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "typeNames")]
    pub type_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImportFileInfo {
    pub name: String,
    pub size: u64,
}
