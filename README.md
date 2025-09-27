# Image Viewer

A full-screen image viewer built with Tauri that allows users to view images in a distraction-free environment. The application supports common image formats and provides EXIF information viewing capabilities.

## Features

- Full-screen viewing without window borders
- Support for JPG, JPEG, PNG, BMP, HEIC, and GIF formats
- Image zoom with mouse wheel
- Image pan with mouse drag
- EXIF information display
- macOS-style close button and info button
- System tray integration (on macOS, the app stays in dock when closed)

## Installation

To run this application, you need to have Rust and Node.js installed on your system.

1. Install dependencies:
```bash
cargo install tauri-cli
```

2. Run the application in development mode:
```bash
cd src-tauri
cargo tauri dev
```

## Usage

- When launched without an image, the application shows an open dialog
- Drag the image to pan around
- Use mouse wheel to zoom in/out
- Move mouse to top of screen to show control buttons
- Click the "i" button to view EXIF information
- Use Command+Q (or Ctrl+Q) to quit completely

## Project Structure

- `src/index.html` - Main HTML structure
- `src/index.js` - Frontend JavaScript logic
- `src-tauri/src/main.rs` - Backend Rust logic
- `src-tauri/tauri.conf.json` - Tauri configuration
- `src-tauri/Cargo.toml` - Rust dependencies