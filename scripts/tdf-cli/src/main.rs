use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use tdf_engine::backend::{Backend, CacheHints, VecBackend};
use tdf_engine::builder::TDFBuilder;
use tdf_engine::impls::binary::backend::BinaryBackend;
use tdf_engine::impls::document::{BackedDocument, TdfDocument};
use tdf_engine::impls::{DocumentWrite, ManifestRead, TDFManifest, TdfDocumentExt};
use tdf_engine::primitives::data::{DataPrimitive, ImageData};
use tdf_engine::primitives::item::{
    Image, ItemPrimitive, ItemUnique, Position, Shape, ShapeKind, TextBox,
};

// 9x20 squid pixel art via claude
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
                    [148u8, 103u8, 189u8] // purple
                } else {
                    [32u8, 178u8, 170u8] // teal background
                }
            })
        })
        .collect()
}

#[derive(ValueEnum, Clone, Default)]
enum Format {
    #[default]
    Json,
    Binary,
}

#[derive(Parser)]
#[command(name = "tdf-cli", about = "Read and write TDF documents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Write a hardcoded TDF document to a file
    Write {
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Json)]
        format: Format,
    },
    /// Read a TDF document from a file and print its contents
    Read {
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Json)]
        format: Format,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Write { path, format } => cmd_write(&path, &format),
        Command::Read { path, format } => cmd_read(&path, &format),
    }
}

fn cmd_write(path: &std::path::Path, format: &Format) {
    match format {
        Format::Json => {
            let doc = build_sample_doc(TDFBuilder::<VecBackend>::new());
            write_doc(path, &doc, "JSON");
        }
        Format::Binary => {
            let doc = build_sample_doc(TDFBuilder::<BinaryBackend>::new());
            write_doc(path, &doc, "binary");
        }
    }
}

fn build_sample_doc<B: Backend + Default>(mut builder: TDFBuilder<B>) -> BackedDocument<B>
where
    BackedDocument<B>: DocumentWrite,
{
    let squid_ptr = builder.stage_data(DataPrimitive::ImageData(ImageData {
        bytes: squid_rgb_bytes(),
    }));

    builder
        .title("Sample Document")
        .add_page(vec![
            (
                ItemPrimitive::Shape(Shape {
                    kind: ShapeKind::Circle,
                }),
                ItemUnique {
                    position: Position { x: 10, y: 20 },
                    ..Default::default()
                },
            ),
            (
                ItemPrimitive::TextBox(TextBox {
                    content: "Hello, TDF!".into(),
                    font: None,
                }),
                ItemUnique {
                    position: Position { x: 50, y: 60 },
                    ..Default::default()
                },
            ),
        ])
        .add_page(vec![
            (
                ItemPrimitive::TextBox(TextBox {
                    content: "Page two content".into(),
                    font: None,
                }),
                ItemUnique {
                    position: Position { x: 0, y: 0 },
                    ..Default::default()
                },
            ),
            (
                ItemPrimitive::Image(Image {
                    width: SQUID_W,
                    height: SQUID_H,
                    data: squid_ptr,
                }),
                ItemUnique {
                    position: Position { x: 700, y: 0 },
                    ..Default::default()
                },
            ),
        ])
        .build()
}

fn write_doc(path: &std::path::Path, doc: &impl DocumentWrite, label: &str) {
    let file = File::create(path).expect("failed to create file");
    let mut writer = BufWriter::new(file);
    doc.to_writer(&mut writer).expect("failed to write TDF");
    println!("wrote {} TDF to {}", label, path.display());
}

fn cmd_read(path: &std::path::Path, format: &Format) {
    match format {
        Format::Json => cmd_read_json(path),
        Format::Binary => cmd_read_binary(path),
    }
}

fn cmd_read_json(path: &std::path::Path) {
    let mut file = File::open(path).expect("failed to open file");
    let manifest = TDFManifest::<tdf_engine::impls::vec::backend::VecTypes>::from_reader(&mut file)
        .expect("failed to read manifest");

    println!("title: {:?}", manifest.meta.document_title);
    println!("pages: {}", manifest.pages.page_count());

    let mut doc = manifest
        .load_backend::<VecBackend, _>(file)
        .expect("failed to load backend");

    print_doc_items(&mut doc);
}

fn cmd_read_binary(path: &std::path::Path) {
    let mut file = File::open(path).expect("failed to open file");
    let mut doc =
        BackedDocument::<tdf_engine::impls::binary::backend::BinaryBackend>::from_binary_reader(
            &mut file,
        )
        .expect("failed to read binary TDF");

    println!("title: {:?}", doc.manifest().meta.document_title);
    println!("pages: {}", doc.manifest().pages.page_count());

    print_doc_items(&mut doc);
}

fn print_doc_items<D: TdfDocumentExt>(doc: &mut D) {
    let page_count = doc.manifest().pages.page_count();
    for page_num in 0..page_count {
        println!("\n--- page {} ---", page_num);
        for (item, unique) in doc.iter_page_items(page_num, CacheHints::Cache) {
            println!("  position: ({}, {})", unique.position.x, unique.position.y);
            match item {
                ItemPrimitive::TextBox(t) => println!("  text: {:?}", t.content),
                ItemPrimitive::Shape(s) => println!("  shape: {:?}", s.kind),
                ItemPrimitive::Vector(v) => println!("  vector: {} points", v.points.len()),
                ItemPrimitive::Image(i) => {
                    println!("  image: {}x{}", i.width, i.height);
                    match doc.fetch_data_item(&i.data) {
                        Some(DataPrimitive::ImageData(data)) => {
                            let w = i.width as usize;
                            let rows: Vec<&[u8]> = data.bytes.chunks(w * 3).collect();
                            for pair in rows.chunks(2) {
                                let top = pair[0];
                                let bot = if pair.len() > 1 { pair[1] } else { top };
                                print!("  ");
                                for x in 0..w {
                                    let t = &top[x * 3..x * 3 + 3];
                                    let b = &bot[x * 3..x * 3 + 3];
                                    print!(
                                        "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m▄\x1b[0m",
                                        t[0], t[1], t[2], b[0], b[1], b[2]
                                    );
                                }
                                println!();
                            }
                        }
                        Some(other) => println!("  (unexpected data kind: {:?})", other),
                        None => println!("  (data not found)"),
                    }
                }
            }
        }
    }
}
