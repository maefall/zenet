use super::super::errors::{MalformedStringError, MalformedStringKind};
use bytestr::ByteStr;
use tokio_util::bytes::Bytes;

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
        max_length: Option<usize>,
        policy: ByteStringFieldPolicy,
    ) -> Result<ByteStr, MalformedStringError>;
}

impl ByteStringFieldExt for Bytes {
    fn to_bytestr_field(
        self,
        field: &'static str,
        max_length: Option<usize>,
        policy: ByteStringFieldPolicy,
    ) -> Result<ByteStr, MalformedStringError> {
        #[allow(clippy::collapsible_if)]
        if let Some(max_length) = max_length {
            if self.len() > max_length {
                return Err(MalformedStringError {
                    field: Some(field),
                    kind: MalformedStringKind::TooLong(self.len(), max_length),
                });
            }
        }

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
