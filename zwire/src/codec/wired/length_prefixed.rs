use super::{WiredInt, WiredField};

pub trait WiredLengthPrefixed: WiredField {
    type LengthPrefix: WiredInt;

    const MAX_LENGTH: usize;
}
