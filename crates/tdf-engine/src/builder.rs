use crate::segments::store::concrete::{data_store::DataStore, page_store::PagesStore};

#[derive(Debug, Default)]
pub struct TDFBuilder<'a> {
    pages_store: PagesStore<'a>,
    data_store: DataStore<'a>,
}
