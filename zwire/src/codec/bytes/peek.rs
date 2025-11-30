use super::super::wired::{WiredInt, WiredLengthPrefixed};
use crate::{helpers::CheckedAddWire, WireError};
use std::marker::PhantomData;
use tokio_util::bytes::BytesMut;

pub struct PeekLength<I: WiredLengthPrefixed> {
    ready: bool,
    length: usize,
    _phantom: PhantomData<I>,
}

impl<I: WiredLengthPrefixed> PeekLength<I> {
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    #[inline]
    pub fn get(&self) -> Option<usize> {
        self.ready.then_some(self.length)
    }

    #[inline]
    pub fn get_with_header(&self) -> Option<usize> {
        let length = self.get()?;

        length.checked_add(I::LengthPrefix::SIZE)
    }
}

pub trait BytesPeekExt {
    fn peek_at<I: WiredLengthPrefixed>(&self) -> Result<PeekLength<I>, WireError>;
}

impl BytesPeekExt for BytesMut {
    fn peek_at<I: WiredLengthPrefixed>(&self) -> Result<PeekLength<I>, WireError> {
        const DEFAULT_LENGTH: usize = 0;

        let start_offset = I::OFFSET;
        let end_offset =
            start_offset.checked_add_wire("OFFSET", I::LengthPrefix::SIZE, "LENGTH_HEADER_SIZE")?;

        if self.len() < end_offset {
            return Ok(PeekLength {
                ready: false,
                length: DEFAULT_LENGTH,
                _phantom: PhantomData,
            });
        }

        let prefix = &self[start_offset..end_offset];

        Ok(match I::LengthPrefix::read(prefix, I::FIELD_NAME)? {
            Some(length) => PeekLength {
                ready: true,
                length,
                _phantom: PhantomData,
            },
            None => PeekLength {
                ready: false,
                length: DEFAULT_LENGTH,
                _phantom: PhantomData,
            },
        })
    }
}
