//! The content store is where all actual data in a TDF is stored.
//!
//! A store contains a giant list of many store item references. Each store item
//! reference may be the literal content of some data item (like an actual
//! image, with all the actual image data), or a pointer to some other item in
//! the big master list, using an index as a reference.

pub mod concrete;
pub mod impls;
pub mod store;
