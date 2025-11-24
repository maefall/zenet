use super::{WiredInt, WiredLengthPrefixed};
use crate::WireError;
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

    pub fn header_size(&self) -> usize {
        I::Int::SIZE
    }

    #[inline]
    pub fn get(&self) -> Option<usize> {
        self.ready.then_some(self.length)
    }

    #[inline]
    pub fn get_with_header(&self) -> Option<usize> {
        let length = self.get()?;
        let header_size = self.header_size();

        length.checked_add(header_size)
    }
}

pub trait BytesPeekExt {
    fn peek_at<I: WiredLengthPrefixed>(
        &self,
        offset: usize,
        field_name: &'static str,
    ) -> Result<PeekLength<I>, WireError>;
}

impl BytesPeekExt for BytesMut {
    fn peek_at<I: WiredLengthPrefixed>(
        &self,
        start_offset: usize,
        field_name: &'static str,
    ) -> Result<PeekLength<I>, WireError> {
        const DEFAULT_LENGTH: usize = 0;

        let size = I::Int::SIZE;
        let end_offset = start_offset.saturating_add(size);

        if self.len() < end_offset {
            return Ok(PeekLength {
                ready: false,
                length: DEFAULT_LENGTH,
                _phantom: PhantomData,
            });
        }

        let prefix = &self[start_offset..end_offset];

        Ok(match I::Int::read(prefix, field_name)? {
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
