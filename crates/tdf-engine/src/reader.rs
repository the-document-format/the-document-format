use crate::segments::{
    SegmentReadWrite, SegmentSerdeError, header::HeaderSegment, meta::MetaSegment,
};

#[derive(Debug, thiserror::Error)]
pub enum TDFReaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SegmentSerde(#[from] SegmentSerdeError),
}

#[derive(Debug)]
pub struct TDFReader<'a, T>
where
    T: std::io::Seek + std::io::Read,
{
    file: T,
    state: TDFReaderState<'a>,
}

impl<'a, T> TDFReader<'a, T>
where
    T: std::io::Seek + std::io::Read,
{
    pub fn new(file: T) -> Self {
        TDFReader {
            file,
            state: TDFReaderState::BrandNew,
        }
    }

    pub fn read_header(&mut self) -> Result<(), TDFReaderError> {
        match self.state {
            TDFReaderState::BrandNew => {
                self.state = {
                    TDFReaderState::WithHeader(TDFReaderWithHeader {
                        header: HeaderSegment::extract(&mut self.file)?,
                    })
                };

                Ok(())
            }
            _ => Ok(()), // we've already read it
        }
    }
}

#[derive(Debug)]
pub enum TDFReaderState<'a> {
    BrandNew,
    WithHeader(TDFReaderWithHeader),
    WithHeaderAndMeta(TDFReaderWithHeaderAndMeta<'a>),
}

#[derive(Debug)]
pub struct TDFReaderWithHeader {
    header: HeaderSegment,
}

#[derive(Debug)]
pub struct TDFReaderWithHeaderAndMeta<'a> {
    header: HeaderSegment,
    meta: MetaSegment<'a>,
}

#[cfg(test)]
mod tests {
    use crate::segments::header::SegmentOffsets;

    use super::*;
    use std::io::Cursor;

    #[test]
    fn can_read_header_even_with_extra_bytes_after() {
        let header = HeaderSegment::new(999, SegmentOffsets::new(123, 456));
        let mut writer = Cursor::new(Vec::new());
        header.dump(&mut writer).unwrap();

        writer.set_position(0);
        let mut reader = TDFReader::new(writer);
        reader.read_header().unwrap();
    }
}
