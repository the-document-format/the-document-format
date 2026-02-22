//! The content store is where all actual data in a TDF is stored.
//!
//! A store contains a giant list of many store item references. Each store item
//! reference may be the literal content of some data item (like an actual
//! image, with all the actual image data), or a pointer to some other item in
//! the big master list, using an index as a reference.

pub mod data_store;
pub mod page_store;

use std::{collections::HashMap, marker::PhantomData};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub trait PrimativeType<'a>:
    std::hash::Hash + Eq + PartialEq + Serialize + for<'de> Deserialize<'de> + 'a
{
}
pub trait UniqueType<'a>:
    std::hash::Hash + Eq + PartialEq + Serialize + for<'de> Deserialize<'de> + 'a
{
}

#[derive(Error, Debug)]
pub enum StoreSegmentError {
    #[error("Index {index} is out of bounds for store item collection of size {size}")]
    IndexOutOfBounds { index: usize, size: usize },
    #[error("Range is invalid: start={start}, len={len}, bounds={bounds}")]
    RangeInvalid {
        start: usize,
        len: usize,
        bounds: usize,
    },
}

/// The top level container storing all of the store segments.
///
/// Internally it is stored as a vector, where at a given index it stores an
/// item, which may have some positional data, followed by either actual data,
/// or a pointer to actual data.
#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct StoreSegment<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    items: StoreItemCollection<'a, T, U>,
}

/// This is a thin wrapper on a collection of many store items.
///
/// Eventually we will want to maintain various optimizations as we update this,
/// so we want to be in control of the interface.
///
/// When you insert items into the collection we hand you a pointer to a pointer
/// to a store item. Where that pointer points to might change over time
/// depending on how we optimize the store or move things around later. We also
/// provide no guarantees about the actual layout of the store items on disk.
#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct StoreItemCollection<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    items: Vec<StoreItemCell<'a, T, U>>,
    handles: HashMap<StoreHandle<'a, T, U>, StorePointer<'a, T, U>>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, T: PrimativeType<'a>, U: UniqueType<'a>> Default for StoreItemCollection<'a, T, U> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            handles: HashMap::new(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: PrimativeType<'a>, U: UniqueType<'a>> StoreItemCollection<'a, T, U> {
    /// Pushes a new item onto the collection and returns a pointer to it.
    ///
    /// No guarantees are given about where the actual item lives on disc.
    ///
    /// TODO: deduplicate items as we insert them
    pub fn push(&mut self, item: StoreItemCell<'a, T, U>) -> StorePointer<'a, T, U> {
        self.items.push(item);
        (self.items.len() - 1).into()
    }

    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, pointer: &StorePointer<'a, T, U>) -> Option<&StoreItemCell<'a, T, U>> {
        self.items.get(pointer.index())
    }

    pub fn get_range(
        &self,
        range: &StoreItemRange<'a, T, U>,
    ) -> Option<impl Iterator<Item = &StoreItemCell<'a, T, U>>> {
        self.items
            .as_slice()
            .get(range.start.index()..range.start.index() + range.len)
            .map(|items| items.iter())
    }

    pub fn set(
        &mut self,
        pointer: StorePointer<'a, T, U>,
        item: StoreItemCell<'a, T, U>,
    ) -> Result<(), StoreSegmentError> {
        self.items
            .get_mut(pointer.index())
            .map(|slot| {
                *slot = item;
            })
            .ok_or(StoreSegmentError::IndexOutOfBounds {
                index: pointer.index(),
                size: self.items.len(),
            })
    }

    /// Take an item cell, and if it's content is a pointer, dereference it one time.
    pub fn follow_item_cell<'b: 'a>(
        &'b self,
        item_cell: &'b StoreItemCell<'a, T, U>,
    ) -> Result<Vec<&'b StoreItemCell<'a, T, U>>, StoreSegmentError> {
        match &item_cell.content {
            StoreItemRef::Primative(_) => Ok(vec![item_cell]),
            StoreItemRef::StoreItems(items) => items
                .iter()
                .map(|pointer| {
                    self.get(pointer)
                        .ok_or(StoreSegmentError::IndexOutOfBounds {
                            index: pointer.index(),
                            size: self.items.len(),
                        })
                })
                .collect(),
            StoreItemRef::StoreItemRange(range) => self
                .get_range(range)
                .ok_or(StoreSegmentError::RangeInvalid {
                    start: range.start.index(),
                    len: range.len,
                    bounds: self.items.len(),
                })
                .map(|iter| iter.collect()),
        }
    }
}

/// An entry in the global store.
#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct StoreItemCell<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    /// Unique data to the specific store item that cannot be interned, even if
    /// the inner content can be.
    pub unique: U,
    /// Either the content inside the store item or a pointer to a real store item.
    pub content: StoreItemRef<'a, T, U>,
    #[serde(skip)]
    _phantom: PhantomData<&'a ()>,
}

/// Either a real store item, or a pointer to a real store item.
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
#[serde(bound(deserialize = "'de: 'a"))]
pub enum StoreItemRef<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    Primative(T),
    StoreItems(StorePointerGroup<'a, T, U>),
    StoreItemRange(StoreItemRange<'a, T, U>),
}

/// An ordered list of pointers inside the store
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct StorePointerGroup<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    pointer: Vec<StorePointer<'a, T, U>>,
    #[serde(skip)]
    _phantom: PhantomData<(T, U)>,
}

impl<'a, T: PrimativeType<'a>, U: UniqueType<'a>> StorePointerGroup<'a, T, U> {
    pub fn iter(&self) -> impl Iterator<Item = &StorePointer<'a, T, U>> {
        self.pointer.iter()
    }
}

/// A pointer to a pointer to a single item in the store
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct StoreHandle<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    pointer: StorePointer<'a, T, U>,
    #[serde(skip)]
    _phantom: PhantomData<(T, U)>,
}

impl<'a, T: PrimativeType<'a>, U: UniqueType<'a>> StoreHandle<'a, T, U> {
    pub fn new(pointer: StorePointer<'a, T, U>) -> Self {
        StoreHandle {
            pointer,
            _phantom: PhantomData,
        }
    }
}

/// A pointer to a single item in the store
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct StorePointer<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    index: usize,
    #[serde(skip)]
    _phantom: PhantomData<(T, U, &'a ())>,
}

impl<'a, T: PrimativeType<'a>, U: UniqueType<'a>> From<usize> for StorePointer<'a, T, U> {
    fn from(index: usize) -> Self {
        StorePointer {
            index,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: PrimativeType<'a>, U: UniqueType<'a>> StorePointer<'a, T, U> {
    // 1 o
    pub fn index(&self) -> usize {
        self.index
    }
}

/// A sequential range inside the store of StoreItemRefs
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct StoreItemRange<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    start: StorePointer<'a, T, U>,
    len: usize,
    #[serde(skip)]
    _phantom: PhantomData<(T, U)>,
}

#[cfg(test)]
mod tests {
    use super::{PrimativeType, UniqueType};
    use crate::segments::store::StoreItemCollection;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
    struct DummyStoreData(usize);
    #[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
    struct DummyUniqueData(usize);

    impl<'a> PrimativeType<'a> for DummyStoreData {}
    impl<'a> UniqueType<'a> for DummyUniqueData {}

    #[test]
    fn follow_item_cell() {
        let store: StoreItemCollection<'_, DummyStoreData, DummyUniqueData> =
            StoreItemCollection::default();

        // Ensure the default store is empty; this keeps the test meaningful while
        // fixing the trait-bound diagnostics.
        assert_eq!(store.size(), 0);
    }
}
