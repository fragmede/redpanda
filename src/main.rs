use anyhow::{Context, Result};
use clap::Parser;
use image::io::Reader as ImageReader;
use sixel_bytes;
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

        // Convert to sixel format and print
        let rgba_image = resized.into_rgba8();
        let sixel_data = sixel_bytes::sixel_string(
            rgba_image.as_raw(),
            rgba_image.width() as i32,
            rgba_image.height() as i32,
            sixel_bytes::PixelFormat::RGBA8888,
        ).map_err(|e| anyhow::anyhow!("Failed to encode image: {:?}", e))?;
        
        print!("{}", sixel_data);
        
        if num_files > 1 {
            println!("{}", file.display());
        }
    }

    Ok(())
}
