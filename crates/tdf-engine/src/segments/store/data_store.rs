use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

use crate::segments::store::StoreItemCollection;

#[derive(Debug, Serialize, Deserialize, Constructor, Default)]
pub struct DataStore {
    pages: StoreItemCollection<DataItemPrimative>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DataItemPrimative {
    Font(FontItem),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FontItem {
    tags: FontTags,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FontTags {
    // TODO
}
