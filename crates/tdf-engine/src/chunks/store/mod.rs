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
    page_data::PageData,
    primatives::{FontItem, ImageItem, TextItem, VectorItem},
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreChunkError {
    #[error("Index {index} is out of bounds for store item collection of size {size}")]
    IndexOutOfBounds { index: usize, size: usize },
}

/// The top level container storing all of the store chunks.
///
/// Internally it is stored as a vector, where at a given index it stores an
/// item, which may have some positional data, followed by either actual data,
/// or a pointer to actual data.
#[derive(Serialize, Deserialize, Debug)]
pub struct StoreChunk {
    items: StoreItemCollection,
}

/// This is a thin wrapper on a collection of many store items.
///
/// Eventually we will want to maintain various optimizations as we update this,
/// so we want to be in control of the interface.
#[derive(Serialize, Deserialize, Debug)]
pub struct StoreItemCollection {
    items: Vec<StoreItemCell>,
}

impl StoreItemCollection {
    pub fn iter(&self) -> impl Iterator<Item = &StoreItemCell> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, index: usize) -> Option<&StoreItemCell> {
        self.items.get(index)
    }

    pub fn set(
        &mut self,
        index: usize,
        item: StoreItemCell,
    ) -> Result<(), StoreChunkError> {
        if index < self.items.len() {
            self.items[index] = item;
            Ok(())
        } else {
            Err(StoreChunkError::IndexOutOfBounds {
                index,
                size: self.items.len(),
            })
        }
    }
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
    StoreItems(StoreItemCollection),
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
