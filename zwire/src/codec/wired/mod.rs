mod fixed_bytes;
mod int;
mod length_prefixed;

pub use self::{
    fixed_bytes::WiredFixedBytes,
    int::{WiredInt, WiredIntInner},
    length_prefixed::WiredLengthPrefixed,
};
pub use zenet_macros::define_fields;
