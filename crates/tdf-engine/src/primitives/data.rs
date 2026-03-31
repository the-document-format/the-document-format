use serde::{Deserialize, Serialize};

use crate::store::traits::StoreTypes;

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
pub type DataStorePointer = crate::backend::BackendPointer<
    crate::store::frontend::optimized::OptimizedFrontend<DataTypes, crate::backend::VecBackend>,
    crate::backend::VecBackend,
>;

impl crate::store::traits::PrimitiveType for DataPrimitive {}

pub struct DataTypes;

impl StoreTypes for DataTypes {
    type Primitive = DataPrimitive;
    type Unique = ();
}
