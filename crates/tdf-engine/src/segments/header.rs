// TODO: we might be able to remove this new

use crate::misc::Hash;
use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

pub const MAGIC_BYTES: [u8; 6] = [b'T', b'R', b'E', b'V', b'D', b'F'];

#[derive(Serialize, Deserialize, Debug)]
pub struct HeaderSegment {
    pub magic_bytes: [u8; 6],
    pub version: u8,
    pub file_len: u64,
    pub compression: Compression,
    pub segment_offsets: SegmentOffsets,
    pub checksum: Hash,
}

#[derive(Serialize, Deserialize, Debug, Constructor)]
pub struct SegmentOffsets {
    pub meta_offset: u64,
    pub pages_offset: u64,
    pub store_offset: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Compression {
    None,
}

impl HeaderSegment {
    pub fn new(file_len: u64, segment_offsets: SegmentOffsets) -> Self {
        Self {
            magic_bytes: MAGIC_BYTES,
            version: 1,
            file_len,
            compression: Compression::None,
            segment_offsets,
            checksum: Hash::default(),
        }
    }
}
