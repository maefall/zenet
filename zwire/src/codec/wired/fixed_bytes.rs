use super::super::bytes::Bytes;

pub trait WiredFixedBytes {
    const SIZE: usize;
    const FIELD_NAME: &'static str;

    type Output;

    fn from_bytes(bytes: Bytes) -> Self::Output;
}
