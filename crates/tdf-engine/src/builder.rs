use crate::segments::{
    footer::FooterSegement,
    header::Compression,
    meta::MetaSegment,
    pages::PagesSegment,
    store::{data_store::DataStore, page_store::PagesStore, StoreItemRef},
};

pub struct TDFBuilder<'a> {
    segments: TDFSegements<'a>,
    pages_store: PagesStore,
    data_store: DataStore,
}

impl<'a> TDFBuilder<'a> {
    pub fn new() -> Self {
        Self {
            segments: TDFSegements {
                compression: Compression::None,
                meta_segment: MetaSegment::new(),
                pages_segment: PagesSegment::new(StoreItemRef::StoreItems(Default::default())),
                footer_segment: FooterSegement::new(),
            },
            pages_store: PagesStore::default(),
            data_store: DataStore::default(),
        }
    }
}

struct TDFSegements<'a> {
    compression: Compression,
    meta_segment: MetaSegment<'a>,
    pages_segment: PagesSegment,
    footer_segment: FooterSegement,
}
