pub mod bytestring;

mod peek;
pub use peek::BytesPeekExt;

mod take;
pub use take::BytesMutTakeExt;

mod put;
pub use put::BytesMutPutExt;

mod wired_int;
pub use wired_int::{WiredInt, WiredIntField};

mod wired_length_prefixed;
pub use wired_length_prefixed::WiredLengthPrefixed;

mod wired_fixed_bytes;
pub use wired_fixed_bytes::WiredFixedBytes;

mod checked_add;
pub use checked_add::CheckedAddWire;

pub use zenet_macros::define_fields;

use crate::{errors::WireError, Frame, Message};
use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, Encoder},
};

define_fields! {
    (Message, u8, fixed),
    (Payload, u16, length_prefix, 1300),
}

#[derive(Clone, Copy)]
pub struct FrameCodec {
    max_length: usize,
    max_payload_length: usize,
}

impl Default for FrameCodec {
    fn default() -> Self {
        FrameCodec {
            max_length: fields::MAX_LENGTH,
            max_payload_length: fields::payload::MAX_LENGTH,
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

        let total_length = fields::FIXED_PART_LENGTH.checked_add_wire(
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

        destination.put_single::<fields::message::Wired>(frame.message.into()); // repr
        destination.put_length_prefixed::<fields::payload::Wired>(
            &frame.payload,
            "payload_length",
            None,
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
            .peek_at::<fields::payload::Wired>(fields::payload::OFFSET, "payload_length")?
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

        let total_length = fields::FIXED_PART_LENGTH.checked_add_wire(
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

        if source.len() < total_length {
            return Ok(None);
        }

        let Some(message_code) = source.take_single::<fields::message::Wired>() else {
            return Ok(None);
        };

        let message = Message::try_from(message_code)?;

        if let Some(payload) = source.take_length_prefixed::<fields::payload::Wired>()? {
            Ok(Some(Frame { message, payload }))
        } else {
            Ok(None)
        }
    }
}
