pub mod binary;
pub mod document;
pub mod vec;

pub use binary::{BinaryBackend, BinaryTDFBuilder, BinaryTypes, TdfBinaryError};
pub use document::{
    BackedDocument, DocumentWrite, ManifestRead, TDFManifest, TdfDocument, TdfDocumentExt,
};
pub use vec::{DummyTDFBuilder, TDFBuilder};
