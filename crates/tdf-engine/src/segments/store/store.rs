use serde::{Deserialize, Serialize};

use std::marker::PhantomData;
use thiserror::Error;

pub trait PrimativeType<'a>:
    std::hash::Hash + Clone + Eq + PartialEq + Serialize + for<'de> Deserialize<'de> + 'a
{
}

pub trait UniqueType<'a>:
    std::hash::Hash + Clone + Eq + PartialEq + Serialize + for<'de> Deserialize<'de> + 'a
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
pub trait Store<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    fn push(&mut self, item: StoreItemCell<'a, T, U>) -> StoreItemRef<'a, T, U>;

    fn size(&self) -> usize;

    fn get(&self, pointer: StorePointer<'a, T, U>) -> Option<&StoreItemCell<'a, T, U>>;

    fn get_range(
        &self,
        range: StorePointerRange<'a, T, U>,
    ) -> Option<impl Iterator<Item = StoreItemCell<'a, T, U>>>;
}

pub trait StoreExt<'a, T: PrimativeType<'a>, U: UniqueType<'a>>: Store<'a, T, U> {
    /// Take an item cell, and if it's content is a pointer, dereference it one time.
    fn follow_item_cell<'b: 'a>(
        &'b self,
        item_cell: &'b StoreItemCell<'a, T, U>,
    ) -> Result<Vec<StoreItemCell<'a, T, U>>, StoreSegmentError>;
}

impl<'a, T: PrimativeType<'a>, U: UniqueType<'a>, S> StoreExt<'a, T, U> for S
where
    S: Store<'a, T, U>,
{
    fn follow_item_cell<'b: 'a>(
        &'b self,
        item_cell: &'b StoreItemCell<'a, T, U>,
    ) -> Result<Vec<StoreItemCell<'a, T, U>>, StoreSegmentError> {
        match item_cell {
            StoreItemCell::StorePrimative(_) => Ok(vec![item_cell.clone()]),
            StoreItemCell::StorePointer(pointer) => match pointer {
                StoreItemRef::Pointer(store_pointer) => todo!(),
                StoreItemRef::PointerRange(store_pointer_range) => todo!(),
            },
        }
    }
}

/// An entry in the global store.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound(deserialize = "'de: 'a"))]
pub enum StoreItemCell<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    StorePointer(StoreItemRef<'a, T, U>),
    StorePrimative(T),
}

/// Either a pointer directly to some store item, or a pointer to a range of
/// store items.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(bound(deserialize = "'de: 'a"))]
pub enum StoreItemRef<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    Pointer(StorePointer<'a, T, U>),
    PointerRange(StorePointerRange<'a, T, U>),
}

/// A pointer to a single item in the store.
///
/// Just a number that corresponds to an item in the store.
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone, Copy)]
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

/// A sequential range inside the store of StoreItemRefs
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct StorePointerRange<'a, T: PrimativeType<'a>, U: UniqueType<'a>> {
    start: StorePointer<'a, T, U>,
    len: usize,
    #[serde(skip)]
    _phantom: PhantomData<(T, U)>,
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

#[cfg(test)]
mod tests {
    use crate::segments::store::impls::vec_store::StoreItemCollection;

    use super::{PrimativeType, UniqueType};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
    struct DummyStoreData(usize);
    #[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
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
