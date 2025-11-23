use super::{CheckedAddWire, LengthPrefix};
use crate::errors::WireError;
use tokio_util::bytes::{Buf, Bytes, BytesMut};

pub trait BytesMutTakeExt {
    fn take_length_prefixed<T: LengthPrefix>(
        &mut self,
        max_payload_length: usize,
        payload_field_name: &'static str,
    ) -> Result<Option<Bytes>, WireError>;
}

impl BytesMutTakeExt for BytesMut {
    fn take_length_prefixed<T: LengthPrefix>(
        &mut self,
        max_payload_length: usize,
        payload_field_name: &'static str,
    ) -> Result<Option<Bytes>, WireError> {
        let width = T::WIDTH;

        if self.len() < width {
            return Ok(None);
        }

        let Some(expected_payload_length) = T::read(&self[..width]) else {
            return Ok(None);
        };

        if expected_payload_length > max_payload_length {
            return Err(WireError::Oversized(
                payload_field_name,
                expected_payload_length,
                max_payload_length,
            ));
        }

        let total_length = width.checked_add_wire(
            expected_payload_length,
            "payload_length_header",
            payload_field_name,
        )?;

        if self.len() < total_length {
            return Ok(None);
        }

        self.advance(width);

        let bytes = self.split_to(expected_payload_length).freeze();

        Ok(Some(bytes))
    }
}
