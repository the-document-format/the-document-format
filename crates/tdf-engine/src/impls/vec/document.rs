use std::io::{Read, Write};

use serde::de::DeserializeOwned;

use crate::backend::{
    Backend, BackendAccess, BackendPointer, StoreItemCell, VecBackend, vec_backend::VecTypes,
};
use crate::impls::document::{
    BackedDocument, DocumentWrite, ManifestRead, TDFManifest, TdfDocument,
};
use crate::primitives::item::{ItemPrimitive, ItemTypes, ItemUnique};
use crate::primitives::page::PageTypes;

use super::utils::read_length_prefixed;

impl ManifestRead for TDFManifest<VecTypes> {
    type BTypes = VecTypes;

    fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let header = read_length_prefixed(reader)?;
        let meta = read_length_prefixed(reader)?;
        let pages = read_length_prefixed(reader)?;
        Ok(Self {
            header,
            meta,
            pages,
        })
    }

    fn load_backend<B, R>(self, mut reader: R) -> std::io::Result<BackedDocument<B>>
    where
        B: Backend<Types = VecTypes> + DeserializeOwned,
        R: Read,
    {
        let backend = read_length_prefixed(&mut reader)?;
        Ok(BackedDocument {
            manifest: self,
            backend,
        })
    }
}

impl TdfDocument for BackedDocument<VecBackend> {
    type B = VecBackend;

    fn backend(&self) -> &VecBackend {
        &self.backend
    }

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

impl DocumentWrite for BackedDocument<VecBackend> {
    fn to_writer<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        use super::utils::write_length_prefixed;
        write_length_prefixed(writer, &self.manifest.header)?;
        write_length_prefixed(writer, &self.manifest.meta)?;
        write_length_prefixed(writer, &self.manifest.pages)?;
        write_length_prefixed(writer, &self.backend)
    }
}
