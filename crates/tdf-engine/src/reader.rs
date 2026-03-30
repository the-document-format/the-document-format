//! The TDFReader: highest-level interface for reading a TDF document.

use crate::backend::{Backend, VecBackend};
use crate::primitives::data::{DataPrimitive, DataStorePointer};
use crate::primitives::item::{ItemPrimitive, ItemUnique};
use crate::primitives::page::ItemPointer;
use crate::primitives::signature::{SignaturePrimitive, SignatureUnique};
use crate::segments::{header::HeaderSegment, meta::MetaSegment, pages::PagesSegment};
use crate::store::frontend::append_only::AppendOnlyStore;
use crate::store::frontend::optimized::OptimizedStore;

pub trait TDFReader<B: Backend> {
    fn header(&self) -> &HeaderSegment;
    fn meta(&self) -> &MetaSegment;
    fn pages(&self) -> &PagesSegment;

    fn iter_page_items(
        &self,
        page_number: usize,
    ) -> Box<dyn Iterator<Item = (ItemPrimitive, ItemUnique)>>;

    fn deref_handle(&self, handle: &DataStorePointer) -> Option<DataPrimitive>;
}

pub type PageStore = AppendOnlyStore<ItemPointer, (), VecBackend>;
pub type ItemStore = OptimizedStore<ItemPrimitive, ItemUnique, VecBackend>;
pub type DataStore = OptimizedStore<DataPrimitive, (), VecBackend>;
pub type SigStore = AppendOnlyStore<SignaturePrimitive, SignatureUnique, VecBackend>;

/// Concrete reader backed by a [`VecBackend`].
pub struct VecReader {
    backend: VecBackend,
    page_store: PageStore,
    item_store: ItemStore,
    data_store: DataStore,
    sig_store: SigStore,
    header: HeaderSegment,
    meta: MetaSegment,
    pages: PagesSegment,
}

impl VecReader {
    pub fn new(
        backend: VecBackend,
        page_store: PageStore,
        item_store: ItemStore,
        data_store: DataStore,
        sig_store: SigStore,
        header: HeaderSegment,
        meta: MetaSegment,
        pages: PagesSegment,
    ) -> Self {
        Self { backend, page_store, item_store, data_store, sig_store, header, meta, pages }
    }
}

impl TDFReader<VecBackend> for VecReader {
    fn header(&self) -> &HeaderSegment { &self.header }
    fn meta(&self) -> &MetaSegment { &self.meta }
    fn pages(&self) -> &PagesSegment { &self.pages }

    fn iter_page_items(
        &self,
        page_number: usize,
    ) -> Box<dyn Iterator<Item = (ItemPrimitive, ItemUnique)>> {
        todo!()
    }

    fn deref_handle(&self, handle: &DataStorePointer) -> Option<DataPrimitive> {
        todo!()
    }
}
