pub use femtovg;

use femtovg::{Baseline, Canvas, Color, FontId, ImageFlags, Paint, Path, renderer::Renderer};
pub use tdf_engine;
use tdf_engine::backend::{CacheHints, VecBackend};
pub use tdf_engine::impls::TdfDocument;
use tdf_engine::impls::{BackedDocument, TdfDocumentExt};
use tdf_engine::primitives::data::DataPrimitive;
use tdf_engine::primitives::item::{Image, ItemPrimitive, ItemUnique, Vector};

pub type Document = BackedDocument<VecBackend>;

pub const PAGE_WIDTH: u32 = 850 / 2;
pub const PAGE_HEIGHT: u32 = 1100 / 2;

const DEFAULT_FONT_SIZE: f32 = 24.0;
const DEFAULT_STROKE_WIDTH: f32 = 2.0;
fn default_text_color() -> Color {
    Color::rgb(50, 100, 255)
}
const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/LiberationSans-Regular.ttf");

#[derive(Clone, Copy, Debug)]
pub struct RendererFonts {
    default_font: FontId,
}

pub fn load_fonts<T: Renderer>(
    canvas: &mut Canvas<T>,
) -> Result<RendererFonts, femtovg::ErrorKind> {
    let default_font = canvas.add_font_mem(FONT_BYTES)?;
    Ok(RendererFonts { default_font })
}

fn render_text<T: Renderer>(
    canvas: &mut Canvas<T>,
    fonts: &RendererFonts,
    content: &str,
    unique: &ItemUnique,
) {
    let font_size = unique
        .tags
        .font_size
        .map(|s| s as f32)
        .unwrap_or(DEFAULT_FONT_SIZE);

    let color = unique
        .tags
        .fill_color
        .as_ref()
        .map(|c| Color::rgba(c.r, c.g, c.b, c.a))
        .unwrap_or_else(default_text_color);

    let paint = Paint::color(color)
        .with_font(&[fonts.default_font])
        .with_font_size(font_size)
        .with_text_baseline(Baseline::Top);

    canvas
        .fill_text(
            unique.position.x as f32,
            unique.position.y as f32,
            content,
            &paint,
        )
        .expect("failed to render text");
}

fn render_image<T: Renderer>(
    canvas: &mut Canvas<T>,
    img: &Image<<VecBackend as tdf_engine::backend::Backend>::Types>,
    unique: &ItemUnique,
    bytes: &[u8],
) {
    let image_id = match canvas.load_image_mem(bytes, ImageFlags::empty()) {
        Ok(id) => id,
        Err(_) => return,
    };

    let x = unique.position.x as f32;
    let y = unique.position.y as f32;
    let w = img.width as f32;
    let h = img.height as f32;

    // Paint::image centers the image at (cx, cy); offset by half-size to place
    // the top-left at (x, y).
    let paint = Paint::image(image_id, x, y, w, h, 0.0, 1.0);
    let mut path = Path::new();
    path.rect(x, y, w, h);
    canvas.fill_path(&path, &paint);

    // We created a fresh ImageId for this draw; free the GPU texture.
    canvas.delete_image(image_id);
}

fn render_vector<T: Renderer>(canvas: &mut Canvas<T>, vector: &Vector, unique: &ItemUnique) {
    if vector.points.is_empty() {
        return;
    }

    let ox = unique.position.x as f32;
    let oy = unique.position.y as f32;

    let mut path = Path::new();
    let first = &vector.points[0];
    path.move_to(ox + first.x as f32, oy + first.y as f32);
    for pt in &vector.points[1..] {
        path.line_to(ox + pt.x as f32, oy + pt.y as f32);
    }

    let paint = Paint::color(Color::rgb(34, 34, 34)).with_line_width(DEFAULT_STROKE_WIDTH);
    canvas.stroke_path(&path, &paint);
}

pub fn render_page<T: Renderer>(
    canvas: &mut Canvas<T>,
    fonts: &RendererFonts,
    doc: &mut Document,
    page_number: usize,
) {
    let items = doc.iter_page_items(page_number, CacheHints::Cache);
    for (item, unique) in items {
        match item {
            ItemPrimitive::TextBox(ref t) => render_text(canvas, fonts, &t.content, &unique),
            ItemPrimitive::Vector(ref v) => render_vector(canvas, v, &unique),
            ItemPrimitive::Image(ref img) => {
                if let Some(DataPrimitive::ImageData(data)) = doc.fetch_data_item(&img.data) {
                    render_image(canvas, img, &unique, &data.bytes);
                }
            }
            ItemPrimitive::Shape(_) => {}
        }
    }
}
