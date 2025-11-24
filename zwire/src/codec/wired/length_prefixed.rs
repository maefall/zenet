use super::WiredInt;

pub trait WiredLengthPrefixed {
    type Int: WiredInt;

    const FIELD_NAME: &'static str;
    const MAX_LENGTH: usize;
}
