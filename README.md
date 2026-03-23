<p align="center">
  <img src="tracelon.png" alt="Tracelon" width="128" />
</p>

<h1 align="center">Tracelon</h1>

<p align="center">
  A desktop app for tracing raster images to clean SVG with optimized curves.<br/>
  Built with Tauri v2, Nuxt 4, and Rust.
</p>

<p align="center">
  <a href="https://github.com/bennyzen/tracelon/actions"><img src="https://github.com/bennyzen/tracelon/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/bennyzen/tracelon/releases"><img src="https://img.shields.io/github/v/release/bennyzen/tracelon?label=download" alt="Release" /></a>
  <img src="https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-blue" alt="Platforms" />
</p>

---

Load an image, select an area, trace it to SVG, and interactively tune curve smoothness with live preview. The key feature is post-trace curve simplification that eliminates the jaggedness typical of bitmap-to-vector conversion.

## Features

### Tracing

- **Three trace modes**: Monochrome (Otsu-thresholded), multi-color (stacked/cutout layers), and outline (stroke-only)
- **Curve simplification**: Smoothness slider powered by kurbo's Bezier simplification — works directly on curves, no lossy polyline conversion
- **Line snapping**: Automatically straightens near-flat cubic segments into clean lines
- **Color mode controls**: Adjustable color count (2–32), stacked vs cutout layering, speckle filtering, and color precision

### Selection

- **Rectangle selection**: Draw a selection to trace a specific region
- **CTRL+drag**: Constrains selection to a square
- **Resize handles**: Drag edges or corners to adjust the selection
- **Move selection**: Drag from inside the selection to reposition it

### Preview & Export

- **Three-pane view**: Source image, SVG preview, and SVGO-optimized output side by side
- **Source overlay**: Toggle a semi-transparent overlay of the original image on the SVG preview for comparison (selection-aware cropping)
- **Zoom & pan**: Scroll to zoom, drag to pan in both preview panes
- **SVGO optimization**: Automatic SVG optimization with configurable plugins (path rounding, group collapsing, transform simplification)
- **Live re-optimization**: Changes to trace mode or smoothness automatically re-optimize when the export view is visible
- **Drag-and-drop**: Drop images directly onto the window

### Pipeline Tuning

- **Smoothness slider**: 0% = raw vtracer output, higher = fewer curve segments
- **Line snap**: Control how aggressively near-straight segments are simplified
- **Speckle filter**: Remove small noise patches (color mode)
- **Color precision**: Control color grouping sensitivity (color mode)

## Installation

Pre-built binaries are available on the [Releases page](https://github.com/bennyzen/tracelon/releases).

### macOS

Download the `.dmg` for your architecture (ARM64 for M1+, x64 for Intel), open it, and drag Tracelon to Applications.

### Windows

Download and run the `.msi` installer.

### Linux (Debian/Ubuntu)

```bash
# Download the .deb from the releases page, then:
sudo dpkg -i Tracelon_*.deb
```

### Arch Linux

A pacman package is included in each release:

```bash
# Download the .pkg.tar.zst from the releases page, then:
sudo pacman -U tracelon-*.pkg.tar.zst
```

Or build from source using the included PKGBUILD:

```bash
git clone https://github.com/bennyzen/tracelon.git
cd tracelon
makepkg -si
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 |
| Frontend | Nuxt 4 (SPA) + Nuxt UI |
| Bitmap tracing | vtracer (Spline mode) |
| Curve simplification | kurbo `simplify_bezpath` |
| SVG optimization | SVGO (browser-side) |
| Image processing | image + imageproc (Otsu threshold) |
| Color quantization | NeuQuant pre-quantization + vtracer stacked/cutout layers |
| CI/CD | GitHub Actions (cross-platform builds) |

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- System dependencies for Tauri v2 ([see docs](https://v2.tauri.app/start/prerequisites/))

### Setup

```bash
git clone https://github.com/bennyzen/tracelon.git
cd tracelon
pnpm install
```

### Run

```bash
pnpm tauri dev
```

First build compiles ~600 Rust crates and takes a few minutes. Subsequent builds are incremental.

### Test

```bash
cd src-tauri
cargo test
```

### Build for production

```bash
pnpm tauri build
```

## Usage

1. **Open** an image (button or drag-and-drop) — PNG, JPG, WebP
2. **Select** the area to trace (draw a rectangle, or skip to trace the full image)
3. Pick a **mode** — Mono, Color, or Outline
4. Click **Trace**
5. Adjust the **smoothness slider** — 0% = raw vtracer output, higher = fewer curve segments
6. Click **Optimize** to see the SVGO-optimized output with size savings
7. **Export SVG** when satisfied

## License

MIT
