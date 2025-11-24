use super::WiredIntInner;

pub trait WiredLengthPrefixed {
    type Inner: WiredIntInner;

    const FIELD_NAME: &'static str;
    const MAX_LENGTH: usize;
}
