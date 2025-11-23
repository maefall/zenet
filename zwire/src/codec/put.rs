use super::length_prefix::LengthPrefix;
use crate::WireError;
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

pub trait BytesMutPutExt {
    fn put_single<T: LengthPrefix>(&mut self, value: T::Int);
    fn put_length_prefixed<T: LengthPrefix>(
        &mut self,
        payload: &Bytes,
        payload_field_name: &'static str,
        offset: Option<usize>,
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
            let required_length = offset + payload_length;

            if self.len() < required_length {
                self.resize(required_length, 0);
            }

            self[offset..required_length].copy_from_slice(payload);

            return Ok(());
        } else {
            self.extend_from_slice(payload);

            Ok(())
        }
    }

    #[inline]
    fn put_single<T: LengthPrefix>(&mut self, value: T::Int) {
        let bytes = T::to_bytes(value);

        self.put_slice(bytes.as_ref());
    }

    fn put_length_prefixed<T: LengthPrefix>(
        &mut self,
        payload: &Bytes,
        payload_field_name: &'static str,
        offset: Option<usize>,
    ) -> Result<(), WireError> {
        let payload_length = payload.len();

        if payload_length > T::MAX {
            return Err(WireError::Oversized(
                payload_field_name,
                payload_length,
                T::MAX,
            ));
        }

        let payload_length_bytes = T::to_bytes_from_usize(payload_length);
        let payload_length_bytes_slice = payload_length_bytes.as_ref();

        if let Some(offset) = offset {
            let header_length = offset + T::WIDTH;
            let total_length = header_length + payload_length;

            if self.len() < total_length {
                self.resize(total_length, 0);
            }

            self[offset..header_length].copy_from_slice(payload_length_bytes_slice);
            self[header_length..total_length].copy_from_slice(payload);

            return Ok(());
        }

        self.put_slice(payload_length_bytes_slice);
        self.put_slice(payload);

        Ok(())
    }
}
