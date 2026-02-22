use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

use crate::segments::store::{
    StoreItemRef,
    page_store::{PageItemPrimative, PageItemUnique},
};

#[derive(Serialize, Deserialize, Debug, Constructor)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct PagesSegment<'a> {
    item: StoreItemRef<'a, PageItemPrimative, PageItemUnique>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Page<'a> {
    tags: PageTags,
    items: Vec<StoreItemRef<'a, PageItemPrimative, PageItemUnique>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PageTags {
    physical_page_number: Option<u32>,
}
