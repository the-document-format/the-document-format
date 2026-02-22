use serde::{Deserialize, Serialize};

use crate::segments::{Segment, SegmentReadWrite};

#[derive(Debug, thiserror::Error)]
pub enum SegmentSerdeError {
    #[error("failed to serialize segment to json: {0}")]
    JsonSerialize(#[from] serde_json::Error),

    #[error("invalid segment header encoding: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("invalid segment length header: {0}")]
    InvalidLength(#[from] std::num::ParseIntError),

    #[error("invalid segment framing: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid segment framing")]
    InvalidFraming,

    #[error("segment payload out of bounds")]
    OutOfBounds,
}
impl<T> SegmentReadWrite for T
where
    T: Segment + Serialize + for<'de> Deserialize<'de>,
{
    type Error = SegmentSerdeError;

    fn dump<W: std::io::Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        let json = serde_json::to_vec(self)?;
        let json_decimal_len_string = json.len().to_string();
        let json_decimal_len_bytes = json_decimal_len_string.as_bytes();

        writer.write_all(json_decimal_len_bytes)?;
        writer.write_all(&json)?;
        Ok(())
    }

    fn extract<R: std::io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // Read ASCII decimal length prefix (at least 1 digit), then read that many bytes of JSON.
        let mut len_buf: Vec<u8> = Vec::new();
        let mut byte = [0u8; 1];

        // Read first byte; must be a digit.
        if reader.read_exact(&mut byte).is_err() {
            return Err(SegmentSerdeError::OutOfBounds);
        }
        if !byte[0].is_ascii_digit() {
            return Err(SegmentSerdeError::InvalidFraming);
        }
        len_buf.push(byte[0]);

        // Read remaining digits, leaving the first non-digit as part of the JSON by buffering it.
        let first_json_byte: u8;
        loop {
            match reader.read_exact(&mut byte) {
                Ok(()) => {
                    if byte[0].is_ascii_digit() {
                        len_buf.push(byte[0]);
                    } else {
                        first_json_byte = byte[0];
                        break;
                    }
                }
                Err(_) => return Err(SegmentSerdeError::OutOfBounds),
            }
        }

        let json_len: usize = std::str::from_utf8(&len_buf)?.parse()?;
        if json_len == 0 {
            return Err(SegmentSerdeError::InvalidFraming);
        }

        let mut json_bytes = vec![0u8; json_len];
        json_bytes[0] = first_json_byte;
        if json_len > 1 {
            reader
                .read_exact(&mut json_bytes[1..])
                .map_err(|_| SegmentSerdeError::OutOfBounds)?;
        }

        Ok(serde_json::from_slice(&json_bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestSegment {
        a: u32,
    }
    impl Segment for TestSegment {}

    #[test]
    fn three_segments_back_to_back_in_fake_file() {
        let s1 = TestSegment { a: 1 };
        let s2 = TestSegment { a: 2 };
        let s3 = TestSegment { a: 3 };

        // Write three framed segments back-to-back into a fake file.
        let mut fake_file = Cursor::new(Vec::<u8>::new());
        s1.dump(&mut fake_file).unwrap();
        s2.dump(&mut fake_file).unwrap();
        s3.dump(&mut fake_file).unwrap();

        // Rewind and read them back sequentially.
        fake_file.set_position(0);

        let r1 = TestSegment::extract(&mut fake_file).unwrap();
        let r2 = TestSegment::extract(&mut fake_file).unwrap();
        let r3 = TestSegment::extract(&mut fake_file).unwrap();

        assert_eq!(r1, s1);
        assert_eq!(r2, s2);
        assert_eq!(r3, s3);
    }
}
