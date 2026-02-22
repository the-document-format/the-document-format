use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

use crate::segments::store::{PrimativeType, StoreItemCollection, UniqueType};

#[derive(Debug, Serialize, Deserialize, Constructor, Default)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct PagesStore<'a> {
    pages: StoreItemCollection<'a, PageItemPrimative, PageItemUnique>,
}

#[derive(Debug, Serialize, Deserialize, Constructor, Hash, PartialEq, Eq)]
pub struct PageItemUnique {
    pub position: Position,
}

impl<'a> UniqueType<'a> for PageItemUnique {}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub enum PageItemPrimative {
    Image(ImageItem),
    Vector(VectorItem),
    Text(TextItem),
}

impl<'a> PrimativeType<'a> for PageItemPrimative {}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct ImageItem {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct VectorItem {
    tags: VectorTags,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct VectorTags {
    haah: String,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct TextItem {
    tags: TextTags,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct TextTags {
    haah: String,
}

#[derive(Serialize, Deserialize, Debug, Constructor)]
pub struct PageData {
    position: [f64; 3],
    transformation: Transform,
}

/// The position of an item on a page.
#[derive(Serialize, Deserialize, Constructor, Debug, Hash, PartialEq, Eq)]
pub struct Position(u64, u64);

impl Position {
    fn x(&self) -> u64 {
        self.0
    }

    fn y(&self) -> u64 {
        self.1
    }
}

/// The different transformations that can be applied to a page item.
#[derive(Serialize, Deserialize, Debug, Constructor)]
pub struct Transform {
    translation: [f64; 3],
    rotation: [f64; 3],
    scale: [f64; 3],
}

impl Transform {
    fn translation(&self) -> [f64; 3] {
        self.translation
    }

    fn rotation(&self) -> [f64; 3] {
        self.rotation
    }

    fn scale(&self) -> [f64; 3] {
        self.scale
    }
}
