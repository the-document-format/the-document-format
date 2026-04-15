//! The backend layer: physical storage abstraction for all four TDF stores.
//!
//! [`Backend`] is the storage interface implemented by concrete backends like [`VecBackend`].
//! [`BackendView`] is the typed, offset-aware accessor that stores hold to communicate with
//! the backend without knowing about other stores' regions.

pub use crate::impls::vec::backend::{VecBackend, VecRange};

use std::borrow::Cow;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::impls::binary::error::TdfBinaryError;
use crate::store::traits::{StoreTypes, UniqueType};

/// Hint for whether `get_cells` should use the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheHints {
    Cache,
    NoCache,
}

/// Trait for combining unique data as a pointer chain is traversed.
pub trait UniqueReduce: Sized + Clone {
    fn reduce(self, other: Self) -> Self;
}

/// Trait for a single-item pointer type. Carries one unique value.
pub trait SinglePointerType<U>:
    Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone + PartialEq + Eq + Hash + Default
{
    fn unique(&self) -> U;
}

/// Trait for a group pointer type. Carries one unique value per item.
pub trait GroupPointerType<U>:
    Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone + PartialEq + Eq + Hash + Default
{
    fn uniques(&self) -> Vec<U>;
}

impl UniqueReduce for () {
    fn reduce(self, _other: Self) -> Self {}
}

impl UniqueType for () {}

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

    /// Returns all unique values carried by this pointer.
    /// Single pointers return a one-element vec; groups return one per item.
    pub fn uniques(&self) -> Vec<S::Unique> {
        match self {
            BackendPointer::Single(s) => vec![s.unique()],
            BackendPointer::Group(g) => g.uniques(),
        }
    }
}

/// What you get back when reading from any store.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(bound = "S: StoreTypes, B: BackendTypes")]
pub enum StoreItemCell<S: StoreTypes, B: BackendTypes> {
    BackendPointer(BackendPointer<S, B>),
    StorePrimitive(S::Primitive),
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

    fn get_cells<'a>(
        &'a mut self,
        pointer: &BackendPointer<S, B::Types>,
        hints: CacheHints,
    ) -> Result<Vec<Cow<'a, StoreItemCell<S, B::Types>>>, TdfBinaryError>;

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
    type Group<S: StoreTypes>: GroupPointerType<S::Unique>;
    type Single<S: StoreTypes>: SinglePointerType<S::Unique>;
}

use crate::primitives::data::DataTypes;
use crate::primitives::item::ItemTypes;
use crate::primitives::page::PageTypes;
use crate::primitives::signature::SignatureTypes;

/// Physical storage for all four TDF stores.
pub trait Backend:
    Sized
    + GetStore<Self::PageStore>
    + GetStore<Self::ItemStore>
    + GetStore<Self::DataStore>
    + GetStore<Self::SigStore>
    + BackendAccess<PageTypes<Self::Types>, Self>
    + BackendAccess<ItemTypes<Self::Types>, Self>
    + BackendAccess<DataTypes, Self>
    + BackendAccess<SignatureTypes, Self>
{
    type Types: BackendTypes;

    type PageStore;
    type ItemStore;
    type DataStore;
    type SigStore;
}
