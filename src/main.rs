use anyhow::{Context, Result};
use clap::Parser;
use image::io::Reader as ImageReader;
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

        // Convert to RGB
        let rgb = resized.to_rgb8();
        
        // Start sixel output
        print!("\x1BP");
        
        // TODO: Add actual sixel conversion and output
        // This is where we'd convert the RGB data to sixel format
        // For now this is a placeholder that needs implementation
        
        // End sixel output
        print!("\x1B\\");
        
        if args.files.len() > 1 {
            println!("{}", file.display());
        }
    }

    Ok(())
}
