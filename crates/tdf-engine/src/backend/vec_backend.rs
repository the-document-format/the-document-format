//! In-memory backend using four Vecs — one per store.

use crate::backend::{Backend, BackendPointer, StoreItemCell};
use crate::primitives::{
    data::DataPrimitive,
    item::{ItemPrimitive, ItemUnique},
    page::ItemPointer,
    signature::{SignaturePrimitive, SignatureUnique},
};

/// The simplest possible backend: four `Vec`s, one per store.
#[derive(Debug, Default)]
pub struct VecBackend {
    page_store: Vec<StoreItemCell<ItemPointer, ()>>,
    item_store: Vec<StoreItemCell<ItemPrimitive, ItemUnique>>,
    data_store: Vec<StoreItemCell<DataPrimitive, ()>>,
    sig_store: Vec<StoreItemCell<SignaturePrimitive, SignatureUnique>>,
}

impl VecBackend {
    pub fn new() -> Self { Self::default() }
}

impl Backend for VecBackend {
    fn push_page(&mut self, item: StoreItemCell<ItemPointer, ()>)
        -> BackendPointer<ItemPointer, ()> { todo!() }
    fn get_page(&self, pointer: &BackendPointer<ItemPointer, ()>)
        -> Option<&StoreItemCell<ItemPointer, ()>> { todo!() }
    fn page_store_size(&self) -> usize { self.page_store.len() }

    fn push_item(&mut self, item: StoreItemCell<ItemPrimitive, ItemUnique>)
        -> BackendPointer<ItemPrimitive, ItemUnique> { todo!() }
    fn get_item(&self, pointer: &BackendPointer<ItemPrimitive, ItemUnique>)
        -> Option<&StoreItemCell<ItemPrimitive, ItemUnique>> { todo!() }
    fn item_store_size(&self) -> usize { self.item_store.len() }

    fn push_data(&mut self, item: StoreItemCell<DataPrimitive, ()>)
        -> BackendPointer<DataPrimitive, ()> { todo!() }
    fn get_data(&self, pointer: &BackendPointer<DataPrimitive, ()>)
        -> Option<&StoreItemCell<DataPrimitive, ()>> { todo!() }
    fn data_store_size(&self) -> usize { self.data_store.len() }

    fn push_sig(&mut self, item: StoreItemCell<SignaturePrimitive, SignatureUnique>)
        -> BackendPointer<SignaturePrimitive, SignatureUnique> { todo!() }
    fn get_sig(&self, pointer: &BackendPointer<SignaturePrimitive, SignatureUnique>)
        -> Option<&StoreItemCell<SignaturePrimitive, SignatureUnique>> { todo!() }
    fn sig_store_size(&self) -> usize { self.sig_store.len() }
}
