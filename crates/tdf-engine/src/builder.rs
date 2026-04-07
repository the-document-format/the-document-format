//! TDFBuilder trait and DummyTDFBuilder concrete implementation.

use crate::backend::vec_backend::VecTypes;
use crate::backend::{BackendAccess, BackendPointer, VecBackend};
use crate::primitives::item::{ItemPrimitive, ItemTypes, ItemUnique};
use crate::reader::VecReader;
use crate::segments::{
    header::{HeaderSegment, SegmentOffsets},
    meta::MetaSegment,
    pages::{PageEntry, PageTags, PagesSegment},
};
use crate::store::traits::Store;
use crate::store::{DataStore, ItemStore, PageStore, SignatureStore};

pub trait TDFBuilder: Sized {
    type Output;
    fn title(self, title: impl Into<String>) -> Self;
    fn add_page(self, items: Vec<(ItemPrimitive<VecTypes>, ItemUnique)>) -> Self;
    fn build(self) -> Self::Output;
}

/// Builds a TDF document in memory.
#[derive(Default)]
pub struct DummyTDFBuilder {
    backend: VecBackend,
    page_store: PageStore<VecBackend>,
    item_store: ItemStore<VecBackend>,
    data_store: DataStore<VecBackend>,
    sig_store: SignatureStore<VecBackend>,
    meta: MetaSegment,
    pages: PagesSegment<VecTypes>,
    staged_pages: Vec<Vec<(ItemPrimitive<VecTypes>, ItemUnique)>>,
}

impl DummyTDFBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TDFBuilder for DummyTDFBuilder {
    type Output = VecReader;

    fn title(mut self, title: impl Into<String>) -> Self {
        self.meta.document_title = Some(title.into());
        self
    }

    fn add_page(mut self, items: Vec<(ItemPrimitive<VecTypes>, ItemUnique)>) -> Self {
        self.staged_pages.push(items);
        self
    }

    fn build(mut self) -> VecReader {
        for page_items in self.staged_pages {
            let mut item_ptrs: Vec<BackendPointer<ItemTypes<VecTypes>, VecTypes>> = vec![];

            for (primitive, unique) in page_items {
                let ptr = self.item_store.push(primitive, unique, &mut self.backend);
                item_ptrs.push(ptr);
            }

            let item_pointer =
                <VecBackend as BackendAccess<ItemTypes<VecTypes>, VecBackend>>::group_together(
                    &mut self.backend,
                    item_ptrs,
                );

            let page_ptr = self.page_store.push(item_pointer, (), &mut self.backend);

            self.pages.pages.push(PageEntry {
                page_ref: page_ptr,
                tags: PageTags::default(),
            });
        }

        let header = HeaderSegment::new(0, SegmentOffsets::new(0, 0, 0));

        VecReader::new(
            self.backend,
            self.page_store,
            self.item_store,
            self.data_store,
            self.sig_store,
            header,
            self.meta,
            self.pages,
        )
    }
}
