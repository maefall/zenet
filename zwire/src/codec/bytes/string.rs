pub use bytestr::ByteStr;

use crate::errors::{MalformedStringError, MalformedStringKind};
use super::Bytes;

pub enum ByteStringFieldPolicy {
    Utf8,
    AsciiHyphen,
}

impl ByteStringFieldPolicy {
    #[inline]
    fn validate(&self, source: &ByteStr) -> Result<(), MalformedStringError> {
        match self {
            ByteStringFieldPolicy::Utf8 => Ok(()),
            ByteStringFieldPolicy::AsciiHyphen => {
                for &bytes in source.as_bytes() {
                    if !(bytes.is_ascii_alphanumeric() || bytes == b'_' || bytes == b'-') {
                        return Err(MalformedStringError {
                            field: None,
                            kind: MalformedStringKind::InvalidCharacter(bytes),
                        });
                    }
                }
                Ok(())
            }
        }
    }
}

pub trait ByteStringFieldExt {
    fn to_bytestr_field(
        self,
        field: &'static str,
        policy: ByteStringFieldPolicy,
    ) -> Result<ByteStr, MalformedStringError>;
}

impl ByteStringFieldExt for Bytes {
    fn to_bytestr_field(
        self,
        field: &'static str,
        policy: ByteStringFieldPolicy,
    ) -> Result<ByteStr, MalformedStringError> {
        let byte_string =
            bytestr::ByteStr::from_utf8(self).map_err(|error| MalformedStringError {
                field: Some(field),
                kind: MalformedStringKind::InvalidUtf8(error),
            })?;

        if let Err(mut error) = policy.validate(&byte_string) {
            error.field = Some(field);

            return Err(error);
        }

        Ok(byte_string)
    }
}
