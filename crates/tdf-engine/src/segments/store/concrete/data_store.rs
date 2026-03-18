use crate::segments::store::store::{PrimativeType, StoreItemCollection, UniqueType};
use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Constructor, Default)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct DataStore<'a> {
    pages: StoreItemCollection<'a, DataItemPrimative, DataItemUnique>,
}

impl<'a> UniqueType<'a> for DataItemUnique {}

/// Data items have no non-internable properties right now.
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
struct DataItemUnique;

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub enum DataItemPrimative {
    Font(FontItem),
}

impl<'a> PrimativeType<'a> for DataItemPrimative {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct FontItem {
    tags: FontTags,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct FontTags {
    // TODO
}
