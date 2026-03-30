//! The backend layer: physical storage abstraction for all four TDF stores.
//!
//! [`Backend`] is the storage interface implemented by concrete backends like [`VecBackend`].
//! [`BackendView`] is the typed, offset-aware accessor that stores hold to communicate with
//! the backend without knowing about other stores' regions.

pub mod vec_backend;
pub use vec_backend::{VecBackend, VecRange};

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::primitives::{
    data::DataPrimitive,
    item::{ItemPrimitive, ItemUnique},
    page::ItemPointer,
    signature::{SignaturePrimitive, SignatureUnique},
};

/// Trait for combining unique data as a pointer chain is traversed.
pub trait UniqueReduce: Sized + Clone {
    fn reduce(self, other: Self) -> Self;
}

impl UniqueReduce for () {
    fn reduce(self, _other: Self) -> Self {}
}

impl crate::store::traits::UniqueType for () {}

/// The core reference type. Every item in every store is addressed by `BackendPointer<T, U, G>`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BackendPointer<T, U, G> {
    /// References a single item.
    Single {
        index: usize,
        unique: U,
        #[serde(skip)]
        _phantom: PhantomData<T>,
    },
    /// References a grouped range of items.
    Group {
        group: G,
        unique: U,
        #[serde(skip)]
        _phantom: PhantomData<T>,
    },
}

impl<T: PartialEq + Eq, U: Ord, G: Ord + Eq + PartialEq> PartialOrd for BackendPointer<T, U, G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: PartialEq + Eq, U: Ord, G: Ord + Eq + PartialEq> Ord for BackendPointer<T, U, G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        todo!()
    }
}

impl<T, U: Default, G> BackendPointer<T, U, G> {
    pub fn new(index: usize) -> Self {
        BackendPointer::Single {
            index,
            unique: U::default(),
            _phantom: PhantomData,
        }
    }
}

/// What you get back when reading from any store.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum StoreItemCell<T, U, G> {
    BackendPointer(BackendPointer<T, U, G>),
    StorePrimitive(T),
}

/// A typed, offset-aware accessor into one region of the backend.
#[derive(Debug)]
pub struct BackendView<P, B: Backend> {
    pub offset: usize,
    _phantom: PhantomData<(P, B)>,
}

impl<P, B: Backend> BackendView<P, B> {
    pub fn new(offset: usize) -> Self {
        BackendView {
            offset,
            _phantom: PhantomData,
        }
    }
}

/// Generic push/get bridge so Store impls can call the backend without
/// needing to know which concrete method (push_page, push_item, …) to use.
pub trait BackendAccess<P, U> {
    type Group: Clone;
    fn push_cell(
        &mut self,
        item: StoreItemCell<P, U, Self::Group>,
    ) -> BackendPointer<P, U, Self::Group>;
    fn get_cell(
        &self,
        pointer: &BackendPointer<P, U, Self::Group>,
    ) -> Option<&StoreItemCell<P, U, Self::Group>>;
    fn group_together(
        &mut self,
        items: Vec<BackendPointer<P, U, Self::Group>>,
    ) -> BackendPointer<P, U, Self::Group>
    where
        U: Default;
    fn expand_group(
        &self,
        group: &Self::Group,
        unique: U,
    ) -> Vec<BackendPointer<P, U, Self::Group>>;
}

/// Physical storage for all four TDF stores.
pub trait Backend {
    fn push_page(
        &mut self,
        item: StoreItemCell<ItemPointer, (), VecRange>,
    ) -> BackendPointer<ItemPointer, (), VecRange>;
    fn get_page(
        &self,
        pointer: &BackendPointer<ItemPointer, (), VecRange>,
    ) -> Option<&StoreItemCell<ItemPointer, (), VecRange>>;
    fn page_store_size(&self) -> usize;

    fn push_item(
        &mut self,
        item: StoreItemCell<ItemPrimitive, ItemUnique, VecRange>,
    ) -> BackendPointer<ItemPrimitive, ItemUnique, VecRange>;
    fn get_item(
        &self,
        pointer: &BackendPointer<ItemPrimitive, ItemUnique, VecRange>,
    ) -> Option<&StoreItemCell<ItemPrimitive, ItemUnique, VecRange>>;
    fn item_store_size(&self) -> usize;

    fn push_data(
        &mut self,
        item: StoreItemCell<DataPrimitive, (), VecRange>,
    ) -> BackendPointer<DataPrimitive, (), VecRange>;
    fn get_data(
        &self,
        pointer: &BackendPointer<DataPrimitive, (), VecRange>,
    ) -> Option<&StoreItemCell<DataPrimitive, (), VecRange>>;
    fn data_store_size(&self) -> usize;

    fn push_sig(
        &mut self,
        item: StoreItemCell<SignaturePrimitive, SignatureUnique, VecRange>,
    ) -> BackendPointer<SignaturePrimitive, SignatureUnique, VecRange>;
    fn get_sig(
        &self,
        pointer: &BackendPointer<SignaturePrimitive, SignatureUnique, VecRange>,
    ) -> Option<&StoreItemCell<SignaturePrimitive, SignatureUnique, VecRange>>;
    fn sig_store_size(&self) -> usize;
}
