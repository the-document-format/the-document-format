//! TDFBuilder trait and DummyTDFBuilder concrete implementation.

use crate::backend::{BackendPointer, VecBackend};
use crate::primitives::item::{ItemPrimitive, ItemUnique};
use crate::reader::{VecReader, PageStore, ItemStore, DataStore, SigStore};
use crate::segments::{header::{HeaderSegment, SegmentOffsets}, meta::MetaSegment, pages::{PageEntry, PageTags, PagesSegment}};
use crate::store::traits::Store;

pub trait TDFBuilder: Sized {
    type Output;
    fn title(self, title: impl Into<String>) -> Self;
    fn add_page(self, items: Vec<(ItemPrimitive, ItemUnique)>) -> Self;
    fn build(self) -> Self::Output;
}

/// Builds a TDF document in memory.
#[derive(Default)]
pub struct DummyTDFBuilder {
    backend: VecBackend,
    page_store: PageStore,
    item_store: ItemStore,
    data_store: DataStore,
    sig_store: SigStore,
    meta: MetaSegment,
    pages: PagesSegment,
    staged_pages: Vec<Vec<(ItemPrimitive, ItemUnique)>>,
}

impl DummyTDFBuilder {
    pub fn new() -> Self { Self::default() }
}

impl TDFBuilder for DummyTDFBuilder {
    type Output = VecReader;

    fn title(mut self, title: impl Into<String>) -> Self {
        self.meta.document_title = Some(title.into());
        self
    }

    fn add_page(mut self, items: Vec<(ItemPrimitive, ItemUnique)>) -> Self {
        self.staged_pages.push(items);
        self
    }

    fn build(mut self) -> VecReader {
        for page_items in self.staged_pages {
            let mut item_ptrs = vec![];
            for (primitive, unique) in page_items {
                let mut ptr = self.item_store.push(primitive, &mut self.backend);
                if let BackendPointer::Pointer { unique: u, .. } = &mut ptr { *u = unique; }
                item_ptrs.push(ptr);
            }
            let item_pointer = self.item_store.group(item_ptrs, &mut self.backend);
            let page_ptr = self.page_store.push(item_pointer, &mut self.backend);
            self.pages.pages.push(PageEntry { page_ref: page_ptr, tags: PageTags::default() });
        }
        let header = HeaderSegment::new(0, SegmentOffsets::new(0, 0, 0));
        VecReader::new(self.backend, self.page_store, self.item_store,
                       self.data_store, self.sig_store, header, self.meta, self.pages)
    }
}
