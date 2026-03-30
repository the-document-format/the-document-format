//! The backend layer: physical storage abstraction for all four TDF stores.
//!
//! [`Backend`] is the storage interface implemented by concrete backends like [`VecBackend`].
//! [`BackendView`] is the typed, offset-aware accessor that stores hold to communicate with
//! the backend without knowing about other stores' regions.

pub mod vec_backend;
pub use vec_backend::VecBackend;

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

/// The core reference type. Every item in every store is addressed by `BackendPointer<T, U>`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BackendPointer<T, U> {
    /// References a single item.
    Pointer {
        index: usize,
        unique: U,
        #[serde(skip)]
        _phantom: PhantomData<T>,
    },
    /// References a contiguous range of items.
    PointerRange {
        start: usize,
        len: usize,
        unique: U,
        #[serde(skip)]
        _phantom: PhantomData<T>,
    },
}

impl<T: PartialEq + Eq, U: Ord> PartialOrd for BackendPointer<T, U> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: PartialEq + Eq, U: Ord> Ord for BackendPointer<T, U> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        todo!()
    }
}

impl<T, U: Default> BackendPointer<T, U> {
    pub fn new(index: usize) -> Self {
        BackendPointer::Pointer { index, unique: U::default(), _phantom: PhantomData }
    }

    pub fn range(start: usize, len: usize) -> Self {
        BackendPointer::PointerRange { start, len, unique: U::default(), _phantom: PhantomData }
    }
}

/// What you get back when reading from any store.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum StoreItemCell<T, U> {
    BackendPointer(BackendPointer<T, U>),
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
        BackendView { offset, _phantom: PhantomData }
    }
}

/// Physical storage for all four TDF stores.
pub trait Backend {
    fn push_page(&mut self, item: StoreItemCell<ItemPointer, ()>)
        -> BackendPointer<ItemPointer, ()>;
    fn get_page(&self, pointer: &BackendPointer<ItemPointer, ()>)
        -> Option<&StoreItemCell<ItemPointer, ()>>;
    fn page_store_size(&self) -> usize;

    fn push_item(&mut self, item: StoreItemCell<ItemPrimitive, ItemUnique>)
        -> BackendPointer<ItemPrimitive, ItemUnique>;
    fn get_item(&self, pointer: &BackendPointer<ItemPrimitive, ItemUnique>)
        -> Option<&StoreItemCell<ItemPrimitive, ItemUnique>>;
    fn item_store_size(&self) -> usize;

    fn push_data(&mut self, item: StoreItemCell<DataPrimitive, ()>)
        -> BackendPointer<DataPrimitive, ()>;
    fn get_data(&self, pointer: &BackendPointer<DataPrimitive, ()>)
        -> Option<&StoreItemCell<DataPrimitive, ()>>;
    fn data_store_size(&self) -> usize;

    fn push_sig(&mut self, item: StoreItemCell<SignaturePrimitive, SignatureUnique>)
        -> BackendPointer<SignaturePrimitive, SignatureUnique>;
    fn get_sig(&self, pointer: &BackendPointer<SignaturePrimitive, SignatureUnique>)
        -> Option<&StoreItemCell<SignaturePrimitive, SignatureUnique>>;
    fn sig_store_size(&self) -> usize;
}
