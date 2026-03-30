//! TDF Engine — core document format implementation.
//!
//! Entry points: [`reader::TDFReader`] for reading, [`builder::TDFBuilder`] for building.

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

        let reader = DummyTDFBuilder::new()
            .add_page(vec![
                (
                    ItemPrimitive::Shape(Shape { kind: ShapeKind::Rectangle }),
                    ItemUnique { position: Position { x: 10, y: 20 }, ..Default::default() },
                ),
                (
                    ItemPrimitive::TextBox(TextBox { content: "Hello".into(), font: None }),
                    ItemUnique { position: Position { x: 30, y: 40 }, ..Default::default() },
                ),
            ])
            .add_page(vec![
                (
                    ItemPrimitive::TextBox(TextBox { content: "Page 2".into(), font: None }),
                    ItemUnique { position: Position { x: 0, y: 0 }, ..Default::default() },
                ),
            ])
            .build();

        let items: Vec<_> = reader.iter_page_items(0).collect();
        assert_eq!(items.len(), 2);

        let items: Vec<_> = reader.iter_page_items(1).collect();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_unique_reduce_position() {
        use crate::backend::UniqueReduce;
        use crate::primitives::item::{ItemUnique, Position};

        let a = ItemUnique { position: Position { x: 0, y: 0 }, ..Default::default() };
        let b = ItemUnique { position: Position { x: 2, y: 2 }, ..Default::default() };
        let c = ItemUnique { position: Position { x: 3, y: 3 }, ..Default::default() };
        let result = a.reduce(b).reduce(c);
        assert_eq!(result.position, Position { x: 5, y: 5 });
    }
}
