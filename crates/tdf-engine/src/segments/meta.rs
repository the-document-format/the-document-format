use serde::{Deserialize, Serialize};

use crate::misc::{Instant, PageAnchor, PageRef};

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaSegment<'a> {
    /// The title of the document
    document_title: Option<&'a str>,
    /// Search index for the file.
    index: Option<SearchIndex>,
    /// The table of contents.
    table_of_contents: Option<TableOfContents<'a>>,
    /// All tags relating to the overall document.
    tags: HeaderTags<'a>,
}

impl MetaSegment<'_> {
    pub fn new() -> Self {
        Self {
            document_title: None,
            index: None,
            table_of_contents: None,
            tags: HeaderTags::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchIndex {
    // TODO
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct HeaderTags<'a> {
    /// All authors of the document
    authors: Option<Vec<&'a str>>,
    /// The creation date of the document
    creation_date: Option<Instant>,
}

impl HeaderTags<'_> {
    pub fn new() -> Self {
        Self {
            authors: None,
            creation_date: None,
        }
    }
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
    ancher: Option<PageAnchor>,
}
