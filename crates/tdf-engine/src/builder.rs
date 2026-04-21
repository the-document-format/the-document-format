use crate::backend::{Backend, BackendAccess, BackendPointer};
use crate::impls::document::{BackedDocument, TDFManifest};
use crate::primitives::data::{DataPrimitive, DataStorePointer};
use crate::primitives::item::{ItemPrimitive, ItemTypes, ItemUnique};
use crate::primitives::page::{PageStorePrimitive, PageTags};
use crate::segments::{
    header::{HeaderSegment, SegmentOffsets},
    meta::MetaSegment,
    pages::PagesSegment,
};
use crate::store::frontend::Frontend;
use crate::store::{DataStore, ItemStore, PageStore, SignatureStore};

/// Constructs a TDF document using any [`Backend`].
pub struct TDFBuilder<B: Backend> {
    title: Option<String>,
    backend: B,
    data_frontend: DataStore<B>,
    item_frontend: ItemStore<B>,
    page_frontend: PageStore<B>,
    sig_frontend: SignatureStore<B>,
    staged_pages: Vec<Vec<(ItemPrimitive<B::Types>, ItemUnique)>>,
}

impl<B: Backend + Default> TDFBuilder<B> {
    pub fn new() -> Self {
        Self {
            title: None,
            backend: B::default(),
            data_frontend: Default::default(),
            item_frontend: Default::default(),
            page_frontend: Default::default(),
            sig_frontend: Default::default(),
            staged_pages: vec![],
        }
    }

    /// Eagerly pushes a data item to the backend and returns a pointer to it.
    pub fn stage_data(&mut self, data: DataPrimitive) -> DataStorePointer<B::Types> {
        self.data_frontend.push(data, (), &mut self.backend)
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn add_page(mut self, items: Vec<(ItemPrimitive<B::Types>, ItemUnique)>) -> Self {
        self.staged_pages.push(items);
        self
    }

    pub fn build(mut self) -> BackedDocument<B> {
        let mut pages_segment = PagesSegment::new();

        for page_items in self.staged_pages {
            let item_ptrs: Vec<BackendPointer<ItemTypes<B::Types>, B::Types>> = page_items
                .into_iter()
                .map(|(prim, uniq)| self.item_frontend.push(prim, uniq, &mut self.backend))
                .collect();

            let item_group = <B as BackendAccess<ItemTypes<B::Types>, B>>::group_together(
                &mut self.backend,
                item_ptrs,
            );

            let page_ptr = self.page_frontend.push(
                PageStorePrimitive {
                    tags: PageTags::default(),
                    items: item_group,
                },
                (),
                &mut self.backend,
            );

            pages_segment.pages.push(page_ptr);
        }

        BackedDocument {
            manifest: TDFManifest {
                header: HeaderSegment::new(0, SegmentOffsets::new(0, 0, 0)),
                meta: MetaSegment {
                    document_title: self.title,
                    ..Default::default()
                },
                pages: pages_segment,
            },
            backend: self.backend,
            page_frontend: self.page_frontend,
            item_frontend: self.item_frontend,
            data_frontend: self.data_frontend,
            sig_frontend: self.sig_frontend,
        }
    }
}

impl<B: Backend + Default> Default for TDFBuilder<B> {
    fn default() -> Self {
        Self::new()
    }
}
