use crate::segments::{header::HeaderSegment, meta::MetaSegment, pages::PagesSegment, store::concrete::page_store::{PageItemPrimative, PageItemUnique}};

pub trait TDFReader<'a> {
    // Getters for the segments of the store

    fn header(&'a self) -> &'a HeaderSegment;
    fn meta(&'a self) -> &'a MetaSegment;
    fn pages(&'a self) -> &'a PagesSegment;

    // Helpers to interact with content
    fn get_page_items(&'a self, page_id: usize) -> impl Iterator<Item = (PageItemPrimative, PageItemUnique)>;
}
