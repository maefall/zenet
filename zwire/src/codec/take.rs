use super::{CheckedAddWire, WiredFixedBytes, WiredInt, WiredIntField, WiredLengthPrefixed};
use crate::errors::WireError;
use tokio_util::bytes::{Buf, Bytes, BytesMut};

pub trait BytesMutTakeExt {
    fn take_single<I: WiredIntField>(
        &mut self,
    ) -> Option<<<I as WiredIntField>::Int as WiredInt>::Int>;
    fn take_fixed_bytes<F: WiredFixedBytes>(&mut self) -> Option<F::Output>;
    fn take_length_prefixed<I: WiredLengthPrefixed>(&mut self) -> Result<Option<Bytes>, WireError>;
}

impl BytesMutTakeExt for BytesMut {
    #[inline]
    fn take_fixed_bytes<F: WiredFixedBytes>(&mut self) -> Option<F::Output> {
        let size = F::SIZE;

        if self.len() < size {
            return None;
        }

        let chunk: Bytes = self.split_to(size).freeze();

        Some(F::from_bytes(chunk))
    }

    fn take_single<I: WiredIntField>(
        &mut self,
    ) -> Option<<<I as WiredIntField>::Int as WiredInt>::Int> {
        let size = I::Int::SIZE;
        let value = I::Int::read_raw(&self[..size])?;

        self.advance(size);

        Some(value)
    }

    fn take_length_prefixed<I: WiredLengthPrefixed>(&mut self) -> Result<Option<Bytes>, WireError> {
        let size = I::Int::SIZE;
        let max_payload_length = I::MAX_LENGTH;

        if self.len() < size {
            return Ok(None);
        }

        let Some(expected_payload_length) = I::Int::read(&self[..size], "payload_length")? else {
            return Ok(None);
        };

        if expected_payload_length > max_payload_length {
            return Err(WireError::Oversized(
                I::FIELD_NAME,
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
