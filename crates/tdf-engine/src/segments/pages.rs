use crate::backend::BackendTypes;
use crate::primitives::page::PageStorePointer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(bound(serialize = "B: BackendTypes", deserialize = "B: BackendTypes"))]
pub struct PagesSegment<B: BackendTypes> {
    pub pages: Vec<PageEntry<B>>,
}

impl<B: BackendTypes> PagesSegment<B> {
    pub fn new() -> Self {
        Self { pages: vec![] }
    }

    pub fn get_page(&self, page_number: usize) -> Option<&PageEntry<B>> {
        self.pages.get(page_number)
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(serialize = "B: BackendTypes", deserialize = "B: BackendTypes"))]
pub struct PageEntry<B: BackendTypes> {
    pub tags: PageTags,
    pub page_ref: PageStorePointer<B>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PageTags {
    pub physical_page_number: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
