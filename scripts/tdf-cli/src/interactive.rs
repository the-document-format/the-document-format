use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, ValueEnum};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tdf_engine::backend::VecBackend;
use tdf_engine::builder::TDFBuilder;
use tdf_engine::impls::DocumentWrite;
use tdf_engine::impls::binary::backend::BinaryBackend;
use tdf_engine::primitives::data::{DataPrimitive, DataStorePointer, ImageData};
use tdf_engine::primitives::item::{
    Image, ItemPrimitive, ItemUnique, Position, Shape, ShapeKind, TextBox,
};

// 9x20 squid pixel art
#[rustfmt::skip]
const SQUID_PIXELS: [[u8; 9]; 20] = [
    [0, 0, 0, 0, 1, 0, 0, 0, 0],
    [0, 0, 0, 1, 1, 1, 0, 0, 0],
    [0, 0, 1, 1, 1, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 1, 1, 0, 0],
    [0, 1, 1, 1, 1, 1, 1, 1, 0],
    [0, 1, 1, 1, 1, 1, 1, 1, 0],
    [1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 1, 0, 1, 1, 1, 0, 1, 1],
    [1, 1, 1, 1, 1, 1, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 1, 0],
    [0, 1, 1, 1, 1, 1, 1, 1, 0],
    [0, 0, 1, 1, 1, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 1, 1, 0, 0],
    [0, 0, 0, 1, 1, 1, 0, 0, 0],
    [1, 0, 1, 0, 1, 0, 1, 0, 1],
    [1, 0, 1, 0, 1, 0, 1, 0, 1],
    [1, 0, 0, 0, 1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 1],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
];
const SQUID_W: u32 = 9;
const SQUID_H: u32 = 20;

fn squid_rgb_bytes() -> Vec<u8> {
    SQUID_PIXELS
        .iter()
        .flat_map(|row| {
            row.iter().flat_map(|&px| {
                if px == 1 {
                    [148u8, 103, 189]
                } else {
                    [32u8, 178, 170]
                }
            })
        })
        .collect()
}

struct Rng(u64);

impl Rng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64)
            .unwrap_or(12345);
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next_u64() % (hi - lo) as u64) as i64
    }
}

#[derive(ValueEnum, Clone, Default)]
enum Backend {
    #[default]
    Json,
    Binary,
    Pdf,
}

#[derive(Parser)]
#[command(name = "tdf-interactive", about = "Interactively build a TDF document")]
struct Args {
    /// Output base name or path (extension added automatically)
    output: Option<String>,
    #[arg(long, value_enum, default_value_t = Backend::Json)]
    backend: Backend,
    /// Export to all backends (output.tdfi, output.tdf, output.pdf)
    #[arg(long)]
    all: bool,
}

#[derive(Clone)]
enum ItemSpec {
    Text { content: String, x: i64, y: i64 },
    Shape { kind: ShapeKind, x: i64, y: i64 },
    Squid { x: i64, y: i64 },
}

// Print a line in raw mode (col 0, then content, then \r\n).
macro_rules! rln {
    ($($arg:tt)*) => {{
        print!("\r{}\r\n", format!($($arg)*));
        let _ = io::stdout().flush();
    }};
}

// Print a prompt in raw mode (no trailing newline).
macro_rules! rprompt {
    ($($arg:tt)*) => {{
        print!("\r{}", format!($($arg)*));
        let _ = io::stdout().flush();
    }};
}

/// Read one keypress, echo it, and advance to the next line.
fn read_key() -> io::Result<Option<char>> {
    loop {
        match read()? {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match code {
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => return Ok(None),
                KeyCode::Esc => return Ok(None),
                KeyCode::Char(c) => {
                    print!("{}\r\n", c);
                    io::stdout().flush()?;
                    return Ok(Some(c));
                }
                _ => continue,
            },
            _ => continue,
        }
    }
}

/// Read a line of text in raw mode with backspace support.
fn read_line(prompt: &str) -> io::Result<Option<String>> {
    rprompt!("{}", prompt);
    let mut buf = String::new();
    loop {
        match read()? {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match code {
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => return Ok(None),
                KeyCode::Esc => return Ok(None),
                KeyCode::Enter => {
                    print!("\r\n");
                    io::stdout().flush()?;
                    return Ok(Some(buf));
                }
                KeyCode::Backspace => {
                    if buf.pop().is_some() {
                        print!("\x08 \x08");
                        io::stdout().flush()?;
                    }
                }
                KeyCode::Char(c) => {
                    buf.push(c);
                    print!("{}", c);
                    io::stdout().flush()?;
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn collect_pages(rng: &mut Rng) -> io::Result<(Vec<Vec<ItemSpec>>, usize)> {
    let mut pages: Vec<Vec<ItemSpec>> = vec![];
    let mut current: Vec<ItemSpec> = vec![];
    let mut repeat: usize = 1;

    rln!("=== TDF Interactive Builder ===");

    'outer: loop {
        rln!("");
        rln!(
            "Page {} | {} item(s) on current page",
            pages.len() + 1,
            current.len()
        );
        rprompt!(
            "  [a] Add item   [n] Next page   [c] Count ({}x)   [d] Done > ",
            repeat
        );

        match read_key()? {
            None => break,
            Some('c') => {
                let input = match read_line("  Repeat count: ")? {
                    None => break 'outer,
                    Some(s) => s,
                };
                if let Ok(n) = input.trim().parse::<usize>() {
                    if n >= 1 {
                        repeat = n;
                        rln!("  -> Will repeat {}x", repeat);
                    } else {
                        rln!("  Count must be at least 1");
                    }
                } else {
                    rln!("  Not a valid number");
                }
            }
            Some('a') => {
                rprompt!("  Add [t]ext, [s]hape, or [i]mage > ");
                match read_key()? {
                    None => break 'outer,
                    Some('i') => {
                        rprompt!("  Place at [t]op or [b]ottom > ");
                        let y = loop {
                            match read_key()? {
                                None => break 'outer,
                                Some('t') => break rng.range(0, 550),
                                Some('b') => break rng.range(550, 980),
                                Some(_) => rprompt!("  Try t or b > "),
                            }
                        };
                        let x = rng.range(0, 796);
                        rln!("  -> Added Squid at ({}, {})", x, y);
                        current.push(ItemSpec::Squid { x, y });
                    }
                    Some('t') => {
                        let content = match read_line("  Text: ")? {
                            None => break 'outer,
                            Some(s) => s,
                        };
                        let x = rng.range(0, 825);
                        let y = rng.range(0, 1075);
                        rln!("  -> Added TextBox {:?} at ({}, {})", content, x, y);
                        current.push(ItemSpec::Text { content, x, y });
                    }
                    Some('s') => {
                        rprompt!("  Place at [t]op or [b]ottom > ");
                        let y = loop {
                            match read_key()? {
                                None => break 'outer,
                                Some('t') => break rng.range(0, 550),
                                Some('b') => break rng.range(550, 1020),
                                Some(_) => rprompt!("  Try t or b > "),
                            }
                        };
                        rprompt!("  Shape: [r]ectangle, [c]ircle, [t]riangle > ");
                        let kind = loop {
                            match read_key()? {
                                None => break 'outer,
                                Some('r') => break ShapeKind::Rectangle,
                                Some('c') => break ShapeKind::Circle,
                                Some('t') => break ShapeKind::Triangle,
                                Some(_) => rprompt!("  Try r, c, or t > "),
                            }
                        };
                        let x = rng.range(0, 770);
                        rln!("  -> Added {:?} at ({}, {})", kind, x, y);
                        current.push(ItemSpec::Shape { kind, x, y });
                    }
                    Some(_) => rln!("  Try t or s"),
                }
            }
            Some('n') => {
                let count = current.len();
                pages.push(std::mem::take(&mut current));
                rln!(
                    "  -> Page {} complete ({} item(s)). Starting page {}...",
                    pages.len(),
                    count,
                    pages.len() + 1
                );
            }
            Some('d') => {
                if !current.is_empty() {
                    pages.push(current);
                }
                break;
            }
            Some(_) => rln!("  Try a, n, c, or d"),
        }
    }

    Ok((pages, repeat))
}

fn spec_to_primitive<B: tdf_engine::backend::BackendTypes>(
    spec: ItemSpec,
    squid_ptr: &DataStorePointer<B>,
) -> (ItemPrimitive<B>, ItemUnique) {
    match spec {
        ItemSpec::Text { content, x, y } => (
            ItemPrimitive::TextBox(TextBox {
                content,
                font: None,
            }),
            ItemUnique {
                position: Position { x, y },
                ..Default::default()
            },
        ),
        ItemSpec::Shape { kind, x, y } => (
            ItemPrimitive::Shape(Shape { kind }),
            ItemUnique {
                position: Position { x, y },
                ..Default::default()
            },
        ),
        ItemSpec::Squid { x, y } => (
            ItemPrimitive::Image(Image {
                width: SQUID_W,
                height: SQUID_H,
                data: squid_ptr.clone(),
                alt: None,
            }),
            ItemUnique {
                position: Position { x, y },
                ..Default::default()
            },
        ),
    }
}

fn build_and_write_json(pages: Vec<Vec<ItemSpec>>, output: &str) -> io::Result<()> {
    let mut builder = TDFBuilder::<VecBackend>::new();
    let squid_ptr = builder.stage_data(DataPrimitive::ImageData(ImageData {
        bytes: squid_rgb_bytes(),
    }));
    let mut builder = builder.title("Interactive TDF Document");
    for page in pages {
        builder = builder.add_page(
            page.into_iter()
                .map(|s| spec_to_primitive(s, &squid_ptr))
                .collect(),
        );
    }
    builder
        .build()
        .to_writer(&mut BufWriter::new(File::create(output)?))
}

fn build_and_write_binary(pages: Vec<Vec<ItemSpec>>, output: &str) -> io::Result<()> {
    let mut builder = TDFBuilder::<BinaryBackend>::new();
    let squid_ptr = builder.stage_data(DataPrimitive::ImageData(ImageData {
        bytes: squid_rgb_bytes(),
    }));
    let mut builder = builder.title("Interactive TDF Document");
    for page in pages {
        builder = builder.add_page(
            page.into_iter()
                .map(|s| spec_to_primitive(s, &squid_ptr))
                .collect(),
        );
    }
    builder
        .build()
        .to_writer(&mut BufWriter::new(File::create(output)?))
}

fn build_and_write_pdf(pages: Vec<Vec<ItemSpec>>, output: &str) -> io::Result<()> {
    use krilla::Document;
    use krilla::color::rgb;
    use krilla::geom::{PathBuilder, Point};
    use krilla::num::NormalizedF32;
    use krilla::page::PageSettings;
    use krilla::paint::{Fill, Stroke};
    use krilla::text::{Font, TextDirection};

    let font_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/tdf-renderer/assets/fonts/LiberationSans-Regular.ttf");
    let font_data = std::fs::read(&font_path)?;
    let font = Font::new(font_data.into(), 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to load font"))?;

    let page_w = 850.0_f32;
    let page_h = 1100.0_f32;
    let settings = PageSettings::from_wh(page_w, page_h)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid page dimensions"))?;

    let mut doc = Document::new();

    for page_items in pages {
        let mut page = doc.start_page_with(settings.clone());
        let mut surface = page.surface();

        for item in page_items {
            match item {
                ItemSpec::Text { content, x, y } => {
                    surface.set_fill(Some(Fill {
                        paint: rgb::Color::new(0, 0, 0).into(),
                        opacity: NormalizedF32::ONE,
                        rule: Default::default(),
                    }));
                    surface.draw_text(
                        Point::from_xy(x as f32, y as f32),
                        font.clone(),
                        14.0,
                        &content,
                        false,
                        TextDirection::Auto,
                    );
                }
                ItemSpec::Shape { kind, x, y } => {
                    let (fx, fy, size) = (x as f32, y as f32, 80.0_f32);
                    let mut pb = PathBuilder::new();
                    match kind {
                        ShapeKind::Rectangle => {
                            pb.move_to(fx, fy);
                            pb.line_to(fx + size, fy);
                            pb.line_to(fx + size, fy + size);
                            pb.line_to(fx, fy + size);
                        }
                        ShapeKind::Triangle => {
                            pb.move_to(fx + size / 2.0, fy);
                            pb.line_to(fx + size, fy + size);
                            pb.line_to(fx, fy + size);
                        }
                        ShapeKind::Circle => {
                            // Approximate circle with 4 cubic bezier curves
                            let r = size / 2.0;
                            let cx = fx + r;
                            let cy = fy + r;
                            let k = r * 0.5523;
                            pb.move_to(cx, cy - r);
                            pb.cubic_to(cx + k, cy - r, cx + r, cy - k, cx + r, cy);
                            pb.cubic_to(cx + r, cy + k, cx + k, cy + r, cx, cy + r);
                            pb.cubic_to(cx - k, cy + r, cx - r, cy + k, cx - r, cy);
                            pb.cubic_to(cx - r, cy - k, cx - k, cy - r, cx, cy - r);
                        }
                    }
                    pb.close();
                    let path = pb.finish().unwrap();

                    surface.set_fill(Some(Fill {
                        paint: rgb::Color::new(70, 130, 180).into(),
                        opacity: NormalizedF32::ONE,
                        rule: Default::default(),
                    }));
                    surface.set_stroke(Some(Stroke {
                        paint: rgb::Color::new(30, 80, 130).into(),
                        ..Default::default()
                    }));
                    surface.draw_path(&path);
                }
                ItemSpec::Squid { x, y } => {
                    use image::{ImageBuffer, Rgb};
                    use krilla::image::Image as KrillaImage;

                    let img =
                        ImageBuffer::<Rgb<u8>, _>::from_raw(SQUID_W, SQUID_H, squid_rgb_bytes())
                            .unwrap();
                    let mut png_bytes: Vec<u8> = Vec::new();
                    img.write_to(
                        &mut std::io::Cursor::new(&mut png_bytes),
                        image::ImageFormat::Png,
                    )
                    .unwrap();

                    let scale = 6.0_f32;
                    let size =
                        krilla::geom::Size::from_wh(SQUID_W as f32 * scale, SQUID_H as f32 * scale)
                            .unwrap();
                    surface.push_transform(&krilla::geom::Transform::from_translate(
                        x as f32, y as f32,
                    ));
                    if let Ok(ki) = KrillaImage::from_png(png_bytes.into(), false) {
                        surface.draw_image(ki, size);
                    }
                    surface.pop();
                }
            }
        }

        surface.finish();
        page.finish();
    }

    let pdf = doc
        .finish()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("pdf error: {e:?}")))?;
    std::fs::write(output, &pdf)
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut rng = Rng::new();
    enable_raw_mode()?;
    let result = collect_pages(&mut rng);
    disable_raw_mode()?;

    let (base_pages, repeat) = result?;
    if base_pages.is_empty() {
        println!("No pages — nothing to save.");
        return Ok(());
    }

    let pages: Vec<Vec<ItemSpec>> = base_pages
        .iter()
        .cloned()
        .cycle()
        .take(base_pages.len() * repeat)
        .collect();
    println!(
        "Saving document with {} page(s) ({}x repeat)...",
        pages.len(),
        repeat
    );

    if args.all {
        let raw = args.output.as_deref().unwrap_or("output");
        let base = std::path::Path::new(raw)
            .with_extension("")
            .to_string_lossy()
            .into_owned();
        build_and_write_json(pages.clone(), &format!("{base}.tdfi"))?;
        build_and_write_binary(pages.clone(), &format!("{base}.tdf"))?;
        build_and_write_pdf(pages, &format!("{base}.pdf"))?;
        println!("Written {base}.tdfi, {base}.tdf, {base}.pdf ✓");
    } else {
        let output = args.output.unwrap_or_else(|| match args.backend {
            Backend::Json => "output.tdfi".into(),
            Backend::Binary => "output.tdf".into(),
            Backend::Pdf => "output.pdf".into(),
        });
        match args.backend {
            Backend::Json => build_and_write_json(pages, &output)?,
            Backend::Binary => build_and_write_binary(pages, &output)?,
            Backend::Pdf => build_and_write_pdf(pages, &output)?,
        }
        println!("Written to {output} ✓");
    }
    Ok(())
}
