use anyhow::{Context, Result};
use clap::Parser;
use image::io::Reader as ImageReader;
use sixel::{encoder::{self, QuickFrame}, optflags};
use std::path::PathBuf;

/// Display images in terminal using sixel graphics
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Image files to display
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Force specific number of colors
    #[arg(short = 'n', long = "num-colors")]
    num_colors: Option<u32>,

    /// Maximum image width
    #[arg(long, default_value = "800")]
    max_width: u32,

    /// Maximum image height
    #[arg(long, default_value = "480")] 
    max_height: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let num_files = args.files.len();
    for file in args.files {
        let img = ImageReader::open(&file)
            .with_context(|| format!("Failed to open {}", file.display()))?
            .decode()
            .with_context(|| format!("Failed to decode {}", file.display()))?;

        // Resize image if needed while maintaining aspect ratio
        let resized = if img.width() > args.max_width || img.height() > args.max_height {
            let ratio = f32::min(
                args.max_width as f32 / img.width() as f32,
                args.max_height as f32 / img.height() as f32
            );
            let new_width = (img.width() as f32 * ratio) as u32;
            let new_height = (img.height() as f32 * ratio) as u32;
            img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };

        // Convert to RGB and encode as sixel
        let rgb = resized.to_rgb8();
        let width = rgb.width() as i32;
        let height = rgb.height() as i32;
        
        let mut enc = encoder::Encoder::new()
            .map_err(|e| anyhow::anyhow!("Failed to create encoder: {:?}", e))?;
        
        if let Some(colors) = args.num_colors {
            enc.set_color_option(optflags::ColorOption::builtin_palette(
                optflags::BuiltinPalette::XTerm256
            )).map_err(|e| anyhow::anyhow!("Failed to set color limit: {:?}", e))?;
        }

        let frame = QuickFrame {
            pixels: rgb.as_raw().to_vec(),
            width: width as usize,
            height: height as usize,
        };

        enc.encode_bytes(frame)
            .map_err(|e| anyhow::anyhow!("Failed to encode image: {:?}", e))?;
        
        if num_files > 1 {
            println!("{}", file.display());
        }
    }

    Ok(())
}
