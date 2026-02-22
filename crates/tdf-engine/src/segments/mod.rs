//! A segment is one component of a TDF document.
//!
//! You can serialize or deserialize an entire segment independently of the other segments.

use std::error::Error;

pub mod footer;
pub mod header;
pub mod meta;
pub mod pages;
pub mod store;
pub mod suffix;
pub mod wire;

pub trait Segment {}

pub trait SegmentReadWrite: Sized {
    type Error: Error;

    fn dump<W: std::io::Write>(&self, writer: &mut W) -> Result<(), Self::Error>;
    fn extract<R: std::io::Read>(reader: &mut R) -> Result<Self, Self::Error>;
}
