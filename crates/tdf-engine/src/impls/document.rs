use std::io::Write;

use serde::Serialize;

use crate::backend::{vec_backend::VecTypes, Backend, BackendTypes, VecBackend};
use crate::primitives::item::{ItemPrimitive, ItemUnique};
use crate::segments::{header::HeaderSegment, meta::MetaSegment, pages::PagesSegment};

/// The cheaply-loaded portion of a TDF file: header, meta, and page list.
///
/// Load this first to inspect the document before committing to reading the stores.
/// Call [`TDFManifest::load_backend`] (defined in the backend-specific module) to
/// produce a fully-loaded [`BackedDocument`].
pub struct TDFManifest<B: BackendTypes = VecTypes> {
    pub header: HeaderSegment,
    pub meta: MetaSegment,
    pub pages: PagesSegment<B>,
}

/// A fully-loaded TDF document: manifest (header/meta/pages) plus a backend `B`.
pub struct BackedDocument<B: Backend = VecBackend> {
    pub manifest: TDFManifest<B::Types>,
    pub backend: B,
}

impl<B: Backend + Serialize> BackedDocument<B> {
    /// Serialize the document to `writer` as four length-prefixed JSON records.
    pub fn to_writer<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        use crate::impls::vec::utils::write_length_prefixed;
        write_length_prefixed(writer, &self.manifest.header)?;
        write_length_prefixed(writer, &self.manifest.meta)?;
        write_length_prefixed(writer, &self.manifest.pages)?;
        write_length_prefixed(writer, &self.backend)
    }
}

/// Trait for reading content out of a TDF document backed by [`VecBackend`].
pub trait TdfDocument {
    fn manifest(&self) -> &TDFManifest<VecTypes>;

    fn iter_page_items(
        &self,
        page_number: usize,
    ) -> Box<dyn Iterator<Item = (ItemPrimitive<VecTypes>, ItemUnique)> + '_>;
}
