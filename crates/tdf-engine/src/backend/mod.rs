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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
#[serde(bound(
    serialize = "B: Serialize",
    deserialize = "B::Single<S>: Deserialize<'de>, B::Group<S>: Deserialize<'de>"
))]
pub enum BackendPointer<S: StoreTypes, B: BackendTypes> {
    /// References a single item.
    Single(B::Single<S>),
    /// References a grouped range of items.
    Group(B::Group<S>),
}

impl<S: StoreTypes, B: BackendTypes> BackendPointer<S, B> {
    pub fn new_single(index: usize) -> Self
    where
        S::Unique: Default,
    {
        BackendPointer::Single(B::Single::default())
    }
}

/// What you get back when reading from any store.
#[derive(Serialize, Deserialize)]
#[serde(bound = "S: StoreTypes, B: BackendTypes")]
pub enum StoreItemCell<S: StoreTypes, B: BackendTypes> {
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

/// Generic push/get bridge so Store impls can call the backend without
/// needing to know which concrete store region to use.
pub trait BackendAccess<S: StoreTypes, B: Backend> {
    fn push_cell(
        &mut self,
        primitive: S::Primitive,
        unique: S::Unique,
    ) -> BackendPointer<S, B::Types>;

    fn get_cells(
        &self,
        pointer: &BackendPointer<S, B::Types>,
        // TODO: make this a lazy iterator not a vec
    ) -> Option<Vec<&StoreItemCell<S, B::Types>>>;

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

pub trait GetStore<Q> {
    fn get_store(&self) -> &Q;
    fn get_store_mut(&mut self) -> &mut Q;
}

pub trait BackendTypes:
    Hash + std::fmt::Debug + Clone + Eq + PartialEq + Serialize + for<'de> Deserialize<'de>
{
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
}
