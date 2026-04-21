//! Binary file header.

use serde::{Deserialize, Serialize};

pub const MAGIC: [u8; 6] = *b"TREVDF";
pub const CURRENT_VERSION: u8 = 1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BinaryFileHeader {
    pub magic: [u8; 6],
    pub version: u8,
    pub file_len: u64,
    pub meta_offset: u64,
    pub pages_offset: u64,
    pub page_store_offset: u64,
    pub item_store_offset: u64,
    pub data_store_offset: u64,
    pub sig_store_offset: u64,
}

impl Default for BinaryFileHeader {
    fn default() -> Self {
        Self {
            magic: MAGIC,
            version: CURRENT_VERSION,
            file_len: 0,
            meta_offset: 0,
            pages_offset: 0,
            page_store_offset: 0,
            item_store_offset: 0,
            data_store_offset: 0,
            sig_store_offset: 0,
        }
    }
}
