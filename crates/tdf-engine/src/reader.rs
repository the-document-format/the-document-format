//! The TDFReader: highest-level interface for reading a TDF document.

use crate::backend::vec_backend::{DataStore, ItemStore, PageStore, SignatureStore, VecTypes};
use crate::backend::{BackendTypes, VecBackend};
use crate::primitives::data::{DataPrimitive, DataStorePointer};
use crate::primitives::item::{ItemPrimitive, ItemUnique};
use crate::segments::{header::HeaderSegment, meta::MetaSegment, pages::PagesSegment};

pub trait TDFReader<B: BackendTypes> {
    fn header(&self) -> &HeaderSegment;
    fn meta(&self) -> &MetaSegment;
    fn pages(&self) -> &PagesSegment<B>;

    fn iter_page_items(
        &self,
        page_number: usize,
    ) -> Box<dyn Iterator<Item = (ItemPrimitive<B>, ItemUnique)>>;

    fn deref_handle(&self, handle: &DataStorePointer<B>) -> Option<DataPrimitive>;
}

/// Concrete reader backed by a [`VecBackend`].
pub struct VecReader {
    backend: VecBackend,
    page_store: PageStore,
    item_store: ItemStore,
    data_store: DataStore,
    sig_store: SignatureStore,
    header: HeaderSegment,
    meta: MetaSegment,
    pages: PagesSegment<VecTypes>,
}

impl VecReader {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        backend: VecBackend,
        page_store: PageStore,
        item_store: ItemStore,
        data_store: DataStore,
        sig_store: SignatureStore,
        header: HeaderSegment,
        meta: MetaSegment,
        pages: PagesSegment<VecTypes>,
    ) -> Self {
        Self {
            backend,
            page_store,
            item_store,
            data_store,
            sig_store,
            header,
            meta,
            pages,
        }
    }
}

impl TDFReader for VecReader {
    fn header(&self) -> &HeaderSegment {
        &self.header
    }
    fn meta(&self) -> &MetaSegment {
        &self.meta
    }
    fn pages(&self) -> &PagesSegment {
        &self.pages
    }

    fn iter_page_items(
        &self,
        page_number: usize,
    ) -> Box<dyn Iterator<Item = (ItemPrimitive, ItemUnique)>> {
        use crate::backend::StoreItemCell;
        use crate::store::traits::{Store, StoreExt};
        let page_entry = match self.pages.get_page(page_number) {
            Some(e) => e,
            None => return Box::new(std::iter::empty()),
        };
        let cell = match self.page_store.get(&page_entry.page_ref, &self.backend) {
            Some(c) => c,
            None => return Box::new(std::iter::empty()),
        };
        let item_pointer = match cell {
            StoreItemCell::StorePrimitive(p) => p.clone(),
            _ => return Box::new(std::iter::empty()),
        };
        Box::new(
            self.item_store
                .iter_rec(&item_pointer, &self.backend)
                .into_iter(),
        )
    }

    fn deref_handle(&self, handle: &DataStorePointer) -> Option<DataPrimitive> {
        todo!()
    }
}
