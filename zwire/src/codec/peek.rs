use super::length_prefix::LengthPrefix;
use tokio_util::bytes::BytesMut;

pub struct PeekLength {
    pub ready: bool,
    pub length: usize,
}

impl From<PeekLength> for usize {
    fn from(peek_length: PeekLength) -> Self {
        peek_length.length
    }
}

impl PartialEq<usize> for PeekLength {
    #[inline]
    fn eq(&self, other: &usize) -> bool {
        self.ready && self.length == *other
    }
}

impl PartialOrd<usize> for PeekLength {
    #[inline]
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        if !self.ready {
            return Some(std::cmp::Ordering::Less);
        }

        self.length.partial_cmp(other)
    }
}

pub trait BytesPeekExt {
    fn peek_at<T: LengthPrefix>(&self, offset: usize) -> PeekLength;
}

impl BytesPeekExt for BytesMut {
    #[inline]
    fn peek_at<T: LengthPrefix>(&self, start_offset: usize) -> PeekLength {
        const DEFAULT_LENGTH: usize = 0;

        let width = T::WIDTH;
        let end_offset = start_offset.saturating_add(width);

        if self.len() < start_offset.saturating_add(width) {
            return PeekLength {
                ready: false,
                length: DEFAULT_LENGTH,
            };
        }

        let prefix = &self[start_offset..end_offset];

        match T::read(prefix) {
            Some(length) => PeekLength {
                ready: true,
                length,
            },
            None => PeekLength {
                ready: false,
                length: DEFAULT_LENGTH,
            },
        }
    }
}
