use crate::backend::VecRange;
use serde::{Deserialize, Serialize};

/// Large blobs loaded lazily via `TDFReader::deref_handle`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataPrimitive {
    FontData(FontData),
    ImageData(ImageData),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontData {
    pub bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageData {
    pub bytes: Vec<u8>,
}

/// A lazy cross-store reference from an item primitive into the data store.
pub type DataStorePointer = crate::backend::BackendPointer<DataPrimitive, (), VecRange>;

impl crate::store::traits::PrimitiveType for DataPrimitive {}
