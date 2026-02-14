use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

use crate::segments::store::StoreItemCollection;

#[derive(Debug, Serialize, Deserialize, Constructor, Default)]
pub struct PagesStore {
    pages: StoreItemCollection<PageItemPrimative>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PageItemPrimative {
    Image(ImageItem),
    Vector(VectorItem),
    Text(TextItem),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageItem {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VectorItem {
    tags: VectorTags,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VectorTags {
    // TODO
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextItem {
    tags: TextTags,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextTags {
    // TODO
}

#[derive(Serialize, Deserialize, Debug, Constructor)]
pub struct PageData {
    position: [f64; 3],
    transformation: Transform,
}

/// The position of an item on a page.
#[derive(Serialize, Deserialize, Constructor)]
struct Position(f64, f64);

impl Position {
    fn x(&self) -> f64 {
        self.0
    }

    fn y(&self) -> f64 {
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
