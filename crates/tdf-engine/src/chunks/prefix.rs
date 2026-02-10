//! The prefix chunk is whole-document high level metadata.
//!
//! Metadata about the actual semantic information contained in the document
//! should go in the header, not in the prefix.

use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

/// Magic bits to identify a TDF file. "Trev" stands for "Trevor."
///
/// These are ASCII so they appear in hexdump.
pub const MAGIC_BYTES: [u8; 6] = [b'T', b'R', b'E', b'V', b'D', b'F'];

pub struct HeaderChunk {
    /// Magic bits to identify a TDF file.
    magic_bytes: [u8; 6],
    /// The version of TDF that this file is for.
    version: u8,
    /// The length of the entire file, including the prelude.
    file_len: u64,
    /// The compression mode to use for the rest of file.
    compression: Compression,
    /// All offsets corresponding to other chunks in the document.
    chunk_offsets: ChunkOffsets,
}

/// Offsets corresponding to all other chunks in the document.
#[derive(Serialize, Deserialize, Debug, Constructor)]
pub struct ChunkOffsets {
    /// The start byte for the region where pages are stored.
    pages_offset: u64,
    /// The start byte for the region where the store is stored.
    store_offset: u64,
}

enum Compression {
    None,
}

impl HeaderChunk {
    pub fn new(file_len: u64, chunk_offsets: ChunkOffsets) -> Self {
        Self {
            magic_bytes: MAGIC_BYTES,
            version: 1,
            file_len,
            compression: Compression::None,
            chunk_offsets,
        }
    }
}
