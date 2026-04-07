//! TDF Engine — core document format implementation.
//!
//! Entry points: [`reader::TDFReader`] for reading, [`builder::TDFBuilder`] for building.

#![feature(associated_type_defaults)]
#![feature(lazy_type_alias)]

pub mod backend;
pub mod builder;
pub mod misc;
pub mod primitives;
pub mod reader;
pub mod segments;
pub mod store;
pub mod writer;

#[cfg(test)]
mod tests {
    #[test]
    fn test_iter_page_items() {
        use crate::builder::{DummyTDFBuilder, TDFBuilder};
        use crate::primitives::item::*;
        use crate::reader::TDFReader;

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
        use crate::primitives::item::*;
        use crate::reader::TDFReader;

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
