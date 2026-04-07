use serde::{Deserialize, Serialize};

use crate::{
    backend::{BackendPointer, BackendTypes},
    store::traits::StoreTypes,
};

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

pub type DataStorePointer<B: BackendTypes> = BackendPointer<DataTypes, B>;

impl crate::store::traits::PrimitiveType for DataPrimitive {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataTypes;

impl StoreTypes for DataTypes {
    type Primitive = DataPrimitive;
    type Unique = ();
}
