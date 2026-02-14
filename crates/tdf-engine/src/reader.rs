use serde::{Deserialize, Serialize};

use crate::segments::header::HeaderSegment;

#[derive(Debug, Deserialize, Serialize)]
pub struct TDFReader<T>
where
    T: std::io::Seek + std::io::Read,
{
    file: T,
    state: TDFReaderState,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TDFReaderState {
    BrandNew,
    WithHeader(TDFReaderWithHeader),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TDFReaderWithHeader {
    header: HeaderSegment,
}
