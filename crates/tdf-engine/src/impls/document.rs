use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use crate::backend::{Backend, BackendTypes, vec_backend::VecTypes};
use crate::primitives::item::{ItemPrimitive, ItemUnique};
use crate::segments::{header::HeaderSegment, meta::MetaSegment, pages::PagesSegment};

/// The cheaply-loaded, copied portion of a TDF file: header, meta, and page list.
/// This is the part of the TDF that does not have the stores; they follow this section.
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
pub struct BackedDocument<B: Backend> {
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

/// Trait for reading a TDF manifest (header, meta, pages) and
/// optionally loading its backend from a byte stream.
///
/// The associated type `BTypes` ties the manifest to the concrete
/// [`BackendTypes`] it was serialised with, so that [`load_backend`]
/// can enforce the backend-to-types pairing at the call site.
///
/// [`load_backend`]: ManifestRead::load_backend
pub trait ManifestRead: Sized {
    /// The [`BackendTypes`] this manifest was serialised with.
    type BTypes: BackendTypes;

    /// Deserialise header, meta, and pages segments from `reader`.
    ///
    /// The stream must contain three consecutive length-prefixed JSON
    /// records in the order: header, meta, pages.
    fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self>;

    /// Consume the manifest and deserialise the backend from `reader`,
    /// producing a fully-loaded [`BackedDocument`].
    ///
    /// `reader` is taken by value because no further reads are expected
    /// after the backend segment; callers need not retain the stream.
    ///
    /// The stream must contain exactly one length-prefixed JSON record
    /// immediately following the three manifest segments.
    fn load_backend<B, R>(self, reader: R) -> std::io::Result<BackedDocument<B>>
    where
        B: Backend<Types = Self::BTypes> + DeserializeOwned,
        R: Read;
}

/// Trait for reading content out of a TDF document backed by [`VecBackend`].
pub trait TdfDocument {
    fn manifest(&self) -> &TDFManifest<VecTypes>;

    fn iter_page_items(
        &self,
        page_number: usize,
    ) -> Box<dyn Iterator<Item = (ItemPrimitive<VecTypes>, ItemUnique)> + '_>;
}
