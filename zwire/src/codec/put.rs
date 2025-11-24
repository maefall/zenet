use crate::{
    codec::{CheckedAddWire, WiredFixedBytes, WiredInt, WiredIntField, WiredLengthPrefixed},
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
    fn append_bytes(
        &mut self,
        payload: &Bytes,
        max_payload_length: usize,
        field_name: &'static str,
        offset: Option<usize>,
    ) -> Result<(), WireError>;
}

impl BytesMutPutExt for BytesMut {
    #[inline]
    fn put_fixed_bytes<B: WiredFixedBytes>(&mut self, payload: &Bytes) -> Result<(), WireError> {
        self.append_bytes(payload, B::SIZE, B::FIELD_NAME, None)
    }

    fn append_bytes(
        &mut self,
        payload: &Bytes,
        max_payload_length: usize,
        field_name: &'static str,
        offset: Option<usize>,
    ) -> Result<(), WireError> {
        let payload_length = payload.len();

        if payload_length > max_payload_length {
            return Err(WireError::Oversized(
                field_name,
                payload_length,
                max_payload_length,
            ));
        }

        if let Some(offset) = offset {
            let required_length =
                offset.checked_add_wire("REQUIRED_LENGTH", payload_length, "payload_length")?;

            if self.len() < required_length {
                self.resize(required_length, 0);
            }

            self[offset..required_length].copy_from_slice(payload);

            Ok(())
        } else {
            self.extend_from_slice(payload);

            Ok(())
        }
    }

    fn put_single<I: WiredIntField>(&mut self, value: <<I as WiredIntField>::Int as WiredInt>::Int) {
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
