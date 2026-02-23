# redpanda

My version of cat(1)!

Does the usual cat thing of displaying text, but if it's an image file, then it'll use the Kitty graphics protocol, supported by Ghostty and other modern terminals, to display the jpg/png/gif to the screen.

[Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/).

## Building

```bash
cargo build --release
```
The binary will be at `target/release/redpanda`.

## Installing

### Homebrew (macOS)

```bash
brew install --formula https://raw.githubusercontent.com/fragmede/redpanda/main/Formula/rp.rb
```

Or as a tap:
```bash
brew tap fragmede/redpanda https://github.com/fragmede/redpanda
brew install fragmede/redpanda/rp
```

### Manual

```bash
cp target/release/redpanda ~/bin/rp
```

Then add to your `.bashrc` / `.zshrc`:
```bash
alias cat=~/bin/rp
```

## Usage

```bash
rp image.png
rp photo.jpg diagram.svg
rp notes.txt # text files are printed directly
```

### Options

Supports all standard `cat(1)` flags:

| Flag | Description |
|------|-------------|
| `-b` | Number non-blank output lines |
| `-e` | Display non-printing characters and `$` at end of each line |
| `-l` | Set an exclusive advisory lock on stdout |
| `-n` | Number all output lines |
| `-s` | Squeeze multiple adjacent empty lines |
| `-t` | Display non-printing characters and tabs as `^I` |
| `-u` | Disable output buffering |
| `-v` | Display non-printing characters visibly |

Plus image-specific options:

| Flag | Description | Default |
|------|-------------|---------|
| `--max-width` | Maximum image width in pixels | 800 |
| `--max-height` | Maximum image height in pixels | 480 |

## Features

- Displays PNG, JPEG, GIF, BMP, SVG, and other formats supported by the `image` crate
- Automatically resizes large images to fit within max dimensions while preserving aspect ratio
- Transparency is handled natively via PNG encoding
- Text files are detected and printed directly to stdout
- Multiple files can be displayed in sequence
