#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum WireError {
    #[error("IO error: {0:#?}")]
    #[diagnostic(severity(Error))]
    Io(#[from] std::io::Error),

    #[error("Oversized, {1} bytes > {2} bytes limit at field ({0})")]
    #[diagnostic(severity(Error))]
    Oversized(&'static str,usize, usize),

    #[error("Invalid message type ({0})")]
    #[diagnostic(severity(Error))]
    InvalidMessageType(u8),

    #[error("Malformed string ({0})")]
    #[diagnostic(severity(Error))]
    MalformedString(&'static str),
}
