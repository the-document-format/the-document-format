use serde::{Deserialize, Serialize};

/// Unix timestamp in milliseconds.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instant(pub u64);

/// A reference to a page by number.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PageRef(pub u32);

/// A reference to a specific item on a page.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PageAnchor(pub usize);

/// A rolling hash value used for store integrity verification.
/// Computed by `StoreExt::checksum()` and stored in the header.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Hash(pub u64);
