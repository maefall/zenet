mod fixed_bytes;
mod int;
mod length_prefixed;
mod string;

pub use self::{
    fixed_bytes::WiredFixedBytes,
    int::WiredInt,
    length_prefixed::WiredLengthPrefixed,
    string::{WiredString, WiredStringPolicyKind},
};
pub use zenet_macros::{define_fields, define_message};

pub trait WiredField {
    const FIELD_NAME: &'static str;
    const OFFSET: usize;
}
