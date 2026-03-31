use crate::{primitives::data::DataStorePointer, store::traits::StoreTypes};
use serde::{Deserialize, Serialize};

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
    fn reduce(self, other: Self) -> Self {
        Self {
            position: Position {
                x: self.position.x + other.position.x,
                y: self.position.y + other.position.y,
            },
            tags: ItemTags {
                font_size: other.tags.font_size.or(self.tags.font_size),
                stroke_width: other.tags.stroke_width.or(self.tags.stroke_width),
                stroke_color: other.tags.stroke_color.or(self.tags.stroke_color),
                fill_color: other.tags.fill_color.or(self.tags.fill_color),
                opacity: other.tags.opacity.or(self.tags.opacity),
                text_alignment: other.tags.text_alignment.or(self.tags.text_alignment),
            },
        }
    }
}

pub struct ItemTypes;

impl StoreTypes for ItemTypes {
    type Primitive = ItemPrimitive;
    type Unique = ItemUnique;
}
