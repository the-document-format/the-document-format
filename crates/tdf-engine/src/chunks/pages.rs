use serde::{Deserialize, Serialize};

use crate::chunks::store::StoreItemRef;

#[derive(Serialize, Deserialize, Debug)]
pub struct PagesChunk {
    item: StoreItemRef,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    tags: PageTags,
    items: Vec<StoreItemRef>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PageTags {
    physical_page_number: Option<u32>,
}
