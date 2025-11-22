use super::length_prefix::LengthPrefix;
use std::marker::PhantomData;
use tokio_util::bytes::BytesMut;

pub struct PeekLength<T: LengthPrefix> {
    ready: bool,
    length: usize,
    _phantom: PhantomData<T>,
}

impl<T: LengthPrefix> PeekLength<T> {
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
        let header_length = T::WIDTH;

        self.checked_add(header_length)
    }

    #[inline]
    pub fn get_total_seperated(&self) -> Option<(usize, usize)> {
        if let Some(length) = self.get() {
            let header_length = T::WIDTH;

            return Some((length, header_length));
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
    fn peek_at<T: LengthPrefix>(&self, offset: usize) -> PeekLength<T>;
}

impl BytesPeekExt for BytesMut {
    #[inline]
    fn peek_at<T: LengthPrefix>(&self, start_offset: usize) -> PeekLength<T> {
        const DEFAULT_LENGTH: usize = 0;

        let width = T::WIDTH;
        let end_offset = start_offset.saturating_add(width);

        if self.len() < start_offset.saturating_add(width) {
            return PeekLength {
                ready: false,
                length: DEFAULT_LENGTH,
                _phantom: PhantomData,
            };
        }

        let prefix = &self[start_offset..end_offset];

        match T::read(prefix) {
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
        }
    }
}
