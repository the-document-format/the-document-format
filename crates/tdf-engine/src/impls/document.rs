use std::io::{Read, Write};

use serde::de::DeserializeOwned;

use crate::backend::{Backend, BackendTypes, CacheHints, StoreItemCell};
use crate::primitives::data::{DataPrimitive, DataStorePointer};
use crate::primitives::item::{ItemPrimitive, ItemUnique};
use crate::primitives::page::PageStorePrimitive;
use crate::segments::{header::HeaderSegment, meta::MetaSegment, pages::PagesSegment};
use crate::store::frontend::{Frontend, FrontendExt};
use crate::store::{DataStore, ItemStore, PageStore, SignatureStore};

#[derive(Debug)]
pub struct TDFManifest<B: BackendTypes> {
    pub header: HeaderSegment,
    pub meta: MetaSegment,
    pub pages: PagesSegment<B>,
}

pub struct BackedDocument<B: Backend> {
    pub manifest: TDFManifest<B::Types>,
    pub backend: B,
    pub page_frontend: PageStore<B>,
    pub item_frontend: ItemStore<B>,
    pub data_frontend: DataStore<B>,
    pub sig_frontend: SignatureStore<B>,
}

/// Bundle of references to all four frontends + mutable backend access.
/// Enables direct frontend use without borrow conflicts.
pub struct StoreAccess<'a, B: Backend> {
    pub backend: &'a mut B,
    pub pages_store: &'a PageStore<B>,
    pub item_store: &'a ItemStore<B>,
    pub data_store: &'a DataStore<B>,
    pub signature_store: &'a SignatureStore<B>,
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

    fn manifest(&self) -> &TDFManifest<<Self::B as Backend>::Types>;
    fn stores(&mut self) -> StoreAccess<'_, Self::B>;
}

pub trait TdfDocumentExt: TdfDocument {
    fn fetch_data_item(
        &mut self,
        ptr: &DataStorePointer<<Self::B as Backend>::Types>,
    ) -> Option<DataPrimitive> {
        let stores = self.stores();
        let cells = stores
            .data_store
            .get(ptr, stores.backend, CacheHints::Cache)
            .ok()?;
        let cell = cells.into_iter().next()?;
        match cell.into_owned() {
            StoreItemCell::StorePrimitive(p) => Some(p),
            _ => None,
        }
    }

    fn iter_page_items(
        &mut self,
        page_number: usize,
        cache_hints: CacheHints,
    ) -> Vec<(ItemPrimitive<<Self::B as Backend>::Types>, ItemUnique)> {
        let page_ptr = match self.manifest().pages.get_page(page_number) {
            Some(p) => p.clone(),
            None => return vec![],
        };

        let stores = self.stores();
        let cells = match stores
            .pages_store
            .get(&page_ptr, stores.backend, cache_hints)
        {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let page_prim: PageStorePrimitive<_> = match cells.into_iter().next() {
            Some(cow) => match cow.into_owned() {
                StoreItemCell::StorePrimitive(p) => p,
                _ => return vec![],
            },
            _ => return vec![],
        };

        stores
            .item_store
            .iter_rec(&page_prim.items, stores.backend, cache_hints)
    }
}

impl<T: TdfDocument> TdfDocumentExt for T {}
