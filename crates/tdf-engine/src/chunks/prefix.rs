//! The prefix chunk is whole-document high level metadata.
//!
//! Metadata about the actual semantic information contained in the document
//! should go in the header, not in the prefix.

/// Magic bits to identify a TDF file.
///
/// These are ascii so they appear in hexdump.
pub const MAGIC_BITS: [u8; 3] = [b't', b'd', b'f'];

pub struct PreludeChunk {
    /// Magic bits to identify a TDF file.
    magic_bits: [u8; 3],
    /// The version of TDF that this file is for.
    version: u16,
    /// The length of the entire file, including the prelude.
    file_len: u64,
}

impl PreludeChunk {
    pub fn new(len: u64) -> Self {
        Self {
            magic_bits: MAGIC_BITS,
            version: 1,
            file_len: len,
        }
    }
}
