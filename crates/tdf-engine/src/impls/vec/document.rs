use std::io::Read;

use serde::de::DeserializeOwned;

use crate::backend::{
    vec_backend::VecTypes, Backend, BackendAccess, BackendPointer, StoreItemCell, VecBackend,
};
use crate::impls::document::{BackedDocument, TDFManifest, TdfDocument};
use crate::primitives::item::{ItemPrimitive, ItemTypes, ItemUnique};
use crate::primitives::page::PageTypes;

use super::utils::read_length_prefixed;

impl TDFManifest<VecTypes> {
    /// Read header, meta, and pages segments from `reader`.
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let header = read_length_prefixed(reader)?;
        let meta = read_length_prefixed(reader)?;
        let pages = read_length_prefixed(reader)?;
        Ok(Self {
            header,
            meta,
            pages,
        })
    }

    /// Consume the manifest and read the backend from `reader`, producing a full document.
    pub fn load_backend<B, R>(self, reader: &mut R) -> std::io::Result<BackedDocument<B>>
    where
        B: Backend<Types = VecTypes> + DeserializeOwned,
        R: Read,
    {
        let backend = read_length_prefixed(reader)?;
        Ok(BackedDocument {
            manifest: self,
            backend,
        })
    }
}

impl TdfDocument for BackedDocument<VecBackend> {
    fn manifest(&self) -> &TDFManifest<VecTypes> {
        &self.manifest
    }

    fn iter_page_items(
        &self,
        page_number: usize,
    ) -> Box<dyn Iterator<Item = (ItemPrimitive<VecTypes>, ItemUnique)> + '_> {
        let page_entry = match self.manifest.pages.get_page(page_number) {
            Some(e) => e,
            None => return Box::new(std::iter::empty()),
        };

        let cells = match <VecBackend as BackendAccess<PageTypes<VecTypes>, VecBackend>>::get_cells(
            &self.backend,
            &page_entry.page_ref,
        ) {
            Some(c) => c,
            None => return Box::new(std::iter::empty()),
        };

        let item_ptr = match cells.into_iter().next() {
            Some(StoreItemCell::StorePrimitive(p)) => p.clone(),
            _ => return Box::new(std::iter::empty()),
        };

        let item_ptrs: Vec<BackendPointer<ItemTypes<VecTypes>, VecTypes>> = match &item_ptr {
            BackendPointer::Group(g) => <VecBackend as BackendAccess<
                ItemTypes<VecTypes>,
                VecBackend,
            >>::expand_group(&self.backend, g),
            BackendPointer::Single(_) => vec![item_ptr],
        };

        let items: Vec<(ItemPrimitive<VecTypes>, ItemUnique)> = item_ptrs
            .into_iter()
            .filter_map(|ptr| {
                let unique = match &ptr {
                    BackendPointer::Single(s) => s.unique.clone(),
                    BackendPointer::Group(_) => todo!("nested group support not yet implemented"),
                };
                let cells =
                    <VecBackend as BackendAccess<ItemTypes<VecTypes>, VecBackend>>::get_cells(
                        &self.backend,
                        &ptr,
                    )?;
                match cells.into_iter().next()? {
                    StoreItemCell::StorePrimitive(p) => Some((p.clone(), unique)),
                    _ => None,
                }
            })
            .collect();

        Box::new(items.into_iter())
    }
}
