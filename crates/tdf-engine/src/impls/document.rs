use std::io::{Read, Write};

use serde::de::DeserializeOwned;

use crate::backend::{Backend, BackendAccess, BackendTypes, StoreItemCell};
use crate::primitives::data::{DataPrimitive, DataStorePointer, DataTypes};
use crate::primitives::item::{ItemPrimitive, ItemUnique};
use crate::segments::{header::HeaderSegment, meta::MetaSegment, pages::PagesSegment};

#[derive(Debug)]
pub struct TDFManifest<B: BackendTypes> {
    pub header: HeaderSegment,
    pub meta: MetaSegment,
    pub pages: PagesSegment<B>,
}

pub struct BackedDocument<B: Backend> {
    pub manifest: TDFManifest<B::Types>,
    pub backend: B,
}

pub trait DocumentWrite {
    fn to_writer<W: Write>(&self, writer: &mut W) -> std::io::Result<()>;
}

pub trait ManifestRead: Sized {
    type BTypes: BackendTypes;

    fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self>;

    fn load_backend<B, R>(self, reader: R) -> std::io::Result<BackedDocument<B>>
    where
        B: Backend<Types = Self::BTypes> + DeserializeOwned,
        R: Read;
}

pub trait TdfDocument {
    type B: Backend;

    fn backend(&self) -> &Self::B;
    fn manifest(&self) -> &TDFManifest<<Self::B as Backend>::Types>;
    fn iter_page_items(
        &self,
        page_number: usize,
    ) -> Box<dyn Iterator<Item = (ItemPrimitive<<Self::B as Backend>::Types>, ItemUnique)> + '_>;
}

pub trait TdfDocumentExt: TdfDocument
where
    Self::B: BackendAccess<DataTypes, Self::B>,
{
    fn fetch_data_item(
        &self,
        ptr: &DataStorePointer<<Self::B as Backend>::Types>,
    ) -> Option<DataPrimitive> {
        let cells = <Self::B as BackendAccess<DataTypes, Self::B>>::get_cells(self.backend(), ptr)?;

        match cells.into_iter().next()? {
            StoreItemCell::StorePrimitive(p) => Some(p.clone()),
            _ => None,
        }
    }
}

impl<T: TdfDocument> TdfDocumentExt for T where T::B: BackendAccess<DataTypes, T::B> {}
