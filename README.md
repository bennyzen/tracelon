# Tracelon

A desktop app for tracing AI-generated logos to clean SVG with optimized curves.

Load a raster image, select an area, trace it to SVG, and interactively tune curve smoothness with live preview. The key feature is post-trace curve simplification that eliminates the jaggedness typical of bitmap-to-vector conversion.

## Features

- **Three trace modes**: Monochrome (Otsu-thresholded), multi-color (stacked layers), and outline (stroke-only)
- **Curve simplification**: Smoothness slider powered by kurbo's Bezier simplification — works directly on curves, no lossy polyline conversion
- **Live preview**: Side-by-side source image and SVG output with overlay comparison
- **Rectangle selection**: Crop to the area you want to trace
- **Drag-and-drop**: Drop images directly onto the window
- **SVG export**: Clean output with only `<path>` elements, ready for Inkscape or Figma

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 |
| Frontend | Nuxt 4 (SPA) + Nuxt UI |
| Bitmap tracing | vtracer (Spline mode) |
| Curve simplification | kurbo `simplify_bezpath` |
| Image processing | image + imageproc (Otsu threshold) |
| Color quantization | vtracer built-in (stacked layers) |

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
6. **Export SVG** when satisfied

## License

MIT
