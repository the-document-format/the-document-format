use serde::{Deserialize, Serialize};

use crate::backend::BackendTypes;
use crate::primitives::page::PageStorePointer;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(bound(serialize = "B: BackendTypes", deserialize = "B: BackendTypes"))]
pub struct PagesSegment<B: BackendTypes> {
    pub pages: Vec<PageStorePointer<B>>,
}

impl<B: BackendTypes> PagesSegment<B> {
    pub fn new() -> Self {
        Self { pages: vec![] }
    }

    pub fn get_page(&self, page_number: usize) -> Option<&PageStorePointer<B>> {
        self.pages.get(page_number)
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}
