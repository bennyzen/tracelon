# Tracelon — Design Spec

A standalone Tauri desktop app for tracing AI-generated logos to clean SVG with optimized curves.

## Problem

AI image generators produce raster logos (PNG/JPG). Converting them to SVG with tools like potrace yields jagged, imperfect paths that faithfully reproduce pixel artifacts. There is no simple tool that traces **and** smooths the result into clean, professional vector output.

## Solution

Tracelon provides a focused workflow: load image, select area, trace to SVG, and interactively tune curve smoothness with live preview. The key differentiator is the post-trace curve simplification pipeline that eliminates jaggedness and produces optimized Bezier curves.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 |
| Frontend | Nuxt 4 (SPA mode, `ssr: false`) + Nuxt UI (latest) |
| Frontend libs | `@tauri-apps/api` for IPC, HTML Canvas for image display/selection, inline SVG for preview |
| Backend | Rust |
| Image decoding | `image` crate (PNG, JPG, WebP) |
| Bitmap tracing | `potrace` crate (Rust bindings) |
| Curve math | `kurbo` crate (Bezier operations, simplification) |
| Color quantization | `color_quant` or `quantizr` (for multi-color mode) |
| SVG output | `svg` crate |

## Processing Pipeline

```
Load Image → Crop to Selection → Trace (potrace) → Curve Simplify → Export SVG
```

### Step 1: Load Image
- Accept PNG, JPG, WebP via file picker or drag-and-drop
- Display on left canvas panel
- Tauri `dialog::open` for native file picker

### Step 2: Crop Selection
- Rectangle selection tool on the source image canvas
- Draggable corner handles for resizing
- Defaults to full image (no selection = trace everything)
- Selection coordinates passed to Rust backend

### Step 3: Trace (potrace)
- Crop the selected region from the source image
- Apply mode-specific preprocessing:
  - **Monochrome**: threshold to 1-bit bitmap, trace with potrace
  - **Multi-color**: quantize to N colors (user-configurable, default 6), isolate each color layer as a 1-bit bitmap, trace each separately, stack results
  - **Outlines**: edge-detect (e.g., Canny or simple threshold + contour), trace edges as stroked paths (no fill)
- Output: raw SVG path data (potentially jagged)

### Step 4: Curve Simplify (key step)
The core value of Tracelon. After potrace produces raw Bezier paths:

1. **Flatten** — convert Bezier curves to high-resolution polylines
2. **Douglas-Peucker simplify** — reduce point count while preserving shape, controlled by tolerance parameter
3. **Cubic Bezier re-fit** — fit smooth cubic Bezier curves through the simplified point sequence
4. **Corner detection** — identify sharp angle changes and preserve them as hard corners; smooth everything else

The **smoothness slider** (0-100%) maps to the tolerance values in steps 2-3:
- Low (0-20%): nearly faithful to raw trace, minimal smoothing
- Medium (40-60%): balanced — removes jaggedness, preserves detail
- High (80-100%): aggressive simplification, very smooth curves, may lose small details

All processing happens in Rust via `kurbo` for performance. Results are sent to the frontend as SVG path strings via Tauri IPC.

### Step 5: Export SVG
- Save clean SVG file via native save dialog
- SVG contains only `<path>` elements (and `<svg>` root with viewBox)
- Multi-color mode: each color layer is a separate path with fill attribute
- Outline mode: paths have stroke, no fill

## UI Layout

### Window Structure
```
┌─────────────────────────────────────────────────────┐
│ Toolbar: [Open] | Mode: [Mono|Color|Outline] |      │
│          Smoothness: [────●────] 60% | [Export SVG]  │
├──────────────────────────┬──────────────────────────┤
│                          │                          │
│     Source Image         │      SVG Preview         │
│     (HTML Canvas)        │      (Inline SVG)        │
│                          │                          │
│     ┌─ ─ ─ ─ ─ ─┐       │                          │
│     │  selection │       │                          │
│     └─ ─ ─ ─ ─ ─┘       │                          │
│                          │                          │
├──────────────────────────┼──────────────────────────┤
│ file.png 1024×1024       │ 42 paths · 3.2 KB       │
│ Selection: 964×964       │ ☑ Overlay  ☐ Ctrl pts   │
└──────────────────────────┴──────────────────────────┘
```

### Toolbar
- **Open Image** button — triggers native file picker
- **Mode toggle** — three buttons: Mono, Color, Outline (segmented control via Nuxt UI)
- **Smoothness slider** — Nuxt UI range slider, 0-100%, with numeric readout
- **Export SVG** button — triggers native save dialog
- Multi-color mode: additional "Colors" number input (2-16, default 6)

### Left Panel: Source Image
- HTML `<canvas>` element displaying the loaded image
- Rectangle selection overlay drawn on canvas
- Drag to draw, drag corners to resize
- Zoom/pan via scroll wheel and drag (stretch goal)
- Status bar: filename, dimensions, selection size

### Right Panel: SVG Preview
- Inline `<svg>` element rendering the traced output
- Updates live when smoothness slider changes (debounced ~100ms)
- Toggle options in status bar:
  - **Show original overlay**: semi-transparent source image on top of SVG for fidelity comparison
  - **Show control points**: render Bezier handles for debugging curve quality
- Status bar: path count, estimated SVG file size

## Data Flow (IPC)

```
Frontend                          Rust Backend
────────                          ────────────
Load image path ──invoke──→       Read & decode image
                                  Store in memory
                ←─result──        Image dimensions + thumbnail

Selection rect  ──invoke──→       Crop region
+ mode + smooth                   Trace (potrace)
                                  Simplify curves
                ←─result──        SVG path data (string)

Export request  ──invoke──→       Write SVG file to disk
+ output path
                ←─result──        Success/failure
```

### Tauri Commands (Rust → Frontend API)

```rust
#[tauri::command]
fn load_image(path: String) -> Result<ImageInfo, String>

#[tauri::command]
fn trace(selection: Rect, mode: TraceMode, smoothness: f64) -> Result<SvgData, String>

#[tauri::command]
fn export_svg(svg_data: String, output_path: String) -> Result<(), String>
```

### Types

```rust
struct Rect { x: u32, y: u32, width: u32, height: u32 }

enum TraceMode { Monochrome, MultiColor { colors: u8 }, Outline }

struct ImageInfo { width: u32, height: u32, thumbnail_base64: String }

struct SvgData {
    paths: String,        // SVG path elements as string
    path_count: usize,
    viewbox: String,      // e.g., "0 0 964 964"
    estimated_size: usize // bytes
}
```

## Error Handling

- **Unsupported format**: show toast notification, keep previous state
- **Trace failure**: show error in preview panel, allow retry with different settings
- **Export failure** (permissions, disk full): show error dialog with details

## Non-Goals (out of scope)

- Node/path editing within the app (use Inkscape for that)
- Batch processing multiple images
- Auto background removal / magic wand selection
- SVG optimization beyond curve simplification (no SVGO-style minification)
- Undo/redo history
