//! This is where all of the core primative itms

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageItem {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VectorItem {
    // TODO
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextItem {
    // TODO
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FontItem {
    // TODO
}
