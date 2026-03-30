//! TDFBuilder trait and DummyTDFBuilder concrete implementation.

use crate::backend::VecBackend;
use crate::primitives::item::{ItemPrimitive, ItemUnique};
use crate::reader::{VecReader, PageStore, ItemStore, DataStore, SigStore};
use crate::segments::{meta::MetaSegment, pages::PagesSegment};

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

    fn build(self) -> VecReader {
        todo!()
    }
}
