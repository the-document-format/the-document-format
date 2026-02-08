//! Page data is a container that stores all the data necessary to place an item onto a page.
//!
//! This data lives separately from the actual items (i.e. an ImageItem struct
//! does not store its own transformations) so that we can intern the image
//! irrespective of where it lands on different pages.

use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Constructor)]
pub struct PageData {
    position: [f64; 3],
    transformation: Transform,
}

/// The position of an item on a page.
#[derive(Serialize, Deserialize, Constructor)]
pub struct Position(f64, f64);

impl Position {
    pub fn x(&self) -> f64 {
        self.0
    }

    pub fn y(&self) -> f64 {
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
    pub fn translation(&self) -> [f64; 3] {
        self.translation
    }

    pub fn rotation(&self) -> [f64; 3] {
        self.rotation
    }

    pub fn scale(&self) -> [f64; 3] {
        self.scale
    }
}
