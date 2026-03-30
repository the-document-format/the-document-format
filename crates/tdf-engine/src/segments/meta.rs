use serde::{Deserialize, Serialize};
use crate::misc::{Instant, PageRef, PageAnchor};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MetaSegment {
    pub document_title: Option<String>,
    pub search_index: Option<SearchIndex>,
    pub table_of_contents: Option<TableOfContents>,
    pub tags: DocumentTags,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchIndex {
    // TODO
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DocumentTags {
    pub authors: Option<Vec<String>>,
    pub creation_date: Option<Instant>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableOfContents {
    pub sections: Vec<SectionDescription>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SectionDescription {
    pub title: String,
    pub depth: u8,
    pub page_number: PageRef,
    pub anchor: Option<PageAnchor>,
}
