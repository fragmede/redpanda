use anyhow::{Context, Result};
use base64::Engine;
use clap::Parser;
use image::io::Reader as ImageReader;
use std::fs;
use std::io::{self, BufRead, Cursor, Write};
use std::path::PathBuf;

/// Concatenate and print files, with image display via kitty graphics protocol
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Number the non-blank output lines, starting at 1
    #[arg(short = 'b')]
    number_nonblank: bool,

    /// Display non-printing characters (see -v), and display a dollar sign at the end of each line
    #[arg(short = 'e')]
    show_ends: bool,

    /// Set an exclusive advisory lock on the standard output file descriptor
    #[arg(short = 'l')]
    lock: bool,

    /// Number the output lines, starting at 1
    #[arg(short = 'n')]
    number: bool,

    /// Squeeze multiple adjacent empty lines, causing the output to be single spaced
    #[arg(short = 's')]
    squeeze_blank: bool,

    /// Display non-printing characters (see -v), and display tab characters as ^I
    #[arg(short = 't')]
    show_tabs: bool,

    /// Disable output buffering
    #[arg(short = 'u')]
    unbuffered: bool,

    /// Display non-printing characters so they are visible
    #[arg(short = 'v')]
    show_nonprinting: bool,

    /// Maximum image width in pixels
    #[arg(long, default_value = "800")]
    max_width: u32,

    /// Maximum image height in pixels
    #[arg(long, default_value = "480")]
    max_height: u32,

    /// Files to concatenate and print (default: stdin, use '-' for stdin)
    files: Vec<String>,
}

fn is_image_extension(path: &PathBuf) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref(),
        Some(
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "tiff" | "tif" | "webp" | "pnm"
                | "pbm" | "pgm" | "ppm" | "tga" | "qoi" | "avif"
        )
    )
}

fn format_nonprinting(ch: u8, show_tabs: bool, show_nonprinting: bool) -> Vec<u8> {
    if ch == b'\n' {
        return vec![ch];
    }
    if ch == b'\t' && !show_tabs {
        return vec![ch];
    }
    if !show_nonprinting && !show_tabs {
        return vec![ch];
    }
    if ch == b'\t' {
        return vec![b'^', b'I'];
    }
    if !show_nonprinting {
        return vec![ch];
    }
    if ch < 0x20 {
        vec![b'^', ch + 0x40]
    } else if ch == 0x7f {
        vec![b'^', b'?']
    } else if ch >= 0x80 {
        let low = ch & 0x7f;
        if low < 0x20 {
            vec![b'M', b'-', b'^', low + 0x40]
        } else if low == 0x7f {
            vec![b'M', b'-', b'^', b'?']
        } else {
            vec![b'M', b'-', low]
        }
    } else {
        vec![ch]
    }
}

struct CatOptions {
    number: bool,
    number_nonblank: bool,
    squeeze_blank: bool,
    show_ends: bool,
    show_tabs: bool,
    show_nonprinting: bool,
}

struct CatState {
    line_num: usize,
    prev_blank: bool,
}

fn process_text<R: BufRead>(
    mut reader: R,
    out: &mut impl Write,
    opts: &CatOptions,
    state: &mut CatState,
) -> Result<()> {
    let show_nonprinting = opts.show_nonprinting || opts.show_ends || opts.show_tabs;
    let mut buf = Vec::new();

    loop {
        buf.clear();
        let bytes_read = reader.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }

        let has_newline = buf.last() == Some(&b'\n');
        if has_newline {
            buf.pop();
        }

        let is_blank = buf.is_empty();

        if opts.squeeze_blank && is_blank && state.prev_blank {
            continue;
        }
        state.prev_blank = is_blank;

        if opts.number_nonblank {
            if !is_blank {
                state.line_num += 1;
                write!(out, "{:6}\t", state.line_num)?;
            }
        } else if opts.number {
            state.line_num += 1;
            write!(out, "{:6}\t", state.line_num)?;
        }

        for &ch in &buf {
            out.write_all(&format_nonprinting(ch, opts.show_tabs, show_nonprinting))?;
        }

        if opts.show_ends && has_newline {
            out.write_all(b"$")?;
        }

        if has_newline {
            out.write_all(b"\n")?;
        }
    }

    Ok(())
}

fn write_kitty_image(out: &mut impl Write, img: &image::DynamicImage) -> Result<()> {
    let mut png_data = Vec::new();
    img.write_to(
        &mut Cursor::new(&mut png_data),
        image::ImageOutputFormat::Png,
    )
    .context("Failed to encode image as PNG")?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);

    const CHUNK_SIZE: usize = 4096;
    let chunks: Vec<&str> = b64
        .as_bytes()
        .chunks(CHUNK_SIZE)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect();

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        if i == 0 {
            write!(
                out,
                "\x1b_Ga=T,f=100,t=d,m={};{}\x1b\\",
                if is_last { 0 } else { 1 },
                chunk
            )?;
        } else {
            write!(
                out,
                "\x1b_Gm={};{}\x1b\\",
                if is_last { 0 } else { 1 },
                chunk
            )?;
        }
    }

    writeln!(out)?;
    out.flush()?;
    Ok(())
}

fn display_image(
    out: &mut impl Write,
    path: &PathBuf,
    max_width: u32,
    max_height: u32,
) -> Result<()> {
    let img = ImageReader::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?
        .decode()
        .with_context(|| format!("Failed to decode {}", path.display()))?;

    let resized = if img.width() > max_width || img.height() > max_height {
        let ratio = f32::min(
            max_width as f32 / img.width() as f32,
            max_height as f32 / img.height() as f32,
        );
        let new_width = (img.width() as f32 * ratio) as u32;
        let new_height = (img.height() as f32 * ratio) as u32;
        img.resize(
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        img
    };

    write_kitty_image(out, &resized)
}

#[cfg(unix)]
fn lock_stdout() -> Result<()> {
    use std::os::unix::io::AsRawFd;
    let fd = io::stdout().as_raw_fd();
    const LOCK_EX: i32 = 2;
    extern "C" {
        fn flock(fd: i32, operation: i32) -> i32;
    }
    let ret = unsafe { flock(fd, LOCK_EX) };
    if ret != 0 {
        return Err(anyhow::anyhow!(
            "Failed to lock stdout: {}",
            io::Error::last_os_error()
        ));
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    #[cfg(unix)]
    if args.lock {
        lock_stdout()?;
    }

    let files: Vec<String> = if args.files.is_empty() {
        vec!["-".to_string()]
    } else {
        args.files
    };

    let opts = CatOptions {
        number: args.number,
        number_nonblank: args.number_nonblank,
        squeeze_blank: args.squeeze_blank,
        show_ends: args.show_ends,
        show_tabs: args.show_tabs,
        show_nonprinting: args.show_nonprinting,
    };

    let mut state = CatState {
        line_num: 0,
        prev_blank: false,
    };

    let stdout = io::stdout();
    let mut out = stdout.lock();

    for name in &files {
        if name == "-" {
            let stdin = io::stdin();
            let reader = stdin.lock();
            process_text(reader, &mut out, &opts, &mut state)?;
        } else {
            let path = PathBuf::from(name);
            if is_image_extension(&path) {
                display_image(&mut out, &path, args.max_width, args.max_height)?;
            } else {
                let file = fs::File::open(&path)
                    .with_context(|| format!("{}: No such file or directory", name))?;
                let reader = io::BufReader::new(file);
                process_text(reader, &mut out, &opts, &mut state)?;
            }
        }
    }

    Ok(())
}
