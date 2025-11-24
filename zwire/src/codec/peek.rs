use super::WiredInt;
use crate::WireError;
use std::marker::PhantomData;
use tokio_util::bytes::BytesMut;

pub struct PeekLength<I: WiredInt> {
    ready: bool,
    length: usize,
    _phantom: PhantomData<I>,
}

impl<I: WiredInt> PeekLength<I> {
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    #[inline]
    pub fn get(&self) -> Option<usize> {
        self.ready.then_some(self.length)
    }

    #[inline]
    pub fn get_total(&self) -> Option<usize> {
        self.checked_add(I::SIZE)
    }

    #[inline]
    pub fn get_total_separated(&self) -> Option<(usize, usize)> {
        if let Some(length) = self.get() {
            return Some((length, I::SIZE));
        }

        None
    }

    #[inline]
    pub fn saturating_add(&self, right_side: usize) -> Option<usize> {
        self.get()
            .map(|left_side| left_side.saturating_add(right_side))
    }

    #[inline]
    pub fn checked_add(&self, right_side: usize) -> Option<usize> {
        self.get()?.checked_add(right_side)
    }
}

pub trait BytesPeekExt {
    fn peek_at<I: WiredInt>(
        &self,
        offset: usize,
        field_name: &'static str,
    ) -> Result<PeekLength<I>, WireError>;
}

impl BytesPeekExt for BytesMut {
    fn peek_at<I: WiredInt>(
        &self,
        start_offset: usize,
        field_name: &'static str,
    ) -> Result<PeekLength<I>, WireError> {
        const DEFAULT_LENGTH: usize = 0;

        let size = I::SIZE;
        let end_offset = start_offset.saturating_add(size);

        if self.len() < end_offset {
            return Ok(PeekLength {
                ready: false,
                length: DEFAULT_LENGTH,
                _phantom: PhantomData,
            });
        }

        let prefix = &self[start_offset..end_offset];

        Ok(match I::read(prefix, field_name)? {
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
