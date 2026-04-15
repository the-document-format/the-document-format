//! Builder for BinaryBackend documents.

use crate::backend::BackendAccess;
use crate::impls::document::{BackedDocument, TDFManifest};
use crate::primitives::data::{DataPrimitive, DataStorePointer};
use crate::primitives::item::{ItemPrimitive, ItemTypes, ItemUnique};
use crate::primitives::page::{PageStorePrimitive, PageTags};
use crate::segments::header::{HeaderSegment, SegmentOffsets};
use crate::segments::meta::MetaSegment;
use crate::segments::pages::PagesSegment;
use crate::store::frontend::Frontend;
use crate::store::{DataStore, ItemStore, PageStore, SignatureStore};

use super::backend::{BinaryBackend, BinaryTypes};

// TODO: can we have a single builder crate?

pub struct BinaryTDFBuilder {
    title: Option<String>,
    backend: BinaryBackend,
    data_frontend: DataStore<BinaryBackend>,
    item_frontend: ItemStore<BinaryBackend>,
    page_frontend: PageStore<BinaryBackend>,
    sig_frontend: SignatureStore<BinaryBackend>,
    staged_pages: Vec<Vec<(ItemPrimitive<BinaryTypes>, ItemUnique)>>,
}

impl BinaryTDFBuilder {
    pub fn new() -> Self {
        Self {
            title: None,
            backend: BinaryBackend::new(),
            data_frontend: Default::default(),
            item_frontend: Default::default(),
            page_frontend: Default::default(),
            sig_frontend: Default::default(),
            staged_pages: vec![],
        }
    }

    /// Eagerly pushes data to the backend and returns a real pointer.
    pub fn stage_data(&mut self, data: DataPrimitive) -> DataStorePointer<BinaryTypes> {
        self.data_frontend.push(data, (), &mut self.backend)
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn add_page(mut self, items: Vec<(ItemPrimitive<BinaryTypes>, ItemUnique)>) -> Self {
        self.staged_pages.push(items);
        self
    }

    pub fn build(mut self) -> BackedDocument<BinaryBackend> {
        let mut pages_segment = PagesSegment::new();

        for page_items in self.staged_pages {
            let item_ptrs: Vec<_> = page_items
                .into_iter()
                .map(|(prim, uniq)| self.item_frontend.push(prim, uniq, &mut self.backend))
                .collect();

            // group_together is a backend-level operation
            let item_group = <BinaryBackend as BackendAccess<
                ItemTypes<BinaryTypes>,
                BinaryBackend,
            >>::group_together(&mut self.backend, item_ptrs);

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

impl Default for BinaryTDFBuilder {
    fn default() -> Self {
        Self::new()
    }
}
