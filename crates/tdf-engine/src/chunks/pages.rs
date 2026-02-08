use serde::{Deserialize, Serialize};

use crate::chunks::store::StoreItemRef;

#[derive(Serialize, Deserialize, Debug)]
pub struct PagesChunk {
    item: StoreItemRef,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    // TODO
}
