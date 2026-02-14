use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

use crate::segments::store::{page_store::PageItemPrimative, StoreItemRef};

#[derive(Serialize, Deserialize, Debug, Constructor)]
pub struct PagesSegment {
    item: StoreItemRef<PageItemPrimative>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    tags: PageTags,
    items: Vec<StoreItemRef<PageItemPrimative>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PageTags {
    physical_page_number: Option<u32>,
}
