//! DocumentWrite and ManifestRead for BinaryBackend.

use std::io::{Read, Write};

use crate::impls::document::{BackedDocument, DocumentWrite, TDFManifest, TdfDocument};
use crate::segments::header::{HeaderSegment, SegmentOffsets};

use super::backend::{BinaryBackend, BinaryTypes};
use super::error::TdfBinaryError;
use super::header::{BinaryFileHeader, CURRENT_VERSION, MAGIC};
use super::{bincode_config, bincode_header_config};

impl DocumentWrite for BackedDocument<BinaryBackend> {
    fn to_writer<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let config = bincode_config();
        let header_config = bincode_header_config();
        let mut buf = Vec::new();

        // 1. Reserve space for header with a dummy (zeroed offsets).
        // Fixed-int encoding ensures the size is stable regardless of offset values.
        let dummy_header = BinaryFileHeader::default();
        let dummy_bytes = bincode_next::serde::encode_to_vec(&dummy_header, header_config)
            .map_err(|e| TdfBinaryError::Encode(e))?;
        let header_size = dummy_bytes.len();
        buf.extend_from_slice(&dummy_bytes);

        // 2. Write meta
        let meta_offset = buf.len() as u64;
        let meta_bytes = bincode_next::serde::encode_to_vec(&self.manifest.meta, config)
            .map_err(|e| TdfBinaryError::Encode(e))?;
        buf.extend_from_slice(&meta_bytes);

        // 3. Write pages
        let pages_offset = buf.len() as u64;
        let pages_bytes = bincode_next::serde::encode_to_vec(&self.manifest.pages, config)
            .map_err(|e| TdfBinaryError::Encode(e))?;
        buf.extend_from_slice(&pages_bytes);

        // 4. Write stores (raw, no framing)
        let page_store_offset = buf.len() as u64;
        buf.extend_from_slice(self.backend.page_store_bytes());
        let item_store_offset = buf.len() as u64;
        buf.extend_from_slice(self.backend.item_store_bytes());
        let data_store_offset = buf.len() as u64;
        buf.extend_from_slice(self.backend.data_store_bytes());
        let sig_store_offset = buf.len() as u64;
        buf.extend_from_slice(self.backend.sig_store_bytes());
        let file_len = buf.len() as u64;

        // 5. Build the real header and backfill
        let header = BinaryFileHeader {
            magic: MAGIC,
            version: CURRENT_VERSION,
            file_len,
            meta_offset,
            pages_offset,
            page_store_offset,
            item_store_offset,
            data_store_offset,
            sig_store_offset,
        };
        let header_bytes = bincode_next::serde::encode_to_vec(&header, header_config)
            .map_err(|e| TdfBinaryError::Encode(e))?;
        debug_assert_eq!(
            header_bytes.len(),
            header_size,
            "header size changed after filling offsets"
        );
        buf[..header_size].copy_from_slice(&header_bytes);

        writer.write_all(&buf)?;
        Ok(())
    }
}

impl BackedDocument<BinaryBackend> {
    /// Read a binary TDF document from a reader.
    pub fn from_binary_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let config = bincode_config();
        let header_config = bincode_header_config();

        // 1. Decode header from start of buffer
        let (header, header_consumed): (BinaryFileHeader, usize) =
            bincode_next::serde::decode_from_slice(&buf, header_config)
                .map_err(|e| TdfBinaryError::Decode(e))?;

        // 2. Validate
        if header.magic != MAGIC {
            return Err(TdfBinaryError::InvalidMagic.into());
        }
        if header.version != CURRENT_VERSION {
            return Err(TdfBinaryError::UnsupportedVersion(header.version).into());
        }

        // 3. Decode meta at meta_offset
        let (meta, _) =
            bincode_next::serde::decode_from_slice(&buf[header.meta_offset as usize..], config)
                .map_err(|e| TdfBinaryError::Decode(e))?;

        // 4. Decode pages at pages_offset
        let (pages, _) =
            bincode_next::serde::decode_from_slice(&buf[header.pages_offset as usize..], config)
                .map_err(|e| TdfBinaryError::Decode(e))?;

        // 5. Slice store regions
        let page_store =
            buf[header.page_store_offset as usize..header.item_store_offset as usize].to_vec();
        let item_store =
            buf[header.item_store_offset as usize..header.data_store_offset as usize].to_vec();
        let data_store =
            buf[header.data_store_offset as usize..header.sig_store_offset as usize].to_vec();
        // let sig_store = buf[header.sig_store_offset as usize..header.file_len as usize].to_vec();
        let sig_store = buf[header.sig_store_offset as usize..header.file_len as usize].to_vec();

        let backend =
            BinaryBackend::from_store_bytes(page_store, item_store, data_store, sig_store);

        Ok(BackedDocument {
            manifest: TDFManifest {
                header: HeaderSegment::new(0, SegmentOffsets::new(0, 0, 0)),
                meta,
                pages,
            },
            backend,
            page_frontend: Default::default(),
            item_frontend: Default::default(),
            data_frontend: Default::default(),
            sig_frontend: Default::default(),
        })
    }
}

impl TdfDocument for BackedDocument<BinaryBackend> {
    type B = BinaryBackend;

    fn manifest(&self) -> &TDFManifest<BinaryTypes> {
        &self.manifest
    }

    fn stores(&mut self) -> crate::impls::document::StoreAccess<'_, BinaryBackend> {
        crate::impls::document::StoreAccess {
            backend: &mut self.backend,
            pages_store: &self.page_frontend,
            item_store: &self.item_frontend,
            data_store: &self.data_frontend,
            signature_store: &self.sig_frontend,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impls::binary::builder::BinaryTDFBuilder;
    use crate::impls::document::DocumentWrite;
    use crate::primitives::data::{DataPrimitive, ImageData};
    use crate::primitives::item::*;

    #[test]
    fn binary_write_has_magic_bytes() {
        let doc = BinaryTDFBuilder::new()
            .title("test doc")
            .add_page(vec![(
                ItemPrimitive::Shape(Shape {
                    kind: ShapeKind::Circle,
                }),
                ItemUnique::default(),
            )])
            .build();

        let mut buf = Vec::new();
        doc.to_writer(&mut buf).expect("to_writer failed");

        // Decode header directly from the start of the buffer
        let header_config = super::bincode_header_config();
        let (header, _): (BinaryFileHeader, usize) =
            bincode_next::serde::decode_from_slice(&buf, header_config)
                .expect("decode header failed");

        assert_eq!(header.magic, *b"TREVDF");
        assert_eq!(header.version, 1);
        assert_eq!(header.file_len, buf.len() as u64);
        assert!(header.meta_offset > 0);
        assert!(header.pages_offset > header.meta_offset);
        assert!(header.page_store_offset > header.pages_offset);
    }

    #[test]
    fn binary_round_trip_manifest() {
        let mut builder = BinaryTDFBuilder::new();
        let img_ptr = builder.stage_data(DataPrimitive::ImageData(ImageData {
            bytes: vec![0xFF, 0x00, 0xFF],
        }));

        let doc = builder
            .title("round trip")
            .add_page(vec![
                (
                    ItemPrimitive::TextBox(TextBox {
                        content: "hello".into(),
                        font: None,
                    }),
                    ItemUnique {
                        position: Position { x: 10, y: 20 },
                        ..Default::default()
                    },
                ),
                (
                    ItemPrimitive::Image(Image {
                        width: 100,
                        height: 200,
                        data: img_ptr,
                    }),
                    ItemUnique::default(),
                ),
            ])
            .build();

        // Write
        let mut buf = Vec::new();
        doc.to_writer(&mut buf).expect("to_writer failed");

        // Read back
        let mut cursor = std::io::Cursor::new(&buf);
        let loaded = BackedDocument::<BinaryBackend>::from_binary_reader(&mut cursor)
            .expect("from_binary_reader failed");

        assert_eq!(
            loaded.manifest.meta.document_title.as_deref(),
            Some("round trip")
        );
        assert_eq!(loaded.manifest.pages.page_count(), 1);
    }
}
