use crate::segments::store::{data_store::DataStore, page_store::PagesStore};

pub struct TDFBuilder<'a> {
    pages_store: PagesStore<'a>,
    data_store: DataStore<'a>,
}

impl<'a> TDFBuilder<'a> {
    pub fn new() -> Self {
        Self {
            pages_store: PagesStore::default(),
            data_store: DataStore::default(),
        }
    }
}
