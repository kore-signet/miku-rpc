use crate::Response;
use miniserde_miku::de::Visitor;
use miniserde_miku::ser::Fragment;
use miniserde_miku::{make_place, Deserialize, Result as MiniserdeResult, Serialize};
use std::borrow::Cow;

make_place!(Place);

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RobotActionResult {
    Incomplete,
    Success,
    Failure,
}

impl Visitor for Place<RobotActionResult> {
    fn string(&mut self, s: &str) -> MiniserdeResult<()> {
        self.out = match s {
            "INCOMPLETE" => Some(RobotActionResult::Incomplete),
            "SUCCESS" => Some(RobotActionResult::Success),
            "FAILURE" => Some(RobotActionResult::Failure),
            _ => None,
        };

        Ok(())
    }
}

impl Deserialize for RobotActionResult {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        Place::new(out)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MoveDirection {
    Forward,
    Backward,
    Upward,
    Downward,
    Left,
    Right,
}

impl AsRef<str> for MoveDirection {
    fn as_ref(&self) -> &'static str {
        use MoveDirection::*;

        match self {
            Forward => "forward",
            Backward => "backward",
            Upward => "upward",
            Downward => "downward",
            Left => "left",
            Right => "right",
        }
    }
}

impl Serialize for MoveDirection {
    fn begin(&self) -> Fragment {
        Fragment::Str(Cow::Borrowed(self.as_ref()))
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RotationDirection {
    Left,
    Right,
}

impl Serialize for RotationDirection {
    fn begin(&self) -> Fragment {
        Fragment::Str(Cow::Borrowed(self.as_ref()))
    }
}

impl AsRef<str> for RotationDirection {
    fn as_ref(&self) -> &'static str {
        use RotationDirection::*;

        match self {
            Left => "left",
            Right => "right",
        }
    }
}
