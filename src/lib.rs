pub mod bus;
pub mod types;
use miniserde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RPCMessage<T: Serialize + Deserialize> {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: T,
}

impl RPCMessage<()> {
    pub fn list() -> RPCMessage<()> {
        RPCMessage {
            msg_type: String::from("list"),
            data: (),
        }
    }
}
