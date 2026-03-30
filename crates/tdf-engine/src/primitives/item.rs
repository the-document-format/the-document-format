use serde::{Deserialize, Serialize};
use crate::primitives::data::DataStorePointer;

/// Everything that can appear on a page.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemPrimitive {
    TextBox(TextBox),
    Image(Image),
    Vector(Vector),
    Shape(Shape),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextBox {
    pub content: String,
    pub font: Option<DataStorePointer>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: DataStorePointer,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Vector {
    pub points: Vec<BezierPoint>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shape {
    pub kind: ShapeKind,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShapeKind {
    Circle,
    Rectangle,
    Triangle,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BezierPoint {
    pub x: i64,
    pub y: i64,
}

/// Non-internable data that travels with every item pointer and accumulates during traversal.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ItemUnique {
    pub position: Position,
    pub tags: ItemTags,
}

/// 2D position in document units. Adds component-wise during unique reduction.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

/// Optional style/metadata for any `ItemPrimitive` variant.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ItemTags {
    pub font_size: Option<u32>,
    pub stroke_width: Option<u32>,
    pub stroke_color: Option<Color>,
    pub fill_color: Option<Color>,
    pub opacity: Option<u8>,
    pub text_alignment: Option<TextAlignment>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

impl crate::store::traits::PrimitiveType for ItemPrimitive {}
impl crate::store::traits::UniqueType for ItemUnique {}
impl crate::backend::UniqueReduce for ItemUnique {
    fn reduce(self, other: Self) -> Self { todo!() }
}
