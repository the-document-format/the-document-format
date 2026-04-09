//! TDF Engine — core document format implementation.
//!
//! Entry points: [`impls::TdfDocument`] for reading, [`builder::TDFBuilder`] for building.

#![feature(associated_type_defaults)]
#![feature(lazy_type_alias)]

pub mod backend;
pub mod builder;
pub mod misc;
pub mod primitives;
pub mod segments;
pub mod store;

pub mod impls;

#[cfg(test)]
mod tests {
    #[test]
    fn test_iter_page_items() {
        use crate::builder::{DummyTDFBuilder, TDFBuilder};
        use crate::impls::TdfDocument;
        use crate::primitives::item::*;

        let reader = DummyTDFBuilder::default()
            .add_page(vec![
                (
                    ItemPrimitive::Shape(Shape {
                        kind: ShapeKind::Rectangle,
                    }),
                    ItemUnique {
                        position: Position { x: 10, y: 20 },
                        ..Default::default()
                    },
                ),
                (
                    ItemPrimitive::TextBox(TextBox {
                        content: "Hello".into(),
                        font: None,
                    }),
                    ItemUnique {
                        position: Position { x: 30, y: 40 },
                        ..Default::default()
                    },
                ),
            ])
            .add_page(vec![(
                ItemPrimitive::TextBox(TextBox {
                    content: "Page 2".into(),
                    font: None,
                }),
                ItemUnique {
                    position: Position { x: 0, y: 0 },
                    ..Default::default()
                },
            )])
            .build(); // writer -> reader

        let items: Vec<_> = reader.iter_page_items(0).collect();
        assert_eq!(items.len(), 2);

        let items: Vec<_> = reader.iter_page_items(1).collect();
        assert_eq!(items.len(), 1);

        for page in 0..2 {
            println!("--- page {page} ---");
            for (primitive, unique) in reader.iter_page_items(page) {
                println!(
                    "  pos=({}, {})  item={primitive:?}",
                    unique.position.x, unique.position.y
                );
            }
        }
    }

    #[test]
    fn test_iter_page_items_primitives_and_positions() {
        use crate::builder::{DummyTDFBuilder, TDFBuilder};
        use crate::impls::TdfDocument;
        use crate::primitives::item::*;

        let reader = DummyTDFBuilder::default()
            .add_page(vec![
                (
                    ItemPrimitive::Shape(Shape {
                        kind: ShapeKind::Circle,
                    }),
                    ItemUnique {
                        position: Position { x: 1, y: 2 },
                        ..Default::default()
                    },
                ),
                (
                    ItemPrimitive::TextBox(TextBox {
                        content: "hi".into(),
                        font: None,
                    }),
                    ItemUnique {
                        position: Position { x: 3, y: 4 },
                        ..Default::default()
                    },
                ),
            ])
            .build();

        let items: Vec<_> = reader.iter_page_items(0).collect();
        assert_eq!(items.len(), 2);

        assert_eq!(
            items[0].0,
            ItemPrimitive::Shape(Shape {
                kind: ShapeKind::Circle
            })
        );
        assert_eq!(items[0].1.position, Position { x: 1, y: 2 });

        assert_eq!(
            items[1].0,
            ItemPrimitive::TextBox(TextBox {
                content: "hi".into(),
                font: None
            })
        );
        assert_eq!(items[1].1.position, Position { x: 3, y: 4 });

        // out-of-bounds page returns nothing
        assert_eq!(reader.iter_page_items(1).count(), 0);
    }

    #[test]
    fn test_round_trip_serialization() {
        use crate::backend::VecBackend;
        use crate::builder::{DummyTDFBuilder, TDFBuilder};
        use crate::impls::{ManifestRead, TDFManifest, TdfDocument};
        use crate::primitives::item::*;

        let doc = DummyTDFBuilder::default()
            .title("Round-trip test")
            .add_page(vec![
                (
                    ItemPrimitive::Shape(Shape {
                        kind: ShapeKind::Circle,
                    }),
                    ItemUnique {
                        position: Position { x: 1, y: 2 },
                        ..Default::default()
                    },
                ),
                (
                    ItemPrimitive::TextBox(TextBox {
                        content: "hello".into(),
                        font: None,
                    }),
                    ItemUnique {
                        position: Position { x: 3, y: 4 },
                        ..Default::default()
                    },
                ),
            ])
            .add_page(vec![(
                ItemPrimitive::TextBox(TextBox {
                    content: "page two".into(),
                    font: None,
                }),
                ItemUnique {
                    position: Position { x: 0, y: 0 },
                    ..Default::default()
                },
            )])
            .build();

        // Serialize to bytes.
        let mut buf = Vec::new();
        doc.to_writer(&mut buf).expect("to_writer failed");

        // Deserialize: load manifest only, inspect, then load backend.
        let mut cursor = std::io::Cursor::new(&buf);
        let manifest = TDFManifest::from_reader(&mut cursor).expect("from_reader failed");
        assert_eq!(
            manifest.meta.document_title.as_deref(),
            Some("Round-trip test")
        );
        assert_eq!(manifest.pages.page_count(), 2);

        let loaded = manifest
            .load_backend::<VecBackend, _>(cursor)
            .expect("load_backend failed");

        // Page 0: two items at correct positions.
        let page0: Vec<_> = loaded.iter_page_items(0).collect();
        assert_eq!(page0.len(), 2);
        assert_eq!(
            page0[0].0,
            ItemPrimitive::Shape(Shape {
                kind: ShapeKind::Circle
            })
        );
        assert_eq!(page0[0].1.position, Position { x: 1, y: 2 });
        assert_eq!(
            page0[1].0,
            ItemPrimitive::TextBox(TextBox {
                content: "hello".into(),
                font: None
            })
        );
        assert_eq!(page0[1].1.position, Position { x: 3, y: 4 });

        // Page 1: one item.
        let page1: Vec<_> = loaded.iter_page_items(1).collect();
        assert_eq!(page1.len(), 1);
        assert_eq!(
            page1[0].0,
            ItemPrimitive::TextBox(TextBox {
                content: "page two".into(),
                font: None
            })
        );

        // Out-of-bounds.
        assert_eq!(loaded.iter_page_items(2).count(), 0);
    }

    #[test]
    fn test_unique_reduce_position() {
        use crate::backend::UniqueReduce;
        use crate::primitives::item::{ItemUnique, Position};

        let a = ItemUnique {
            position: Position { x: 0, y: 0 },
            ..Default::default()
        };
        let b = ItemUnique {
            position: Position { x: 2, y: 2 },
            ..Default::default()
        };
        let c = ItemUnique {
            position: Position { x: 3, y: 3 },
            ..Default::default()
        };
        let result = a.reduce(b).reduce(c);
        assert_eq!(result.position, Position { x: 5, y: 5 });
    }
}
