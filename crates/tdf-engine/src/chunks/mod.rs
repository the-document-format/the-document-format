//! A chunk is one component of a TDF document.
//!
//! You can serialize or deserialize an entire chunk independently of the other chunks.

pub(super) mod doc_meta;
pub(super) mod pages;
pub(super) mod prefix;
pub(super) mod store;
pub(super) mod trailer;
