use anyhow::{Context, Result};
use base64::Engine;
use clap::Parser;
use image::io::Reader as ImageReader;
use std::fs;
use std::io::{self, Cursor, Write};
use std::path::PathBuf;

/// Display images in terminal using kitty graphics protocol
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Image files to display
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Maximum image width
    #[arg(long, default_value = "800")]
    max_width: u32,

    /// Maximum image height
    #[arg(long, default_value = "480")]
    max_height: u32,
}

fn is_text_file(path: &PathBuf) -> Result<bool> {
    let content = fs::read(path)?;
    Ok(!content.contains(&0))
}

/// Write an image to stdout using the kitty graphics protocol.
/// The image is PNG-encoded, base64'd, and sent in chunks.
fn write_kitty_image(img: &image::DynamicImage) -> Result<()> {
    let mut png_data = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_data), image::ImageOutputFormat::Png)
        .context("Failed to encode image as PNG")?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    const CHUNK_SIZE: usize = 4096;
    let chunks: Vec<&str> = b64.as_bytes().chunks(CHUNK_SIZE).map(|c| {
        std::str::from_utf8(c).unwrap()
    }).collect();

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        if i == 0 {
            // First chunk: include format and action params
            // a=T: transmit and display, f=100: PNG, t=d: direct data
            write!(out, "\x1b_Ga=T,f=100,t=d,m={};{}\x1b\\",
                if is_last { 0 } else { 1 },
                chunk)?;
        } else {
            // Continuation chunk
            write!(out, "\x1b_Gm={};{}\x1b\\",
                if is_last { 0 } else { 1 },
                chunk)?;
        }
    }

    out.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let num_files = args.files.len();
    for file in args.files {
        if is_text_file(&file).unwrap_or(false) {
            let content = fs::read_to_string(&file)
                .with_context(|| format!("Failed to read text file {}", file.display()))?;
            print!("{}", content);

            if num_files > 1 {
                println!("{}", file.display());
            }
            continue;
        }

        let img = ImageReader::open(&file)
            .with_context(|| format!("Failed to open {}", file.display()))?
            .decode()
            .with_context(|| format!("Failed to decode {}", file.display()))?;

        let resized = if img.width() > args.max_width || img.height() > args.max_height {
            let ratio = f32::min(
                args.max_width as f32 / img.width() as f32,
                args.max_height as f32 / img.height() as f32,
            );
            let new_width = (img.width() as f32 * ratio) as u32;
            let new_height = (img.height() as f32 * ratio) as u32;
            img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };

        write_kitty_image(&resized)?;

        if num_files > 1 {
            println!("{}", file.display());
        }
    }

    Ok(())
}
