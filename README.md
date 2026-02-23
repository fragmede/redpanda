# redpanda

Display images and text files in the terminal using the [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/).

Works with terminals that support kitty graphics, including [Ghostty](https://ghostty.org/) and [Kitty](https://sw.kovidgoyal.net/kitty/).

## Usage

```bash
redpanda image.png
redpanda photo.jpg diagram.svg
redpanda notes.txt          # text files are printed directly
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--max-width` | Maximum image width in pixels | 800 |
| `--max-height` | Maximum image height in pixels | 480 |

## Building

```bash
cargo build --release
```

The binary will be at `target/release/redpanda`.

## Features

- Displays PNG, JPEG, GIF, BMP, SVG, and other formats supported by the `image` crate
- Automatically resizes large images to fit within max dimensions while preserving aspect ratio
- Transparency is handled natively via PNG encoding
- Text files are detected and printed directly to stdout
- Multiple files can be displayed in sequence
