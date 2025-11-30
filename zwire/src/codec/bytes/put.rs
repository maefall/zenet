use crate::{
    codec::bytes::ByteStr,
    codec::wired::{WiredFixedBytes, WiredInt, WiredLengthPrefixed, WiredString},
    errors::{MalformedStringError, MalformedStringKind},
    WireError,
};
use std::cmp::Ordering;
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

pub trait BytesMutPutExt {
    fn put_single<I: WiredInt>(&mut self, value: <I as WiredInt>::Int);
    fn put_fixed_bytes<F: WiredFixedBytes>(&mut self, bytes: &Bytes) -> Result<(), WireError>;
    fn put_length_prefixed<I: WiredLengthPrefixed>(
        &mut self,
        payload: &Bytes,
    ) -> Result<(), WireError>;
    fn put_length_prefixed_string<I: WiredString>(
        &mut self,
        payload: impl Into<Bytes>,
    ) -> Result<(), WireError>;
}

impl BytesMutPutExt for BytesMut {
    #[inline]
    fn put_single<I: WiredInt>(&mut self, value: <I as WiredInt>::Int) {
        let bytes = I::to_bytes(value);

        self.put_slice(bytes.as_ref());
    }

    #[inline]
    fn put_fixed_bytes<B: WiredFixedBytes>(&mut self, payload: &Bytes) -> Result<(), WireError> {
        let payload_length = payload.len();
        let required_payload_length = B::LENGTH;

        match payload_length.cmp(&required_payload_length) {
            Ordering::Greater => Err(WireError::Oversized(
                B::FIELD_NAME,
                payload_length,
                required_payload_length,
            )),
            Ordering::Less => Err(WireError::Underflow(
                B::FIELD_NAME,
                payload_length,
                required_payload_length,
            )),
            Ordering::Equal => {
                self.extend_from_slice(payload);

                Ok(())
            }
        }
    }

    fn put_length_prefixed<I: WiredLengthPrefixed>(
        &mut self,
        payload: &Bytes,
    ) -> Result<(), WireError> {
        let payload_length = payload.len();
        let max_length = I::LengthPrefix::MAX;

        if payload_length > max_length {
            return Err(WireError::Oversized(
                I::FIELD_NAME,
                payload_length,
                max_length,
            ));
        }

        let payload_length_bytes = I::LengthPrefix::to_bytes_from_usize(payload_length);

        self.put_slice(payload_length_bytes.as_ref());
        self.put_slice(payload);

        Ok(())
    }

    fn put_length_prefixed_string<I: WiredString>(
        &mut self,
        payload: impl Into<Bytes>,
    ) -> Result<(), WireError> {
        let byte_string =
            ByteStr::from_utf8(payload.into()).map_err(|error| MalformedStringError {
                field: Some(I::FIELD_NAME),
                kind: MalformedStringKind::InvalidUtf8(error),
            })?;

        if let Err(mut error) = I::POLICY.validate(&byte_string) {
            error.field = Some(I::FIELD_NAME);

            return Err(WireError::MalformedString(error));
        };

        self.put_length_prefixed::<I::Inner>(&byte_string.into_bytes())
    }
}
