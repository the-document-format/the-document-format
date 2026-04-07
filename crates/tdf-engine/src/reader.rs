//! The TDFReader: highest-level interface for reading a TDF document.

use crate::backend::vec_backend::VecTypes;
use crate::backend::{BackendAccess, BackendPointer, StoreItemCell};
use crate::backend::{BackendTypes, VecBackend};
use crate::primitives::data::{DataPrimitive, DataStorePointer};
use crate::primitives::item::ItemTypes;
use crate::primitives::item::{ItemPrimitive, ItemUnique};
use crate::segments::{header::HeaderSegment, meta::MetaSegment, pages::PagesSegment};
use crate::store::traits::Store;
use crate::store::{DataStore, ItemStore, PageStore, SignatureStore};

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
    page_store: PageStore<VecBackend>,
    item_store: ItemStore<VecBackend>,
    data_store: DataStore<VecBackend>,
    sig_store: SignatureStore<VecBackend>,
    header: HeaderSegment,
    meta: MetaSegment,
    pages: PagesSegment<VecTypes>,
}

impl VecReader {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        backend: VecBackend,
        page_store: PageStore<VecBackend>,
        item_store: ItemStore<VecBackend>,
        data_store: DataStore<VecBackend>,
        sig_store: SignatureStore<VecBackend>,
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

impl TDFReader<VecTypes> for VecReader {
    fn header(&self) -> &HeaderSegment {
        &self.header
    }
    fn meta(&self) -> &MetaSegment {
        &self.meta
    }
    fn pages(&self) -> &PagesSegment<VecTypes> {
        &self.pages
    }

    fn iter_page_items(
        &self,
        page_number: usize,
    ) -> Box<dyn Iterator<Item = (ItemPrimitive<VecTypes>, ItemUnique)>> {
        let page_entry = match self.pages.get_page(page_number) {
            Some(e) => e,
            None => return Box::new(std::iter::empty()),
        };

        let cell = match self.page_store.get(&page_entry.page_ref, &self.backend) {
            Some(c) => c,
            None => return Box::new(std::iter::empty()),
        };

        // Page primitive is a pointer (group or single) into the item store
        let item_ptr = match cell {
            StoreItemCell::StorePrimitive(p) => p.clone(),
            _ => return Box::new(std::iter::empty()),
        };

        // Expand to individual single pointers, each carrying its unique
        let item_ptrs: Vec<BackendPointer<ItemTypes<VecTypes>, VecTypes>> = match &item_ptr {
            BackendPointer::Group(g) => <VecBackend as BackendAccess<
                ItemTypes<VecTypes>,
                VecBackend,
            >>::expand_group(&self.backend, g),
            BackendPointer::Single(_) => vec![item_ptr],
        };

        let items: Vec<(ItemPrimitive<VecTypes>, ItemUnique)> = item_ptrs
            .into_iter()
            .filter_map(|ptr| {
                let unique = match &ptr {
                    BackendPointer::Single(s) => s.unique.clone(),
                    BackendPointer::Group(_) => todo!("nested group support not yet implemented"),
                };
                let cell = self.backend.get_cells(&ptr)?.into_iter().next()?;
                match cell {
                    StoreItemCell::StorePrimitive(p) => Some((p.clone(), unique)),
                    _ => None,
                }
            })
            .collect();

        Box::new(items.into_iter())
    }

    fn deref_handle(&self, handle: &DataStorePointer<VecTypes>) -> Option<DataPrimitive> {
        todo!()
    }
}
