use serde::{Deserialize, Serialize};

use crate::{
    misc::{Instant, PageAnchor, PageRef},
    tags::Tag,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct HeaderChunk<'a> {
    tags: HeaderTags<'a>,
    /// The title of the document
    document_title: &'a str,
    /// The creation date of the document
    creation_date: Instant,
    /// Offsets corresponding to all other chunks in the document.
    chunks: ChunkOffsets,
}

/// Offsets corresponding to all other chunks in the document.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkOffsets {
    pages_offset: u64,
    store_offset: u64,
    trailer_offset: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct HeaderTags<'a> {
    /// All authors of the document
    authors: Tag<Vec<&'a str>>,
    /// The table of contents.
    table_of_contents: Tag<TableOfContents<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TableOfContents<'a> {
    sections: Vec<SectionDescription<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SectionDescription<'a> {
    title: &'a str,
    depth: u8,
    page_number: PageRef,
    /// Pointer to an item on the page that the section starts at.
    ancher: Tag<PageAnchor>,
}
