use super::super::wired::{WiredFixedBytes, WiredInt, WiredIntInner, WiredLengthPrefixed};
use crate::{errors::WireError, helpers::CheckedAddWire};
use tokio_util::bytes::{Buf, Bytes, BytesMut};

pub trait BytesMutTakeExt {
    fn take_single<I: WiredInt>(
        &mut self,
    ) -> Option<<<I as WiredInt>::Inner as WiredIntInner>::Int>;
    fn take_single_unchecked<I: WiredInt>(
        &mut self,
    ) -> <<I as WiredInt>::Inner as WiredIntInner>::Int;

    fn take_fixed_bytes<F: WiredFixedBytes>(&mut self) -> Option<F::Output>;
    fn take_fixed_bytes_unchecked<F: WiredFixedBytes>(&mut self) -> F::Output;

    fn take_length_prefixed_unchecked<I: WiredLengthPrefixed>(
        &mut self,
    ) -> Result<Bytes, WireError>;
    fn take_length_prefixed<I: WiredLengthPrefixed>(&mut self) -> Result<Option<Bytes>, WireError>;
}

impl BytesMutTakeExt for BytesMut {
    #[inline]
    fn take_single_unchecked<I: WiredInt>(
        &mut self,
    ) -> <<I as WiredInt>::Inner as WiredIntInner>::Int {
        let size = I::Inner::SIZE;
        let value = I::Inner::read_raw_unchecked(&self[..size]);

        self.advance(size);

        value
    }

    #[inline]
    fn take_single<I: WiredInt>(
        &mut self,
    ) -> Option<<<I as WiredInt>::Inner as WiredIntInner>::Int> {
        let size = I::Inner::SIZE;
        let value = I::Inner::read_raw(&self[..size])?;

        self.advance(size);

        Some(value)
    }

    #[inline]
    fn take_fixed_bytes_unchecked<F: WiredFixedBytes>(&mut self) -> F::Output {
        let chunk: Bytes = self.split_to(F::SIZE).freeze();

        F::from_bytes(chunk)
    }

    #[inline]
    fn take_fixed_bytes<F: WiredFixedBytes>(&mut self) -> Option<F::Output> {
        if self.len() < F::SIZE {
            return None;
        }

        Some(self.take_fixed_bytes_unchecked::<F>())
    }

    fn take_length_prefixed_unchecked<I: WiredLengthPrefixed>(
        &mut self,
    ) -> Result<Bytes, WireError> {
        let size = I::Inner::SIZE;
        let max_payload_length = I::MAX_LENGTH;
        let expected_payload_length = I::Inner::read_unchecked(&self[..size], "payload_length")?;
        if expected_payload_length > max_payload_length {
            return Err(WireError::Oversized(
                I::FIELD_NAME,
                expected_payload_length,
                max_payload_length,
            ));
        }

        self.advance(size);

        let bytes = self.split_to(expected_payload_length).freeze();

        Ok(bytes)
    }

    fn take_length_prefixed<I: WiredLengthPrefixed>(&mut self) -> Result<Option<Bytes>, WireError> {
        let size = I::Inner::SIZE;
        let max_payload_length = I::MAX_LENGTH;

        if self.len() < size {
            return Ok(None);
        }

        let Some(expected_payload_length) = I::Inner::read(&self[..size], "payload_length")? else {
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
