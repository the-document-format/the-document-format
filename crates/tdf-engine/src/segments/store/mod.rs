//! The content store is where all actual data in a TDF is stored.
//!
//! A store contains a giant list of many store item references. Each store item
//! reference may be the literal content of some data item (like an actual
//! image, with all the actual image data), or a pointer to some other item in
//! the big master list, using an index as a reference.

pub mod data_store;
pub mod page_store;

use serde::{Deserialize, Serialize};

use thiserror::Error;

use crate::segments::store::page_store::PageData;

#[derive(Error, Debug)]
pub enum StoreSegmentError {
    #[error("Index {index} is out of bounds for store item collection of size {size}")]
    IndexOutOfBounds { index: usize, size: usize },
}

/// The top level container storing all of the store segments.
///
/// Internally it is stored as a vector, where at a given index it stores an
/// item, which may have some positional data, followed by either actual data,
/// or a pointer to actual data.
#[derive(Serialize, Deserialize, Debug)]
pub struct StoreSegment<T> {
    items: StoreItemCollection<T>,
}

/// This is a thin wrapper on a collection of many store items.
///
/// Eventually we will want to maintain various optimizations as we update this,
/// so we want to be in control of the interface.
#[derive(Serialize, Deserialize, Debug)]
pub struct StoreItemCollection<T> {
    items: Vec<StoreItemCell<T>>,
}

impl<T> Default for StoreItemCollection<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T> StoreItemCollection<T> {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, index: usize) -> Option<&StoreItemCell<T>> {
        self.items.get(index)
    }

    pub fn set(&mut self, index: usize, item: StoreItemCell<T>) -> Result<(), StoreSegmentError> {
        if index < self.items.len() {
            self.items[index] = item;
            Ok(())
        } else {
            Err(StoreSegmentError::IndexOutOfBounds {
                index,
                size: self.items.len(),
            })
        }
    }
}

/// An entry in the global store.
#[derive(Serialize, Deserialize, Debug)]
pub struct StoreItemCell<T> {
    /// Unique data to the specific store item that cannot be interned, even if
    /// the inner content can be.
    unique: StoreItemKind,
    /// Either the content inside the store item or a pointer to a real store item.
    content: StoreItemRef<T>,
}

/// Either a real store item, or a pointer to a real store item.
#[derive(Serialize, Deserialize, Debug)]
pub enum StoreItemRef<T> {
    Primative(T),
    StoreItems(StoreItemCollection<T>),
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
