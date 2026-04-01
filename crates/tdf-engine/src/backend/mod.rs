//! The backend layer: physical storage abstraction for all four TDF stores.
//!
//! [`Backend`] is the storage interface implemented by concrete backends like [`VecBackend`].
//! [`BackendView`] is the typed, offset-aware accessor that stores hold to communicate with
//! the backend without knowing about other stores' regions.

pub mod vec_backend;
pub use vec_backend::{VecBackend, VecRange};

use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::store::traits::StoreTypes;

/// Trait for combining unique data as a pointer chain is traversed.
pub trait UniqueReduce: Sized + Clone {
    fn reduce(self, other: Self) -> Self;
}

impl UniqueReduce for () {
    fn reduce(self, _other: Self) -> Self {}
}

impl crate::store::traits::UniqueType for () {}

/// The core reference type. Every item in every store is addressed by `BackendPointer<S, B>`.
///
/// Unique values are stored inline so they remain accessible generically (needed for `iter_rec`).

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(bound(
    serialize = "B: Serialize",
    deserialize = "B::Single<S>: Deserialize<'de>, B::Group<S>: Deserialize<'de>"
))]
pub enum BackendPointer<S: StoreTypes, B: BackendTypes>
{
    /// References a single item.
    Single (B::Single<S>),
    /// References a grouped range of items.
    Group (B::Group<S>),
}

// impl<S: StoreTypes, B: BackendTypes> BackendPointer<S, B> {
//     pub fn new_single(index: usize) -> Self
//     where
//         S::Unique: Default,
//     {
//         BackendPointer::Single(B::Single::default())
//     }
// }

// impl<S: StoreTypes, B: Backend> std::fmt::Debug for BackendPointer<S, B>
// where
//     S::Unique: std::fmt::Debug,
//     B::Range: std::fmt::Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             BackendPointer::Single { index, unique, .. } => f
//                 .debug_struct("Single")
//                 .field("index", index)
//                 .field("unique", unique)
//                 .finish(),
//             BackendPointer::Group { range, uniques, .. } => f
//                 .debug_struct("Group")
//                 .field("range", range)
//                 .field("uniques", uniques)
//                 .finish(),
//         }
//     }
// }

// impl<S: StoreTypes, B: Backend> Clone for BackendPointer<S, B>
// where
//     S::Unique: Clone,
//     B::Range: Clone,
// {
//     fn clone(&self) -> Self {
//         match self {
//             BackendPointer::Single { index, unique, .. } => BackendPointer::Single {
//                 index: *index,
//                 unique: unique.clone(),
//                 _phantom: PhantomData,
//             },
//             BackendPointer::Group { range, uniques, .. } => BackendPointer::Group {
//                 range: range.clone(),
//                 uniques: uniques.clone(),
//                 _phantom: PhantomData,
//             },
//         }
//     }
// }

// impl<S: StoreTypes, B: Backend> PartialEq for BackendPointer<S, B>
// where
//     S::Unique: PartialEq,
//     B::Range: PartialEq,
// {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (
//                 BackendPointer::Single { index: i1, unique: u1, .. },
//                 BackendPointer::Single { index: i2, unique: u2, .. },
//             ) => i1 == i2 && u1 == u2,
//             (
//                 BackendPointer::Group { range: r1, uniques: u1, .. },
//                 BackendPointer::Group { range: r2, uniques: u2, .. },
//             ) => r1 == r2 && u1 == u2,
//             _ => false,
//         }
//     }
// }

// impl<S: StoreTypes, B: Backend> Eq for BackendPointer<S, B>
// where
//     S::Unique: Eq,
//     B::Group: Eq,
// {
// }

// impl<S: StoreTypes, B: Backend> Hash for BackendPointer<S, B>
// where
//     S::Unique: Hash,
//     B::Group: Hash,
// {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         match self {
//             BackendPointer::Single { index, unique, .. } => {
//                 0u8.hash(state);
//                 index.hash(state);
//                 unique.hash(state);
//             }
//             BackendPointer::Group { range, uniques, .. } => {
//                 1u8.hash(state);
//                 range.hash(state);
//                 uniques.hash(state);
//             }
//         }
//     }
// }

/// What you get back when reading from any store.
#[derive(Serialize, Deserialize)]
#[serde(bound = "S: StoreTypes, B: BackendTypes")]
pub enum StoreItemCell<S: StoreTypes, B: BackendTypes>
{
    BackendPointer(BackendPointer<S, B>),
    StorePrimitive(S::Primitive),
}

impl<S: StoreTypes, B: BackendTypes> std::fmt::Debug for StoreItemCell<S, B>
where
    BackendPointer<S, B>: std::fmt::Debug,
    S::Primitive: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreItemCell::BackendPointer(p) => f.debug_tuple("BackendPointer").field(p).finish(),
            StoreItemCell::StorePrimitive(p) => f.debug_tuple("StorePrimitive").field(p).finish(),
        }
    }
}

impl<S: StoreTypes, B: BackendTypes> Clone for StoreItemCell<S, B>
where
    BackendPointer<S, B>: Clone,
    S::Primitive: Clone,
{
    fn clone(&self) -> Self {
        match self {
            StoreItemCell::BackendPointer(p) => StoreItemCell::BackendPointer(p.clone()),
            StoreItemCell::StorePrimitive(p) => StoreItemCell::StorePrimitive(p.clone()),
        }
    }
}

impl<S: StoreTypes, B: BackendTypes> PartialEq for StoreItemCell<S, B>
where
    BackendPointer<S, B>: PartialEq,
    S::Primitive: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StoreItemCell::BackendPointer(a), StoreItemCell::BackendPointer(b)) => a == b,
            (StoreItemCell::StorePrimitive(a), StoreItemCell::StorePrimitive(b)) => a == b,
            _ => false,
        }
    }
}

impl<S: StoreTypes, B: BackendTypes> Eq for StoreItemCell<S, B>
where
    BackendPointer<S, B>: Eq,
    S::Primitive: Eq,
{
}

/// A typed, offset-aware accessor into one region of the backend.
// #[derive(Debug)]
// pub struct BackendView<P, B> {
//     pub offset: usize,
//     _phantom: PhantomData<(P, B)>,
// }

// impl<P, B> BackendView<P, B> {
//     pub fn new(offset: usize) -> Self {
//         BackendView {
//             offset,
//             _phantom: PhantomData,
//         }
//     }
// }

/// Generic push/get bridge so Store impls can call the backend without
/// needing to know which concrete store region to use.
pub trait BackendAccess<S: StoreTypes, B: Backend>
{
    fn push_cell(&mut self, item: StoreItemCell<S, B::Types>) -> BackendPointer<S, B::Types>;
    fn get_cell(&self, pointer: &BackendPointer<S, B::Types>) -> Option<&StoreItemCell<S, B::Types>>;
    fn group_together(
        &mut self,
        items: Vec<BackendPointer<S, B::Types>>,
    ) -> BackendPointer<S, B::Types>
    where
        S::Unique: Default;
    fn expand_group(
        &self,
        range: &<B::Types as BackendTypes>::Group<S>,
    ) -> Vec<BackendPointer<S, B::Types>>;
}

pub trait BackendTypes: Hash + std::fmt::Debug + Clone + Eq + PartialEq + Serialize + for<'de> Deserialize<'de> {
    type Group<S: StoreTypes>: Serialize
        + for<'de> Deserialize<'de>
        + std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + Hash
        + Default;

    type Single<S: StoreTypes>: Serialize
        + for<'de> Deserialize<'de>
        + std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + Hash
        + Default;
}

/// Physical storage for all four TDF stores.
pub trait Backend: Sized {
    type Types: BackendTypes;
    /// The type used to reference a contiguous group of items (e.g., a range of indices).
    fn page_store_size(&self) -> usize;
    fn item_store_size(&self) -> usize;
    fn data_store_size(&self) -> usize;
    fn sig_store_size(&self) -> usize;
}
