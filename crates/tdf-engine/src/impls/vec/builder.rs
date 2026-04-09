use crate::backend::{vec_backend::VecTypes, BackendAccess, BackendPointer, VecBackend};
use crate::primitives::item::{ItemPrimitive, ItemTypes, ItemUnique};
use crate::primitives::page::PageTypes;
use crate::segments::{
    header::{HeaderSegment, SegmentOffsets},
    meta::MetaSegment,
    pages::{PageEntry, PageTags, PagesSegment},
};

use crate::impls::document::{BackedDocument, TDFManifest};

pub trait TDFBuilder: Sized {
    type Output;

    fn title(self, title: impl Into<String>) -> Self;
    fn add_page(self, items: Vec<(ItemPrimitive<VecTypes>, ItemUnique)>) -> Self;
    fn build(self) -> Self::Output;
}

/// Constructs a TDF document in memory using [`VecBackend`].
#[derive(Default)]
pub struct DummyTDFBuilder {
    title: Option<String>,
    staged_pages: Vec<Vec<(ItemPrimitive<VecTypes>, ItemUnique)>>,
}

impl DummyTDFBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TDFBuilder for DummyTDFBuilder {
    type Output = BackedDocument<VecBackend>;

    fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    fn add_page(mut self, items: Vec<(ItemPrimitive<VecTypes>, ItemUnique)>) -> Self {
        self.staged_pages.push(items);
        self
    }

    fn build(self) -> BackedDocument<VecBackend> {
        let mut backend = VecBackend::new();
        let mut pages_segment = PagesSegment::new();

        for page_items in self.staged_pages {
            let item_ptrs: Vec<BackendPointer<ItemTypes<VecTypes>, VecTypes>> = page_items
                .into_iter()
                .map(|(prim, uniq)| {
                    <VecBackend as BackendAccess<ItemTypes<VecTypes>, VecBackend>>::push_cell(
                        &mut backend,
                        prim,
                        uniq,
                    )
                })
                .collect();

            let item_group =
                <VecBackend as BackendAccess<ItemTypes<VecTypes>, VecBackend>>::group_together(
                    &mut backend,
                    item_ptrs,
                );

            let page_ptr =
                <VecBackend as BackendAccess<PageTypes<VecTypes>, VecBackend>>::push_cell(
                    &mut backend,
                    item_group,
                    (),
                );

            pages_segment.pages.push(PageEntry {
                tags: PageTags::default(),
                page_ref: page_ptr,
            });
        }

        let header = HeaderSegment::new(0, SegmentOffsets::new(0, 0, 0));
        let meta = MetaSegment {
            document_title: self.title,
            ..Default::default()
        };

        BackedDocument {
            manifest: TDFManifest {
                header,
                meta,
                pages: pages_segment,
            },
            backend,
        }
    }
}
