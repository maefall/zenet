use super::{WiredField, WiredLengthPrefixed};
use crate::{
    codec::bytes::ByteStr,
    errors::{MalformedStringError, MalformedStringKind},
};

pub trait WiredString: WiredField {
    type Inner: WiredLengthPrefixed;

    const POLICY: WiredStringPolicyKind;
}

pub enum WiredStringPolicyKind {
    Utf8,
    AsciiHyphen,
}

impl WiredStringPolicyKind {
    #[inline]
    pub fn validate(&self, source: &ByteStr) -> Result<(), MalformedStringError> {
        match self {
            WiredStringPolicyKind::Utf8 => Ok(()),
            WiredStringPolicyKind::AsciiHyphen => {
                for &byte in source.as_bytes() {
                    if !(byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-') {
                        return Err(MalformedStringError {
                            field: None,
                            kind: MalformedStringKind::InvalidCharacter(byte),
                        });
                    }
                }

                Ok(())
            }
        }
    }
}
