use std::io::{Read, Write};

use serde::de::DeserializeOwned;

use crate::backend::Backend;
use crate::impls::document::{
    BackedDocument, DocumentWrite, ManifestRead, StoreAccess, TDFManifest, TdfDocument,
};
use crate::impls::vec::backend::{VecBackend, VecTypes};

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
        let backend: B = read_length_prefixed(&mut reader)?;
        Ok(BackedDocument {
            manifest: self,
            backend,
            page_frontend: Default::default(),
            item_frontend: Default::default(),
            data_frontend: Default::default(),
            sig_frontend: Default::default(),
        })
    }
}

impl TdfDocument for BackedDocument<VecBackend> {
    type B = VecBackend;

    fn manifest(&self) -> &TDFManifest<VecTypes> {
        &self.manifest
    }

    fn stores(&mut self) -> StoreAccess<'_, VecBackend> {
        StoreAccess {
            backend: &mut self.backend,
            pages_store: &self.page_frontend,
            item_store: &self.item_frontend,
            data_store: &self.data_frontend,
            signature_store: &self.sig_frontend,
        }
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
