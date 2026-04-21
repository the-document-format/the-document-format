//! BinaryBackend — stores four byte buffers + LRU cache.

use std::borrow::Cow;

use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::backend::{
    Backend, BackendAccess, BackendPointer, BackendTypes, CacheHints, GetStore, GroupPointerType,
    SinglePointerType, StoreItemCell,
};
use crate::primitives::data::DataTypes;
use crate::primitives::item::ItemTypes;
use crate::primitives::page::PageTypes;
use crate::primitives::signature::SignatureTypes;
use crate::store::traits::StoreTypes;

use super::cache::{BackendCacheKey, BinaryCacheExtract, TdfBinCache};
use super::error::TdfBinaryError;

// ---------------------------------------------------------------------------
// Offset newtype
// ---------------------------------------------------------------------------

// TODO: we should add an accessor method
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Constructor)]
pub struct Offset(pub u64);

// ---------------------------------------------------------------------------
// Pointer types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinarySinglePointer<S: StoreTypes> {
    pub offset: Offset,
    pub unique: S::Unique,
}

impl<S: StoreTypes> SinglePointerType<S::Unique> for BinarySinglePointer<S> {
    fn unique(&self) -> S::Unique {
        self.unique.clone()
    }
}

impl<S: StoreTypes> Default for BinarySinglePointer<S> {
    fn default() -> Self {
        Self {
            offset: Offset(0),
            unique: S::Unique::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinaryGroupPointer<S: StoreTypes> {
    pub start: Offset,
    pub len: u32,
    pub uniques: Vec<S::Unique>,
}

impl<S: StoreTypes> GroupPointerType<S::Unique> for BinaryGroupPointer<S> {
    fn uniques(&self) -> Vec<S::Unique> {
        self.uniques.clone()
    }
}

impl<S: StoreTypes> Default for BinaryGroupPointer<S> {
    fn default() -> Self {
        Self {
            start: Offset(0),
            len: 0,
            uniques: vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// BinaryTypes marker
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct BinaryTypes;

impl BackendTypes for BinaryTypes {
    type Single<S: StoreTypes> = BinarySinglePointer<S>;
    type Group<S: StoreTypes> = BinaryGroupPointer<S>;
}

// ---------------------------------------------------------------------------
// Store byte newtypes (so GetStore<Q> can dispatch)
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct PageStoreBytes(pub Vec<u8>);
#[derive(Debug, Default)]
pub struct ItemStoreBytes(pub Vec<u8>);
#[derive(Debug, Default)]
pub struct DataStoreBytes(pub Vec<u8>);
#[derive(Debug, Default)]
pub struct SigStoreBytes(pub Vec<u8>);

// ---------------------------------------------------------------------------
// BinaryBackend
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct BinaryBackend {
    page_store: PageStoreBytes,
    item_store: ItemStoreBytes,
    data_store: DataStoreBytes,
    sig_store: SigStoreBytes,
    cache: TdfBinCache,
}

impl BinaryBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_store_bytes(page: Vec<u8>, item: Vec<u8>, data: Vec<u8>, sig: Vec<u8>) -> Self {
        Self {
            page_store: PageStoreBytes(page),
            item_store: ItemStoreBytes(item),
            data_store: DataStoreBytes(data),
            sig_store: SigStoreBytes(sig),
            cache: TdfBinCache::default(),
        }
    }

    pub fn page_store_bytes(&self) -> &[u8] {
        &self.page_store.0
    }
    pub fn item_store_bytes(&self) -> &[u8] {
        &self.item_store.0
    }
    pub fn data_store_bytes(&self) -> &[u8] {
        &self.data_store.0
    }
    pub fn sig_store_bytes(&self) -> &[u8] {
        &self.sig_store.0
    }
    pub fn page_store_len(&self) -> usize {
        self.page_store.0.len()
    }
    pub fn item_store_len(&self) -> usize {
        self.item_store.0.len()
    }
    pub fn data_store_len(&self) -> usize {
        self.data_store.0.len()
    }
    pub fn sig_store_len(&self) -> usize {
        self.sig_store.0.len()
    }
}

impl Default for BinaryBackend {
    fn default() -> Self {
        Self {
            page_store: PageStoreBytes::default(),
            item_store: ItemStoreBytes::default(),
            data_store: DataStoreBytes::default(),
            sig_store: SigStoreBytes::default(),
            cache: TdfBinCache::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// GetStore impls
// ---------------------------------------------------------------------------

macro_rules! impl_get_store_binary {
    ($store_ty:ty, $field:ident) => {
        impl GetStore<$store_ty> for BinaryBackend {
            fn get_store(&self) -> &$store_ty {
                &self.$field
            }
            fn get_store_mut(&mut self) -> &mut $store_ty {
                &mut self.$field
            }
        }
    };
}

impl_get_store_binary!(PageStoreBytes, page_store);
impl_get_store_binary!(ItemStoreBytes, item_store);
impl_get_store_binary!(DataStoreBytes, data_store);
impl_get_store_binary!(SigStoreBytes, sig_store);

impl Backend for BinaryBackend {
    type Types = BinaryTypes;
    type PageStore = PageStoreBytes;
    type ItemStore = ItemStoreBytes;
    type DataStore = DataStoreBytes;
    type SigStore = SigStoreBytes;
}

// ---------------------------------------------------------------------------
// BinaryStoreAccess — maps StoreTypes → the right Vec<u8> field
// ---------------------------------------------------------------------------

pub trait BinaryStoreAccess: StoreTypes + BinaryCacheExtract {
    fn store_bytes(backend: &BinaryBackend) -> &Vec<u8>;
    fn store_bytes_mut(backend: &mut BinaryBackend) -> &mut Vec<u8>;
}

macro_rules! impl_binary_store_access {
    ($store_types:ty, $field:ident) => {
        impl BinaryStoreAccess for $store_types {
            fn store_bytes(backend: &BinaryBackend) -> &Vec<u8> {
                &backend.$field.0
            }
            fn store_bytes_mut(backend: &mut BinaryBackend) -> &mut Vec<u8> {
                &mut backend.$field.0
            }
        }
    };
}

impl_binary_store_access!(PageTypes<BinaryTypes>, page_store);
impl_binary_store_access!(ItemTypes<BinaryTypes>, item_store);
impl_binary_store_access!(DataTypes, data_store);
impl_binary_store_access!(SignatureTypes, sig_store);

// ---------------------------------------------------------------------------
// BackendAccess — single generic impl for all four stores
// ---------------------------------------------------------------------------

impl<S> BackendAccess<S, BinaryBackend> for BinaryBackend
where
    S: BinaryStoreAccess,
    StoreItemCell<S, BinaryTypes>: Serialize + for<'de> Deserialize<'de>,
{
    fn push_cell(
        &mut self,
        primitive: S::Primitive,
        unique: S::Unique,
    ) -> BackendPointer<S, BinaryTypes> {
        let store = S::store_bytes_mut(self);
        let offset = Offset(store.len() as u64);
        let cell = StoreItemCell::<S, BinaryTypes>::StorePrimitive(primitive);
        let config = super::bincode_config();
        let encoded = bincode_next::serde::encode_to_vec(&cell, config)
            .expect("bincode encode should not fail for in-memory push");
        store.extend_from_slice(&encoded);

        // Also insert into cache
        let key = BackendCacheKey {
            store: S::KIND,
            offset,
        };
        self.cache.insert(key, S::wrap(cell));

        BackendPointer::Single(BinarySinglePointer { offset, unique })
    }

    fn get_cells<'a>(
        &'a mut self,
        pointer: &BackendPointer<S, BinaryTypes>,
        hints: CacheHints,
    ) -> Result<Vec<Cow<'a, StoreItemCell<S, BinaryTypes>>>, TdfBinaryError> {
        match pointer {
            BackendPointer::Single(single) => {
                let offset = single.offset;
                match hints {
                    CacheHints::Cache => self.get_cell_cached::<S>(offset),
                    CacheHints::NoCache => self.get_cell_uncached::<S>(offset),
                }
                .map(|cell| vec![cell])
            }
            BackendPointer::Group(group) => {
                let mut cells = Vec::with_capacity(group.len as usize);
                let config = super::bincode_config();
                let store = S::store_bytes(self);
                let mut cursor = group.start.0 as usize;
                for _ in 0..group.len {
                    if cursor >= store.len() {
                        return Err(TdfBinaryError::InvalidPointerRef);
                    }
                    let (cell, consumed): (StoreItemCell<S, BinaryTypes>, usize) =
                        bincode_next::serde::decode_from_slice(&store[cursor..], config)?;
                    cells.push(Cow::Owned(cell));
                    cursor += consumed;
                }
                Ok(cells)
            }
        }
    }

    fn group_together(
        &mut self,
        items: Vec<BackendPointer<S, BinaryTypes>>,
    ) -> BackendPointer<S, BinaryTypes>
    where
        S::Unique: Default,
    {
        if items.is_empty() {
            return BackendPointer::Group(BinaryGroupPointer::default());
        }

        let start = match &items[0] {
            BackendPointer::Single(s) => s.offset,
            BackendPointer::Group(_) => todo!("nested recursive groups in group_together"),
        };

        let uniques = items
            .iter()
            .map(|ptr| match ptr {
                BackendPointer::Single(s) => s.unique.clone(),
                BackendPointer::Group(_) => todo!("nested recursive groups in group_together"),
            })
            .collect();

        BackendPointer::Group(BinaryGroupPointer {
            start,
            len: items.len() as u32,
            uniques,
        })
    }

    fn expand_group(&self, group: &BinaryGroupPointer<S>) -> Vec<BackendPointer<S, BinaryTypes>> {
        let config = super::bincode_config();
        let store = S::store_bytes(self);
        let mut cursor = group.start.0 as usize;
        let mut ptrs = Vec::with_capacity(group.len as usize);

        for (i, unique) in group.uniques.iter().enumerate() {
            let offset = Offset(cursor as u64);
            // Advance cursor past this record
            if cursor < store.len() {
                if let Ok((_, consumed)) = bincode_next::serde::decode_from_slice::<
                    StoreItemCell<S, BinaryTypes>,
                    _,
                >(&store[cursor..], config)
                {
                    cursor += consumed;
                }
            }
            ptrs.push(BackendPointer::Single(BinarySinglePointer {
                offset,
                unique: unique.clone(),
            }));
        }

        ptrs
    }
}

impl BinaryBackend {
    fn get_cell_cached<'a, S: BinaryStoreAccess>(
        &'a mut self,
        offset: Offset,
    ) -> Result<Cow<'a, StoreItemCell<S, BinaryTypes>>, TdfBinaryError>
    where
        StoreItemCell<S, BinaryTypes>: Serialize + for<'de> Deserialize<'de>,
    {
        let key = BackendCacheKey {
            store: S::KIND,
            offset,
        };

        // Populate cache if not present (peek uses &self, no mutable borrow conflict)
        if !self.cache.contains(&key) {
            // TODO: this could be the else clause of the proceeding if statement

            let store = S::store_bytes(self);
            let off = offset.0 as usize;
            if off >= store.len() {
                return Err(TdfBinaryError::InvalidPointerRef);
            }

            let config = super::bincode_config();
            let (cell, _): (StoreItemCell<S, BinaryTypes>, usize) =
                bincode_next::serde::decode_from_slice(&store[off..], config)?;
            self.cache.insert(key.clone(), S::wrap(cell));
        }

        // Peek from cache (immutable borrow — no conflict with other self access).
        // We just ensured it's in the cache above. peek doesn't update LRU order
        // but the item was just inserted/is already at the front.
        if let Some(cell) = self.cache.peek(&key).and_then(|v| v.as_store::<S>()) {
            return Ok(Cow::Borrowed(cell));
        }

        // Cache invariant violated or evicted — fallback to uncached
        debug_assert!(
            false,
            "cache invariant violated: value missing after insert"
        );
        self.get_cell_uncached(offset)
    }

    fn get_cell_uncached<'a, S: BinaryStoreAccess>(
        &'a self,
        offset: Offset,
    ) -> Result<Cow<'a, StoreItemCell<S, BinaryTypes>>, TdfBinaryError>
    where
        StoreItemCell<S, BinaryTypes>: for<'de> Deserialize<'de>,
    {
        let store = S::store_bytes(self);
        let off = offset.0 as usize;
        if off >= store.len() {
            return Err(TdfBinaryError::InvalidPointerRef);
        }
        let config = super::bincode_config();
        let (cell, _): (StoreItemCell<S, BinaryTypes>, usize) =
            bincode_next::serde::decode_from_slice(&store[off..], config)?;
        Ok(Cow::Owned(cell))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{BackendAccess, CacheHints};
    use crate::primitives::item::{
        ItemPrimitive, ItemTypes, ItemUnique, Position, Shape, ShapeKind,
    };

    fn make_backend_with_item() -> (
        BinaryBackend,
        BackendPointer<ItemTypes<BinaryTypes>, BinaryTypes>,
    ) {
        let mut backend = BinaryBackend::new();
        let ptr =
            <BinaryBackend as BackendAccess<ItemTypes<BinaryTypes>, BinaryBackend>>::push_cell(
                &mut backend,
                ItemPrimitive::Shape(Shape {
                    kind: ShapeKind::Circle,
                }),
                ItemUnique {
                    position: Position { x: 1, y: 2 },
                    ..Default::default()
                },
            );
        (backend, ptr)
    }

    #[test]
    fn cache_hint_cache_returns_borrowed() {
        let (mut backend, ptr) = make_backend_with_item();
        let cells =
            <BinaryBackend as BackendAccess<ItemTypes<BinaryTypes>, BinaryBackend>>::get_cells(
                &mut backend,
                &ptr,
                CacheHints::Cache,
            )
            .expect("get_cells should succeed");
        assert_eq!(cells.len(), 1);
        assert!(
            matches!(cells[0], Cow::Borrowed(_)),
            "Cache hint should return Cow::Borrowed"
        );
    }

    #[test]
    fn cache_hint_nocache_returns_owned() {
        let (mut backend, ptr) = make_backend_with_item();
        let cells =
            <BinaryBackend as BackendAccess<ItemTypes<BinaryTypes>, BinaryBackend>>::get_cells(
                &mut backend,
                &ptr,
                CacheHints::NoCache,
            )
            .expect("get_cells should succeed");
        assert_eq!(cells.len(), 1);
        assert!(
            matches!(cells[0], Cow::Owned(_)),
            "NoCache hint should return Cow::Owned"
        );
    }

    #[test]
    fn cache_hit_returns_same_data() {
        let (mut backend, ptr) = make_backend_with_item();

        // First access — cache miss, populates cache
        let cells1 =
            <BinaryBackend as BackendAccess<ItemTypes<BinaryTypes>, BinaryBackend>>::get_cells(
                &mut backend,
                &ptr,
                CacheHints::Cache,
            )
            .expect("first get_cells should succeed");
        let cell1 = cells1[0].clone().into_owned();

        // Second access — cache hit
        let cells2 =
            <BinaryBackend as BackendAccess<ItemTypes<BinaryTypes>, BinaryBackend>>::get_cells(
                &mut backend,
                &ptr,
                CacheHints::Cache,
            )
            .expect("second get_cells should succeed");
        assert!(
            matches!(cells2[0], Cow::Borrowed(_)),
            "second call should be cache hit (Borrowed)"
        );
        let cell2 = cells2[0].clone().into_owned();

        assert_eq!(cell1, cell2, "cached and uncached should return same data");
    }

    #[test]
    fn push_cell_and_get_cells_roundtrip() {
        let (mut backend, ptr) = make_backend_with_item();
        let cells =
            <BinaryBackend as BackendAccess<ItemTypes<BinaryTypes>, BinaryBackend>>::get_cells(
                &mut backend,
                &ptr,
                CacheHints::Cache,
            )
            .expect("get_cells should succeed");
        assert_eq!(cells.len(), 1);
        match cells[0].as_ref() {
            StoreItemCell::StorePrimitive(ItemPrimitive::Shape(s)) => {
                assert_eq!(s.kind, ShapeKind::Circle);
            }
            other => panic!("expected Shape(Circle), got {:?}", other),
        }
    }

    #[test]
    fn group_together_and_expand() {
        let mut backend = BinaryBackend::new();

        let u0 = ItemUnique {
            position: Position { x: 1, y: 2 },
            ..Default::default()
        };
        let u1 = ItemUnique {
            position: Position { x: 3, y: 4 },
            ..Default::default()
        };

        let ptr0 =
            <BinaryBackend as BackendAccess<ItemTypes<BinaryTypes>, BinaryBackend>>::push_cell(
                &mut backend,
                ItemPrimitive::Shape(Shape {
                    kind: ShapeKind::Circle,
                }),
                u0.clone(),
            );
        let ptr1 =
            <BinaryBackend as BackendAccess<ItemTypes<BinaryTypes>, BinaryBackend>>::push_cell(
                &mut backend,
                ItemPrimitive::Shape(Shape {
                    kind: ShapeKind::Rectangle,
                }),
                u1.clone(),
            );

        let group =
            <BinaryBackend as BackendAccess<ItemTypes<BinaryTypes>, BinaryBackend>>::group_together(
                &mut backend,
                vec![ptr0, ptr1],
            );

        match &group {
            BackendPointer::Group(g) => {
                assert_eq!(g.len, 2);
                assert_eq!(g.uniques[0], u0);
                assert_eq!(g.uniques[1], u1);
            }
            _ => panic!("expected Group pointer"),
        }

        let expanded =
            <BinaryBackend as BackendAccess<ItemTypes<BinaryTypes>, BinaryBackend>>::expand_group(
                &backend,
                match &group {
                    BackendPointer::Group(g) => g,
                    _ => unreachable!(),
                },
            );
        assert_eq!(expanded.len(), 2);
    }

    #[test]
    fn invalid_pointer_returns_error() {
        let mut backend = BinaryBackend::new();
        let bad_ptr =
            BackendPointer::<ItemTypes<BinaryTypes>, BinaryTypes>::Single(BinarySinglePointer {
                offset: Offset(9999),
                unique: ItemUnique::default(),
            });
        let result =
            <BinaryBackend as BackendAccess<ItemTypes<BinaryTypes>, BinaryBackend>>::get_cells(
                &mut backend,
                &bad_ptr,
                CacheHints::Cache,
            );
        assert!(result.is_err());
    }
}
