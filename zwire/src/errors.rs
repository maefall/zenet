use std::str::Utf8Error;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum WireError {
    #[error("IO error: {0:#?}")]
    #[diagnostic(severity(Error))]
    Io(#[from] std::io::Error),

    #[error("oversized, {1} bytes > {2} bytes limit at field ({0})")]
    #[diagnostic(severity(Error))]
    Oversized(&'static str, usize, usize),

    #[error("underflow, field ({0}) has {1} bytes, needs {2}")]
    #[diagnostic(severity(Error))]
    Underflow(&'static str, usize, usize),

    #[error("invalid message type ({0})")]
    #[diagnostic(severity(Error))]
    InvalidMessageType(u8),

    #[error("malformed string ({0:?})")]
    #[diagnostic(severity(Error))]
    MalformedString(#[from] MalformedStringError),
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("{field:?}: {kind}")]
pub struct MalformedStringError {
    pub field: Option<&'static str>,
    pub kind: MalformedStringKind,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum MalformedStringKind {
    #[error("{0:?}")]
    #[diagnostic(severity(Error))]
    InvalidUtf8(#[from] Utf8Error),

    #[error("string contains non-ASCII bytes")]
    #[diagnostic(severity(Error))]
    NonAscii,

    #[error("string exceeds maximum length {0} > {1}")]
    #[diagnostic(severity(Error))]
    TooLong(usize, usize),

    #[error("string contains an unallowed byte: 0x{0:02X}")]
    #[diagnostic(severity(Error))]
    InvalidCharacter(u8),
}
