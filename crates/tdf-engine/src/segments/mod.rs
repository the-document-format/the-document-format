//! A segment is one component of a TDF document.
//!
//! You can serialize or deserialize an entire segment independently of the other segments.

pub mod footer;
pub mod header;
pub mod meta;
pub mod pages;
pub mod store;
pub mod suffix;
