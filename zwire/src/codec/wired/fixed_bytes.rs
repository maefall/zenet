use super::{super::bytes::Bytes, WiredField};

pub trait WiredFixedBytes: WiredField {
    type Output;

    const LENGTH: usize;

    fn from_bytes(bytes: Bytes) -> Self::Output;
}
