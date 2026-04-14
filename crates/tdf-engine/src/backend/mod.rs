//! The backend layer: physical storage abstraction for all four TDF stores.
//!
//! [`Backend`] is the storage interface implemented by concrete backends like [`VecBackend`].
//! [`BackendView`] is the typed, offset-aware accessor that stores hold to communicate with
//! the backend without knowing about other stores' regions.

pub use crate::impls::vec::backend::{VecBackend, VecRange};

use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::store::traits::StoreTypes;

/// Trait for combining unique data as a pointer chain is traversed.
pub trait UniqueReduce: Sized + Clone {
    fn reduce(self, other: Self) -> Self;
}

/// Trait for extracting the unique value from a single pointer.
pub trait HasUnique<U> {
    fn unique(&self) -> U;
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

    /// Recursively collect all `(Primitive, Unique)` pairs reachable from `pointer`.
    fn iter_rec(&self, pointer: &BackendPointer<S, B::Types>) -> Vec<(S::Primitive, S::Unique)>
    where
        <B::Types as BackendTypes>::Single<S>: HasUnique<S::Unique>,
    {
        match pointer {
            BackendPointer::Single(s) => {
                let unique = s.unique();
                match self.get_cells(pointer) {
                    Some(cells) => cells
                        .into_iter()
                        .filter_map(|cell| match cell {
                            StoreItemCell::StorePrimitive(p) => Some((p.clone(), unique.clone())),
                            StoreItemCell::BackendPointer(inner) => {
                                self.iter_rec(inner).into_iter().next()
                            }
                        })
                        .collect(),
                    None => vec![],
                }
            }
            BackendPointer::Group(g) => self
                .expand_group(g)
                .into_iter()
                .flat_map(|ptr| self.iter_rec(&ptr))
                .collect(),
        }
    }
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
        + Default
        + HasUnique<S::Unique>;
}

/// Physical storage for all four TDF stores.
pub trait Backend:
    Sized
    + GetStore<Self::PageStore>
    + GetStore<Self::ItemStore>
    + GetStore<Self::DataStore>
    + GetStore<Self::SigStore>
{
    type Types: BackendTypes;

    type PageStore;
    type ItemStore;
    type DataStore;
    type SigStore;
}
