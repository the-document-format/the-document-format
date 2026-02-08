//! The content store is where all actual data in a TDF is stored.
//!
//! A store contains a giant list of many store item references. Each store item
//! reference may be the literal content of some data item (like an actual
//! image, with all the actual image data), or a pointer to some other item in
//! the big master list, using an index as a reference.

mod page_data;
mod primatives;

use serde::{Deserialize, Serialize};

use crate::chunks::store::{
    page_data::PageData, primatives::{FontItem, ImageItem, TextItem, VectorItem}
};

/// The top level container storing all of the store chunks.
///
/// Internally it is stored as a vector, where at a given index it stores an
/// item, which may have some positional data, followed by either actual data,
/// or a pointer to actual data.
#[derive(Serialize, Deserialize, Debug)]
pub struct StoreChunk {
    items: Vec<StoreItemCell>,
}

/// An entry in the global store.
#[derive(Serialize, Deserialize, Debug)]
pub struct StoreItemCell {
    kind: StoreItemKind,
    content: StoreItemRef,
}

/// Either a real store item, or a pointer to a real store item.
#[derive(Serialize, Deserialize, Debug)]
pub enum StoreItemRef {
    Primative(StoreItemPrimative),
    StoreItems(Vec<StoreItemCell>),
}

/// The different categories of store items.
///
/// Some store items have positions on pages. Some are just loose data items. Some pieces of data, like position, we don't want to store
#[derive(Serialize, Deserialize, Debug)]
pub enum StoreItemKind {
    /// An item that actually appears on some page somewhere.
    PageItem(PageData),
    /// An item that doesn't directly appear on a page but is used by items that do. Like a font.
    DataItem,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StoreItemPrimative {
    Image(ImageItem),
    Vector(VectorItem),
    Text(TextItem),
    Font(FontItem),
}
