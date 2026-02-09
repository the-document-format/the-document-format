//! This is where definitions for all core primative data items live.

use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct FontItem {
    tags: FontTags,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FontTags {
    // TODO
}
