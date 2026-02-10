use serde::{Deserialize, Serialize};

use crate::chunks::{
    doc_meta::DocMetaChunk, pages::PagesChunk, store::StoreChunk, trailer::TrailerChunk,
};

/// This is the core container of a TDF document.
///
/// We break down a TDF file into a few distinct length prefixed chunks.
///
/// Each chunk is serialized independently, which allows us to read the header
/// first without processing or having the rest of the document as context.
pub struct TDFReader {
    // TODO
}
