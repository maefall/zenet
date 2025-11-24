use super::{CheckedAddWire, WiredInt};
use crate::errors::WireError;
use tokio_util::bytes::{Buf, Bytes, BytesMut};

pub trait BytesMutTakeExt {
    fn take_length_prefixed<I: WiredInt>(
        &mut self,
        max_payload_length: usize,
        payload_field_name: &'static str,
    ) -> Result<Option<Bytes>, WireError>;
}

impl BytesMutTakeExt for BytesMut {
    fn take_length_prefixed<I: WiredInt>(
        &mut self,
        max_payload_length: usize,
        payload_field_name: &'static str,
    ) -> Result<Option<Bytes>, WireError> {
        let size = I::SIZE;

        if self.len() < size {
            return Ok(None);
        }

        let Some(expected_payload_length) = I::read(&self[..size], "payload_length")? else {
            return Ok(None);
        };

        if expected_payload_length > max_payload_length {
            return Err(WireError::Oversized(
                payload_field_name,
                expected_payload_length,
                max_payload_length,
            ));
        }

        let total_length = size.checked_add_wire(
            "LENGTH_PREFIX_HEADER_SIZE",
            expected_payload_length,
            "payload_length",
        )?;

        if self.len() < total_length {
            return Ok(None);
        }

        self.advance(size);

        let bytes = self.split_to(expected_payload_length).freeze();

        Ok(Some(bytes))
    }
}
