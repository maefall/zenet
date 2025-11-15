mod codec;
pub mod errors;

pub use codec::FrameCodec;
use errors::WireError;
use tokio_util::bytes::Bytes;

// [1:message_type 2:payload_length payload_length:payload]
#[derive(Debug, Clone)]
pub struct Frame {
    pub message_type: MessageType,
    pub payload: Bytes,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Auth = 1,
    AuthValid = 2,
    AuthInvalid = 3,
}

impl TryFrom<u8> for MessageType {
    type Error = WireError;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            1 => Ok(Self::Auth),
            2 => Ok(Self::AuthValid),
            3 => Ok(Self::AuthInvalid),
            _ => Err(WireError::InvalidMessageType(code)),
        }
    }
}
