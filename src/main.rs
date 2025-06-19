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

        // Handle transparency by compositing onto terminal background color
        let rgba_image = resized.to_rgba8();
        let mut rgb_image = image::ImageBuffer::new(rgba_image.width(), rgba_image.height());
        let background_color = image::Rgb([0u8, 0u8, 0u8]); // Black background

        for (x, y, pixel) in rgba_image.enumerate_pixels() {
            let rgba = pixel.0;
            let alpha = rgba[3] as f32 / 255.0;

            if alpha == 0.0 {
                // Fully transparent - use background color
                rgb_image.put_pixel(x, y, background_color);
            } else if alpha == 1.0 {
                // Fully opaque
                rgb_image.put_pixel(x, y, image::Rgb([rgba[0], rgba[1], rgba[2]]));
            } else {
                // Semi-transparent - blend with background
                let blended_r = ((1.0 - alpha) * background_color[0] as f32 + alpha * rgba[0] as f32) as u8;
                let blended_g = ((1.0 - alpha) * background_color[1] as f32 + alpha * rgba[1] as f32) as u8;
                let blended_b = ((1.0 - alpha) * background_color[2] as f32 + alpha * rgba[2] as f32) as u8;
                rgb_image.put_pixel(x, y, image::Rgb([blended_r, blended_g, blended_b]));
            }
        }

        let sixel_data = sixel_bytes::sixel_string(
            rgb_image.as_raw(),
            rgb_image.width() as i32,
            rgb_image.height() as i32,
            sixel_bytes::PixelFormat::RGB888,
        ).map_err(|e| anyhow::anyhow!("Failed to encode image: {:?}", e))?;

        print!("{}", sixel_data);

        if num_files > 1 {
            println!("{}", file.display());
        }
    }

    Ok(())
}
