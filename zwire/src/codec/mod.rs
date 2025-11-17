pub mod bytestring;

mod peek;
pub use peek::BytesPeekExt;

mod take;
pub use take::BytesMutTakeExt;

mod length_prefix;

use crate::{errors::WireError, Frame, Message};
use tokio_util::{
    bytes::{Buf, BufMut, BytesMut},
    codec::{Decoder, Encoder},
};

const MESSAGE_TYPE_FIELD_LENGTH: usize = 1;
const PAYLOAD_LENGTH_FIELD_LENGTH: usize = 2;
const HEADER_LENGTH: usize = MESSAGE_TYPE_FIELD_LENGTH + PAYLOAD_LENGTH_FIELD_LENGTH;
const PAYLOAD_LENGTH_OFFSET: usize = 1;

#[derive(Clone, Copy)]
pub struct FrameCodec {
    max_payload_length: usize,
    max_length: usize,
}

impl Default for FrameCodec {
    fn default() -> Self {
        let max_payload_length = usize::MAX;

        FrameCodec {
            max_payload_length,
            max_length: max_payload_length.saturating_add(HEADER_LENGTH),
        }
    }
}

impl Encoder<Frame> for FrameCodec {
    type Error = WireError;

    fn encode(&mut self, frame: Frame, destination: &mut BytesMut) -> Result<(), Self::Error> {
        let payload_length = frame.payload.len();

        if payload_length > u16::MAX as usize {
            return Err(WireError::Oversized(
                "payload_length",
                payload_length,
                u16::MAX as usize,
            ));
        }

        if self.max_length > self.max_payload_length {
            return Err(WireError::Oversized(
                "payload_length",
                payload_length,
                self.max_payload_length,
            ));
        }

        let total_length =
            HEADER_LENGTH
                .checked_add(payload_length)
                .ok_or(WireError::Oversized(
                    "total_length",
                    payload_length,
                    self.max_length,
                ))?;

        destination.reserve(total_length);

        destination.put_u8(frame.message_type as u8); // repr
        destination.put_u16(payload_length as u16);
        destination.extend_from_slice(&frame.payload);

        Ok(())
    }
}

impl Decoder for FrameCodec {
    type Item = Frame;
    type Error = WireError;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if source.is_empty() {
            return Ok(None);
        }

        let payload_length_peek = source.peek_at::<u16>(PAYLOAD_LENGTH_OFFSET);
        let payload_length = payload_length_peek.length;

        let total_length =
            HEADER_LENGTH
                .checked_add(payload_length)
                .ok_or(WireError::Oversized(
                    "total_length",
                    payload_length,
                    self.max_payload_length,
                ))?;

        if source.len() < total_length {
            return Ok(None);
        }

        let message_type = Message::try_from(source.get_u8())?;

        if let Some(payload) =
            source.take_length_prefixed_payload::<u16>(self.max_payload_length, "payload")?
        {
            Ok(Some(Frame {
                message_type,
                payload,
            }))
        } else {
            println!("Payload not present after complete frame");

            Ok(None)
        }
    }
}
