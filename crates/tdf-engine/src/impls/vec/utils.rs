use serde::{Serialize, de::DeserializeOwned};
use std::io::{Read, Write};

/// Wire format: ASCII decimal byte-count immediately followed by JSON bytes, no separator.
///
/// Example: `18{"title":"hello"}`
///
/// A complete TDF file is four back-to-back records:
/// `<len>HeaderSegment  <len>MetaSegment  <len>PagesSegment  <len>VecBackend`
pub fn read_length_prefixed<R: Read, T: DeserializeOwned>(reader: &mut R) -> std::io::Result<T> {
    // Read ASCII digits until the first non-digit byte (which is the start of JSON).
    let mut len_buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        reader.read_exact(&mut byte)?;
        if byte[0].is_ascii_digit() {
            len_buf.push(byte[0]);
        } else {
            break;
        }
    }

    let len: usize = std::str::from_utf8(&len_buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        .parse()
        .map_err(|e: std::num::ParseIntError| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?;

    // `byte[0]` holds the first non-digit — the first byte of the JSON body.
    let mut json_buf = vec![0u8; len];
    json_buf[0] = byte[0];
    reader.read_exact(&mut json_buf[1..])?;

    serde_json::from_slice(&json_buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn write_length_prefixed<W: Write, T: Serialize>(
    writer: &mut W,
    value: &T,
) -> std::io::Result<()> {
    let json = serde_json::to_vec(value)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    write!(writer, "{}", json.len())?;
    writer.write_all(&json)
}
