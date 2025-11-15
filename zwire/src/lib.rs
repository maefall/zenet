mod codec;
pub mod errors;

pub use codec::FrameCodec;
use errors::WireError;
use tokio_util::{
    bytes::{Bytes, BytesMut},
    codec::{Decoder, Encoder},
};

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

pub trait EncodeIntoFrame: Encoder<Self::EncodeItem> {
    type EncodeItem;

    fn encode_into_frame(
        &mut self,
        payload: Self::EncodeItem,
        message_type: MessageType,
        codec_buffer: &mut BytesMut,
    ) -> Result<Frame, Self::Error> {
        let start_offset = codec_buffer.len();

        self.encode(payload, codec_buffer)?;

        let auth_payload_bytes = codec_buffer.split_off(start_offset);

        Ok(Frame {
            message_type,
            payload: auth_payload_bytes.freeze(),
        })
    }
}

pub trait DecodeFromFrame: Decoder {
    fn decode_from_frame(
        &mut self,
        frame: Frame,
        codec_buffer: &mut BytesMut,
    ) -> Result<Option<(Self::Item, MessageType)>, Self::Error>
    where
        Self: Sized,
    {
        codec_buffer.extend_from_slice(&frame.payload);

        if let Some(auth_payload) = self.decode(codec_buffer)? {
            Ok(Some((auth_payload, frame.message_type)))
        } else {
            Ok(None)
        }
    }
}
