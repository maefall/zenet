pub mod bytestring;

mod peek;
pub use peek::BytesPeekExt;

mod take;
pub use take::BytesMutTakeExt;

mod put;
pub use put::BytesMutPutExt;

mod wired_int;
pub use wired_int::WiredInt;

mod checked_add;
pub use checked_add::CheckedAddWire;

pub use zenet_macros::define_fields;

use crate::{errors::WireError, Frame, Message};
use tokio_util::{
    bytes::{Buf, BytesMut},
    codec::{Decoder, Encoder},
};

define_fields! {
    (Message, u8, 0, fixed),
    (Payload, u16, 1, length_prefix),
}

#[derive(Clone, Copy)]
pub struct FrameCodec {
    max_length: usize,
    max_payload_length: usize,
}

impl Default for FrameCodec {
    fn default() -> Self {
        const DEFAULT_MAX_PAYLOAD_LENGTH: usize = 1300;

        let max_payload_length = usize::MAX;

        FrameCodec {
            max_length: DEFAULT_MAX_PAYLOAD_LENGTH - FIXED_PART_LENGTH,
            max_payload_length,
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

        if payload_length > self.max_payload_length {
            return Err(WireError::Oversized(
                "payload_length",
                payload_length,
                self.max_payload_length,
            ));
        }

        let total_length = FIXED_PART_LENGTH.checked_add_wire(
            "FIXED_PART_LENGTH",
            payload_length,
            "payload_length",
        )?;

        if total_length > self.max_length {
            return Err(WireError::Oversized(
                "total_length",
                total_length,
                self.max_length,
            ));
        }

        destination.reserve(total_length);

        destination.put_single::<MessageWired>(frame.message_type as MessageWired); // repr
        destination.put_length_prefixed::<PayloadWired>(
            &frame.payload,
            "payload_length",
            Some(PAYLOAD_FIELD_OFFSET),
        )?;

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

        let Some(payload_length) = source
            .peek_at::<PayloadWired>(PAYLOAD_FIELD_OFFSET, "payload_length")?
            .get()
        else {
            return Ok(None);
        };

        if payload_length > self.max_payload_length {
            return Err(WireError::Oversized(
                "payload_length",
                payload_length,
                self.max_payload_length,
            ));
        }

        let total_length = FIXED_PART_LENGTH.checked_add_wire(
            "FIXED_PART_LENGTH",
            payload_length,
            "payload_length",
        )?;

        if source.len() < total_length {
            return Ok(None);
        }

        let message_type = Message::try_from(source.get_u8())?;

        if let Some(payload) =
            source.take_length_prefixed::<PayloadWired>(self.max_length, "payload")?
        {
            Ok(Some(Frame {
                message_type,
                payload,
            }))
        } else {
            Ok(None)
        }
    }
}
