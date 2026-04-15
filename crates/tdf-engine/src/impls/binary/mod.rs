//! Binary backend — compact, seekable binary file format.

pub mod backend;
pub mod builder;
pub mod cache;
pub mod document;
pub mod error;
pub mod header;

pub use backend::{BinaryBackend, BinaryGroupPointer, BinarySinglePointer, BinaryTypes, Offset};
pub use builder::BinaryTDFBuilder;
pub use cache::{BackendCacheKey, BackendCacheValue, BinaryCacheExtract, StoreKind, TdfBinCache};
pub use error::TdfBinaryError;
pub use header::BinaryFileHeader;

fn bincode_config() -> impl bincode_next::config::Config {
    bincode_next::config::standard()
}

fn bincode_header_config() -> impl bincode_next::config::Config {
    bincode_next::config::standard().with_fixed_int_encoding()
}
