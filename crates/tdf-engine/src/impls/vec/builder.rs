use crate::backend::{BackendAccess, BackendPointer};
use crate::impls::document::{BackedDocument, TDFManifest};
use crate::impls::vec::backend::{VecBackend, VecSinglePointer, VecTypes};
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
    staged_data: Vec<DataPrimitive>,
    staged_pages: Vec<Vec<(ItemPrimitive<VecTypes>, ItemUnique)>>,
}

impl DummyTDFBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a data item and returns a pointer to it for use in `ItemPrimitive::Image` etc.
    /// Data items are pushed to the backend before pages, so indices are stable.
    pub fn stage_data(&mut self, data: DataPrimitive) -> DataStorePointer<VecTypes> {
        let index = self.staged_data.len();
        self.staged_data.push(data);
        BackendPointer::Single(VecSinglePointer { index, unique: () })
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
        let mut data_frontend = DataStore::<VecBackend>::default();
        let mut item_frontend = ItemStore::<VecBackend>::default();
        let mut page_frontend = PageStore::<VecBackend>::default();
        let sig_frontend = SignatureStore::<VecBackend>::default();
        let mut pages_segment = PagesSegment::new();

        for data in self.staged_data {
            data_frontend.push(data, (), &mut backend);
        }

        for page_items in self.staged_pages {
            let item_ptrs: Vec<BackendPointer<ItemTypes<VecTypes>, VecTypes>> = page_items
                .into_iter()
                .map(|(prim, uniq)| item_frontend.push(prim, uniq, &mut backend))
                .collect();

            // group_together is a backend-level operation (no Frontend equivalent yet)
            let item_group =
                <VecBackend as BackendAccess<ItemTypes<VecTypes>, VecBackend>>::group_together(
                    &mut backend,
                    item_ptrs,
                );

            let page_ptr = page_frontend.push(
                PageStorePrimitive {
                    tags: PageTags::default(),
                    items: item_group,
                },
                (),
                &mut backend,
            );

            pages_segment.pages.push(page_ptr);
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
            page_frontend,
            item_frontend,
            data_frontend,
            sig_frontend,
        }
    }
}
