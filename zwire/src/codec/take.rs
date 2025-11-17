use super::length_prefix::LengthPrefix;
use crate::errors::WireError;
use tokio_util::bytes::{Buf, Bytes, BytesMut};

pub trait BytesMutTakeExt {
    fn take_length_prefixed_payload<T: LengthPrefix>(
        &mut self,
        max_payload_length: usize,
        payload_field_name: &'static str,
    ) -> Result<Option<Bytes>, WireError>;
}

impl BytesMutTakeExt for BytesMut {
    #[inline]
    fn take_length_prefixed_payload<T: LengthPrefix>(
        &mut self,
        max_payload_length: usize,
        payload_field_name: &'static str,
    ) -> Result<Option<Bytes>, WireError> {
        let width = T::WIDTH;

        if self.len() < width {
            return Ok(None);
        }

        let expected_payload_length = match T::read(&self[..width]) {
            Some(n) => n,
            None => return Ok(None),
        };

        if expected_payload_length > max_payload_length {
            return Err(WireError::Oversized(
                payload_field_name,
                expected_payload_length,
                max_payload_length,
            ));
        }

        let total_length = width.saturating_add(expected_payload_length);

        if self.len() < total_length {
            return Ok(None);
        }

        self.advance(width);

        let bytes = self.split_to(expected_payload_length).freeze();

        Ok(Some(bytes))
    }
}
