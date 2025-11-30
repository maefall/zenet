use super::super::wired::{WiredFixedBytes, WiredInt, WiredLengthPrefixed, WiredString};
use crate::{
    codec::bytes::ByteStr,
    errors::{MalformedStringError, MalformedStringKind, WireError},
    helpers::CheckedAddWire,
};
use tokio_util::bytes::{Buf, Bytes, BytesMut};

pub trait BytesMutTakeExt {
    fn take_single<I: WiredInt>(&mut self) -> Option<<I as WiredInt>::Int>;
    fn take_single_unchecked<I: WiredInt>(&mut self) -> <I as WiredInt>::Int;

    fn take_fixed_bytes_unchecked<F: WiredFixedBytes>(&mut self) -> F::Output;
    fn take_fixed_bytes<F: WiredFixedBytes>(&mut self) -> Option<F::Output>;

    fn take_length_prefixed_unchecked<I: WiredLengthPrefixed>(
        &mut self,
    ) -> Result<Bytes, WireError>;
    fn take_length_prefixed<I: WiredLengthPrefixed>(&mut self) -> Result<Option<Bytes>, WireError>;

    fn take_length_prefixed_string<I: WiredString>(&mut self)
        -> Result<Option<ByteStr>, WireError>;
    fn take_length_prefixed_string_unchecked<I: WiredString>(
        &mut self,
    ) -> Result<ByteStr, WireError>;
}

impl BytesMutTakeExt for BytesMut {
    #[inline]
    fn take_single_unchecked<I: WiredInt>(&mut self) -> <I as WiredInt>::Int {
        let size = I::SIZE;
        let value = I::read_raw_unchecked(&self[..size]);

        self.advance(size);

        value
    }

    #[inline]
    fn take_single<I: WiredInt>(&mut self) -> Option<<I as WiredInt>::Int> {
        let size = I::SIZE;
        let value = I::read_raw(&self[..size])?;

        self.advance(size);

        Some(value)
    }

    #[inline]
    fn take_fixed_bytes_unchecked<F: WiredFixedBytes>(&mut self) -> F::Output {
        let chunk: Bytes = self.split_to(F::LENGTH).freeze();

        F::from_bytes(chunk)
    }

    #[inline]
    fn take_fixed_bytes<F: WiredFixedBytes>(&mut self) -> Option<F::Output> {
        if self.len() < F::LENGTH {
            return None;
        }

        Some(self.take_fixed_bytes_unchecked::<F>())
    }

    fn take_length_prefixed_unchecked<I: WiredLengthPrefixed>(
        &mut self,
    ) -> Result<Bytes, WireError> {
        let size = I::LengthPrefix::SIZE;
        let max_payload_length = I::MAX_LENGTH;
        let expected_payload_length =
            I::LengthPrefix::read_unchecked(&self[..size], "payload_length")?;

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
        let size = I::LengthPrefix::SIZE;
        let max_payload_length = I::MAX_LENGTH;

        if self.len() < size {
            return Ok(None);
        }

        let Some(expected_payload_length) = I::LengthPrefix::read(&self[..size], "payload_length")?
        else {
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

    fn take_length_prefixed_string_unchecked<I: WiredString>(
        &mut self,
    ) -> Result<ByteStr, WireError> {
        let payload = self.take_length_prefixed_unchecked::<I::Inner>()?;

        let byte_string = ByteStr::from_utf8(payload).map_err(|error| MalformedStringError {
            field: Some(I::FIELD_NAME),
            kind: MalformedStringKind::InvalidUtf8(error),
        })?;

        if let Err(mut error) = I::POLICY.validate(&byte_string) {
            error.field = Some(I::FIELD_NAME);

            return Err(WireError::MalformedString(error));
        };

        Ok(byte_string)
    }

    fn take_length_prefixed_string<I: WiredString>(
        &mut self,
    ) -> Result<Option<ByteStr>, WireError> {
        let Some(payload) = self.take_length_prefixed::<I::Inner>()? else {
            return Ok(None);
        };

        let byte_string = ByteStr::from_utf8(payload).map_err(|error| MalformedStringError {
            field: Some(I::FIELD_NAME),
            kind: MalformedStringKind::InvalidUtf8(error),
        })?;

        if let Err(mut error) = I::POLICY.validate(&byte_string) {
            error.field = Some(I::FIELD_NAME);

            return Err(WireError::MalformedString(error));
        };

        Ok(Some(byte_string))
    }
}
