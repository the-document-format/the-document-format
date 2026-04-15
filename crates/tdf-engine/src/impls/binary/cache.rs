//! LRU cache for deserialized store cells.

use schnellru::{ByLength, LruMap};

use crate::backend::StoreItemCell;
use crate::primitives::data::DataTypes;
use crate::primitives::item::ItemTypes;
use crate::primitives::page::PageTypes;
use crate::primitives::signature::SignatureTypes;
use crate::store::traits::StoreTypes;

use super::backend::{BinaryTypes, Offset};

/// Identifies which of the four stores a cache entry belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoreKind {
    PageStore,
    ItemStore,
    DataStore,
    SigStore,
}

/// Cache lookup key: (store, byte offset). Unique value is excluded from the key
/// because the same record may be referenced with different uniques by different pointers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackendCacheKey {
    pub store: StoreKind,
    pub offset: Offset,
}

/// Typed union of the four concrete cell types.
pub enum BackendCacheValue {
    PageStore(StoreItemCell<PageTypes<BinaryTypes>, BinaryTypes>),
    ItemStore(StoreItemCell<ItemTypes<BinaryTypes>, BinaryTypes>),
    DataStore(StoreItemCell<DataTypes, BinaryTypes>),
    SigStore(StoreItemCell<SignatureTypes, BinaryTypes>),
}

// TODO: we should probably get rid of this BS trait that is very redundant

impl BackendCacheValue {
    pub fn as_store<S: BinaryCacheExtract>(&self) -> Option<&StoreItemCell<S, BinaryTypes>> {
        S::extract(self)
    }
}

/// Sealed trait mapping each concrete store type to its `BackendCacheValue` variant.
pub trait BinaryCacheExtract: StoreTypes {
    const KIND: StoreKind;
    fn extract(value: &BackendCacheValue) -> Option<&StoreItemCell<Self, BinaryTypes>>;
    fn wrap(cell: StoreItemCell<Self, BinaryTypes>) -> BackendCacheValue;
}

impl BinaryCacheExtract for PageTypes<BinaryTypes> {
    const KIND: StoreKind = StoreKind::PageStore;
    fn extract(value: &BackendCacheValue) -> Option<&StoreItemCell<Self, BinaryTypes>> {
        match value {
            BackendCacheValue::PageStore(cell) => Some(cell),
            _ => None,
        }
    }
    fn wrap(cell: StoreItemCell<Self, BinaryTypes>) -> BackendCacheValue {
        BackendCacheValue::PageStore(cell)
    }
}

impl BinaryCacheExtract for ItemTypes<BinaryTypes> {
    const KIND: StoreKind = StoreKind::ItemStore;
    fn extract(value: &BackendCacheValue) -> Option<&StoreItemCell<Self, BinaryTypes>> {
        match value {
            BackendCacheValue::ItemStore(cell) => Some(cell),
            _ => None,
        }
    }
    fn wrap(cell: StoreItemCell<Self, BinaryTypes>) -> BackendCacheValue {
        BackendCacheValue::ItemStore(cell)
    }
}

impl BinaryCacheExtract for DataTypes {
    const KIND: StoreKind = StoreKind::DataStore;
    fn extract(value: &BackendCacheValue) -> Option<&StoreItemCell<Self, BinaryTypes>> {
        match value {
            BackendCacheValue::DataStore(cell) => Some(cell),
            _ => None,
        }
    }
    fn wrap(cell: StoreItemCell<Self, BinaryTypes>) -> BackendCacheValue {
        BackendCacheValue::DataStore(cell)
    }
}

impl BinaryCacheExtract for SignatureTypes {
    const KIND: StoreKind = StoreKind::SigStore;
    fn extract(value: &BackendCacheValue) -> Option<&StoreItemCell<Self, BinaryTypes>> {
        match value {
            BackendCacheValue::SigStore(cell) => Some(cell),
            _ => None,
        }
    }
    fn wrap(cell: StoreItemCell<Self, BinaryTypes>) -> BackendCacheValue {
        BackendCacheValue::SigStore(cell)
    }
}

// TODO: make this configurable when you are setting up a backend. Maybe you
// provide the cache to the new method of the binary backend so you have more
// control?

const DEFAULT_CACHE_CAPACITY: u32 = 1024;

/// Thin wrapper around an LRU map for deserialized store cells.
pub struct TdfBinCache {
    lru: LruMap<BackendCacheKey, BackendCacheValue, ByLength>,
}

impl TdfBinCache {
    pub fn new(capacity: u32) -> Self {
        Self {
            lru: LruMap::new(ByLength::new(capacity)),
        }
    }

    pub fn contains(&self, key: &BackendCacheKey) -> bool {
        self.lru.peek(key).is_some()
    }

    /// Immutable peek — does not update LRU order.
    pub fn peek(&self, key: &BackendCacheKey) -> Option<&BackendCacheValue> {
        self.lru.peek(key)
    }

    /// Mutable get — updates LRU order.
    pub fn get(&mut self, key: &BackendCacheKey) -> Option<&BackendCacheValue> {
        self.lru.get(key).map(|v| &*v)
    }

    pub fn insert(&mut self, key: BackendCacheKey, value: BackendCacheValue) {
        self.lru.insert(key, value);
    }
}

impl Default for TdfBinCache {
    fn default() -> Self {
        Self::new(DEFAULT_CACHE_CAPACITY)
    }
}

impl std::fmt::Debug for TdfBinCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TdfBinCache")
            .field("len", &self.lru.len())
            .finish()
    }
}
