use crate::{
    codec::wired::{WiredFixedBytes, WiredInt, WiredIntField, WiredLengthPrefixed},
    WireError,
};
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

pub trait BytesMutPutExt {
    fn put_fixed_bytes<F: WiredFixedBytes>(&mut self, bytes: &Bytes) -> Result<(), WireError>;
    fn put_single<I: WiredIntField>(&mut self, value: <<I as WiredIntField>::Int as WiredInt>::Int);
    fn put_length_prefixed<I: WiredLengthPrefixed>(
        &mut self,
        payload: &Bytes,
    ) -> Result<(), WireError>;
}

impl BytesMutPutExt for BytesMut {
    #[inline]
    fn put_fixed_bytes<B: WiredFixedBytes>(&mut self, payload: &Bytes) -> Result<(), WireError> {
        let payload_length = payload.len();
        let max_payload_length = B::SIZE;

        if payload_length > max_payload_length {
            return Err(WireError::Oversized(
                B::FIELD_NAME,
                payload_length,
                max_payload_length,
            ));
        }

        self.extend_from_slice(payload);

        Ok(())
    }

    fn put_single<I: WiredIntField>(
        &mut self,
        value: <<I as WiredIntField>::Int as WiredInt>::Int,
    ) {
        let bytes = <I::Int as WiredInt>::to_bytes(value);

        self.put_slice(bytes.as_ref());
    }

    fn put_length_prefixed<I: WiredLengthPrefixed>(
        &mut self,
        payload: &Bytes,
    ) -> Result<(), WireError> {
        let payload_length = payload.len();

        if payload_length > I::Int::MAX {
            return Err(WireError::Oversized(
                I::FIELD_NAME,
                payload_length,
                I::Int::MAX,
            ));
        }

        let payload_length_bytes = I::Int::to_bytes_from_usize(payload_length);

        self.put_slice(payload_length_bytes.as_ref());
        self.put_slice(payload);

        Ok(())
    }
}
