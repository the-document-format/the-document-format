//! Error type for the binary backend.

#[derive(Debug, thiserror::Error)]
pub enum TdfBinaryError {
    #[error("invalid magic bytes")]
    InvalidMagic,
    #[error("unsupported version {0}")]
    UnsupportedVersion(u8),
    #[error("encode error: {0}")]
    Encode(#[from] bincode_next::error::EncodeError),
    #[error("decode error: {0}")]
    Decode(#[from] bincode_next::error::DecodeError),
    #[error("offset out of bounds: {0}")]
    BadOffset(u64),
    #[error("invalid pointer reference")]
    InvalidPointerRef,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<TdfBinaryError> for std::io::Error {
    fn from(e: TdfBinaryError) -> Self {
        std::io::Error::other(e)
    }
}
