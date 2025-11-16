use crate::{errors::WireError, Frame, MessageType};
use tokio_util::{
    bytes::{Buf, BufMut, BytesMut},
    codec::{Decoder, Encoder},
};

// header_length = message_type_bytes (1) + payload_length_bytes (2) = 3
const MESSAGE_TYPE_FIELD_LENGTH: usize = 1;
const PAYLOAD_LENGTH_FIELD_LENGTH: usize = 2;
const HEADER_LENGTH: usize = MESSAGE_TYPE_FIELD_LENGTH + PAYLOAD_LENGTH_FIELD_LENGTH;
const MESSAGE_TYPE_OFFSET: usize = 0;
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
            return Err(WireError::Oversized("payload_length", payload_length, u16::MAX as usize));
        }

        if self.max_length > self.max_payload_length {
            return Err(WireError::Oversized(
                "payload_length",
                payload_length,
                self.max_payload_length,
            ));
        }

        let total_length = HEADER_LENGTH
            .checked_add(payload_length)
            .ok_or(WireError::Oversized("total_length", payload_length, self.max_length))?;

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
        let source_length = source.len();

        if source_length < HEADER_LENGTH {
            return Ok(None);
        }

        let message_type = MessageType::try_from(source[MESSAGE_TYPE_OFFSET])?;
        let payload_length = u16::from_be_bytes([
            source[PAYLOAD_LENGTH_OFFSET],
            source[PAYLOAD_LENGTH_OFFSET + 1],
        ]) as usize;

        if payload_length > self.max_payload_length {
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
                    self.max_payload_length,
                ))?;

        if source_length < total_length {
            return Ok(None);
        }

        source.advance(HEADER_LENGTH);

        let payload = source.split_to(payload_length).freeze();

        Ok(Some(Frame {
            message_type,
            payload,
        }))
    }
}
