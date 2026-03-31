use crate::backend::VecBackend;
use crate::primitives::page::PageStorePointer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PagesSegment {
    pub pages: Vec<PageEntry>,
}

impl PagesSegment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_page(&self, page_number: usize) -> Option<&PageEntry> {
        self.pages.get(page_number)
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PageEntry {
    pub tags: PageTags,
    pub page_ref: PageStorePointer<VecBackend>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PageTags {
    pub physical_page_number: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
