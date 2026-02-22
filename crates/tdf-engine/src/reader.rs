use crate::segments::{header::HeaderSegment, meta::MetaSegment};

#[derive(Debug, thiserror::Error)]
pub enum TDFReaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
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
                let buf = self.read_until_first_newline()?;

                self.state = TDFReaderState::WithHeader(TDFReaderWithHeader {
                    header: serde_json::from_slice(&buf)?,
                });

                Ok(())
            }
            _ => Ok(()), // we've already read it
        }
    }

    fn read_until_first_newline(&mut self) -> Result<Vec<u8>, TDFReaderError> {
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];

        loop {
            self.file.read_exact(&mut byte)?;
            if byte[0] == b'\n' {
                break;
            }
            buf.push(byte[0]);
        }

        Ok(buf)
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
    use std::io::{Cursor, Write};

    #[test]
    fn can_read_header_even_with_extra_bytes_after() {
        // Build a basic header segment. We assume the HeaderSegment is serde-serializable.
        // If new fields are added later, update this constructor accordingly.
        let header = HeaderSegment::new(999, SegmentOffsets::new(123, 456));

        let header_bytes = serde_json::to_vec(&header).expect("serialize header");

        let mut file_bytes = Vec::new();
        file_bytes
            .write_all(&header_bytes)
            .expect("write header segment bytes");
        file_bytes
            .write_all(b"\nthis is gibberish after the header")
            .expect("write trailing bytes");

        let cursor = Cursor::new(file_bytes);
        let mut reader = TDFReader::new(cursor);

        reader.read_header().expect("read header");
    }
}
