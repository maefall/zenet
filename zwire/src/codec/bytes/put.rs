use crate::{
    codec::wired::{WiredFixedBytes, WiredInt, WiredIntInner, WiredLengthPrefixed},
    WireError,
};
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

pub trait BytesMutPutExt {
    fn put_fixed_bytes<F: WiredFixedBytes>(&mut self, bytes: &Bytes) -> Result<(), WireError>;
    fn put_single<I: WiredInt>(&mut self, value: <<I as WiredInt>::Inner as WiredIntInner>::Int);
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

    fn put_single<I: WiredInt>(&mut self, value: <<I as WiredInt>::Inner as WiredIntInner>::Int) {
        let bytes = <I::Inner as WiredIntInner>::to_bytes(value);

        self.put_slice(bytes.as_ref());
    }

    fn put_length_prefixed<I: WiredLengthPrefixed>(
        &mut self,
        payload: &Bytes,
    ) -> Result<(), WireError> {
        let payload_length = payload.len();
        let max_length = I::Inner::MAX;

        if payload_length > max_length {
            return Err(WireError::Oversized(
                I::FIELD_NAME,
                payload_length,
                max_length,
            ));
        }

        let payload_length_bytes = I::Inner::to_bytes_from_usize(payload_length);

        self.put_slice(payload_length_bytes.as_ref());
        self.put_slice(payload);

        Ok(())
    }
}
