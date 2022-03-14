use miniserde_miku as miniserde;

use miniserde::de::{Deserialize, Map, Visitor};
use miniserde::ser::{Fragment, Serialize};
use miniserde::{make_place, Result as MiniserdeResult};

use std::borrow::Cow;
use std::fmt;

make_place!(Place);

pub type RPCResult<T> = std::result::Result<Response<T>, RPCError>;

pub(crate) enum WrappedRPCResult<T: Deserialize> {
    Ok(Response<T>),
    Err(RPCError),
}

impl<T: Deserialize> Into<RPCResult<T>> for WrappedRPCResult<T> {
    fn into(self) -> RPCResult<T> {
        match self {
            WrappedRPCResult::Ok(v) => Result::Ok(v),
            WrappedRPCResult::Err(e) => Result::Err(e),
        }
    }
}

/// The response to a HLApi call.
pub struct Response<T: Deserialize> {
    pub msg_type: MessageType,
    pub data: T,
}

impl<T: Deserialize + fmt::Debug> fmt::Debug for Response<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("type", &self.msg_type)
            .field("data", &self.data)
            .finish()
    }
}

pub enum MessageType {
    List,
    Methods,
    Result,
    Error,
    Invoke, // never received, only sent
}

impl Visitor for Place<MessageType> {
    fn string(&mut self, b: &str) -> MiniserdeResult<()> {
        self.out = Some(match b {
            "list" => MessageType::List,
            "methods" => MessageType::Methods,
            "result" => MessageType::Result,
            "error" => MessageType::Error,
            "invoke" => MessageType::Invoke,
            _ => return MiniserdeResult::Err(miniserde::Error),
        });

        Ok(())
    }
}

impl Deserialize for MessageType {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        Place::new(out)
    }
}

impl AsRef<str> for MessageType {
    fn as_ref(&self) -> &'static str {
        match self {
            MessageType::List => "list",
            MessageType::Methods => "methods",
            MessageType::Result => "result",
            MessageType::Error => "error",
            MessageType::Invoke => "invoke",
        }
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.as_ref())
    }
}

impl fmt::Debug for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Serialize for MessageType {
    fn begin(&self) -> Fragment {
        Fragment::Str(Cow::Borrowed(self.as_ref()))
    }
}

pub enum RPCError {
    MessageTooLarge,
    UnknownMessageType,
    UnknownDevice,
    UnknownMethod,
    InvalidParameterSignature,
    Other(String),
}

impl Visitor for Place<RPCError> {
    fn string(&mut self, b: &str) -> MiniserdeResult<()> {
        self.out = Some(match b {
            "message too large" => RPCError::MessageTooLarge,
            "unknown message type" => RPCError::UnknownMessageType,
            "unknown device" => RPCError::UnknownDevice,
            "unknown method" => RPCError::UnknownMethod,
            "invalid parameter signature" => RPCError::InvalidParameterSignature,
            _ => RPCError::Other(b.to_owned()),
        });

        Ok(())
    }
}

impl Deserialize for RPCError {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        Place::new(out)
    }
}

impl AsRef<str> for RPCError {
    fn as_ref(&self) -> &str {
        match self {
            RPCError::MessageTooLarge => "message too large",
            RPCError::UnknownMessageType => "unknown message type",
            RPCError::UnknownDevice => "unknown device",
            RPCError::UnknownMethod => "unknown method",
            RPCError::InvalidParameterSignature => "invalid parameter signature",
            RPCError::Other(ref s) => s,
        }
    }
}

impl fmt::Display for RPCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.as_ref())
    }
}

impl fmt::Debug for RPCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for RPCError {}

struct ResponseBuilder<'a, T: Deserialize> {
    msg_type: Option<MessageType>,
    error: Option<RPCError>,
    data: Option<T>,
    out: &'a mut Option<WrappedRPCResult<T>>,
}

impl<'a, T: Deserialize> Map for ResponseBuilder<'a, T> {
    fn key(&mut self, k: &str) -> MiniserdeResult<&mut dyn Visitor> {
        match k {
            "type" => Ok(Deserialize::begin(&mut self.msg_type)),
            "data" => {
                if let Some(MessageType::Error) = self.msg_type {
                    Ok(Deserialize::begin(&mut self.error))
                } else {
                    Ok(Deserialize::begin(&mut self.data))
                }
            }
            _ => Ok(<dyn Visitor>::ignore()),
        }
    }

    fn finish(&mut self) -> MiniserdeResult<()> {
        if let Some(err) = self.error.take() {
            *self.out = Some(WrappedRPCResult::Err(err));

            Ok(())
        } else if let Some(data) = self.data.take() {
            *self.out = Some(WrappedRPCResult::Ok(Response {
                msg_type: self.msg_type.take().ok_or(miniserde::Error)?,
                data,
            }));

            Ok(())
        } else if let Some(MessageType::Result) = self.msg_type {
            // this is a hack in the case that the response has no data field but a success, in which case we can imply that this is an option<T>, hopefully
            *self.out = Some(WrappedRPCResult::Ok(Response {
                msg_type: MessageType::Result,
                data: <T as Deserialize>::default().ok_or(miniserde::Error)?,
            }));

            Ok(())
        } else {
            Err(miniserde::Error)
        }
    }
}

impl<T: Deserialize> Visitor for Place<WrappedRPCResult<T>> {
    fn map(&mut self) -> MiniserdeResult<Box<dyn Map + '_>> {
        Ok(Box::new(ResponseBuilder {
            msg_type: None,
            error: None,
            data: None,
            out: &mut self.out,
        }))
    }
}

impl<T: Deserialize> Deserialize for WrappedRPCResult<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        Place::new(out)
    }
}
