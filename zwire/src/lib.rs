pub mod codec;
pub mod errors;
pub mod helpers;

pub use codec::{
    bytes::{Bytes, BytesMut},
    Decoder, Encoder, FrameCodec,
};
use errors::WireError;

pub mod __zwire_macros_support {
    pub use crate::{
        codec::wired::{WiredField, WiredFixedBytes, WiredInt, WiredLengthPrefixed, WiredString, WiredStringPolicyKind},
        errors::WireError, Message,
    };
    pub use tokio_util::bytes::Bytes;
}

#[derive(Debug, Clone)]
pub struct Message(pub u8);

#[derive(Debug, Clone)]
pub struct Frame {
    pub message: Message,
    pub payload: Bytes,
}

pub trait EncodeIntoFrame: Encoder<Self::EncodeItem> {
    type EncodeItem;

    fn encode_into_frame(
        &mut self,
        payload: Self::EncodeItem,
        message: impl Into<Message>,
        codec_buffer: &mut BytesMut,
    ) -> Result<Frame, Self::Error> {
        let start_offset = codec_buffer.len();

        self.encode(payload, codec_buffer)?;

        let payload_bytes = codec_buffer.split_off(start_offset);

        Ok(Frame {
            message: message.into(),
            payload: payload_bytes.freeze(),
        })
    }
}

pub trait DecodeFromFrame: Decoder {
    fn decode_from_frame(
        &mut self,
        frame: Frame,
        codec_buffer: &mut BytesMut,
    ) -> Result<Option<(Self::Item, Message)>, Self::Error>
    where
        Self: Sized,
    {
        codec_buffer.extend_from_slice(&frame.payload);

        if let Some(payload) = self.decode(codec_buffer)? {
            Ok(Some((payload, frame.message)))
        } else {
            Ok(None)
        }
    }
}
