use anyhow::{Context, Result};
use clap::Parser;
use image::io::Reader as ImageReader;
use sixel::{encoder, optflags, status};
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
        
        let mut enc = encoder::Encoder::new();
        enc.set_opt(optflags::OPTFLAGS_8BITMODE, 1)?;
        if let Some(colors) = args.num_colors {
            enc.set_opt(optflags::OPTFLAGS_COLORS, colors as i32)?;
        }

        let output = enc.encode(
            rgb.as_raw().as_slice(),
            width,
            height,
            8,
            None,
        ).map_err(|e| anyhow::anyhow!("Sixel encoding failed: {:?}", e))?;

        if output.status != status::STATUS_OK {
            return Err(anyhow::anyhow!("Sixel encoding failed with status: {:?}", output.status));
        }

        print!("{}", String::from_utf8_lossy(&output.data));
        
        if num_files > 1 {
            println!("{}", file.display());
        }
    }

    Ok(())
}
