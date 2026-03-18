# CLAUDE.md

## Project Overview

Tracelon is a desktop app for tracing raster images to clean SVG with optimized curves. Built with Tauri v2 (Rust backend) + Nuxt 4 SPA (Vue frontend) + Nuxt UI.

## Build & Run

```bash
pnpm install          # install frontend deps
pnpm tauri dev        # run dev (first build compiles ~600 crates)
pnpm tauri build      # production build
```

## Tests

```bash
cd src-tauri && cargo test    # run all Rust tests (32 tests)
```

No frontend test suite. Rust tests cover the pipeline modules and integration.

## Architecture

### Backend (src-tauri/src/)

- **commands/** — Tauri IPC commands (load, trace, simplify, export)
- **pipeline/** — SVG post-processing stages:
  - `simplify.rs` — kurbo `simplify_bezpath` (curve reduction)
  - `line_snap.rs` — two-pass line straightening (per-segment + run-level collinear merge)
  - `segment_count.rs` — count drawing segments in SVG paths
  - `corner_split.rs` — corner detection (written but not wired into pipeline — breaks filled paths)
- **types.rs** — shared types: `Rect`, `TraceMode`, `PipelineParams`, `SvgData`, `ImageInfo`
- **lib.rs** — AppState (loaded image + cached trace paths), Tauri setup

### Processing Pipeline

```
Image → vtracer (Spline mode, adaptive config) → kurbo simplify → line snap → SVG
```

- vtracer config is tuned per mode: tighter corners for mono/outline, more filtering for color
- Multicolor mode pre-quantizes to exact N colors via NeuQuant before tracing
- Mono/outline use Otsu's automatic threshold for foreground/background separation
- Line snap always runs (even at smoothness 0) — straightens wobbly near-straight segments
- Corner splitting is NOT in the pipeline — it shatters filled paths by breaking path continuity

### Frontend (app/)

- **composables/useTracer.ts** — wraps Tauri IPC with reactive refs, uses double-rAF for loading state
- **components/AppToolbar.vue** — mode selection, smoothness slider, collapsible tune panel with individual pipeline params
- **components/SvgPreview.vue** — zoomable/pannable SVG preview (zoom baked into SVG width/height for crisp rendering)
- **components/SourceCanvas.vue** — image display with rectangle selection
- **pages/index.vue** — main page wiring everything together

## Key Conventions

- Tauri v2 IPC: `#[serde(rename_all = "camelCase")]` on Rust response types, snake_case Rust ↔ camelCase JS
- Tauri v2 capabilities must be explicitly granted in `src-tauri/capabilities/default.json`
- Custom title bar: `decorations: false` in tauri.conf.json, `data-tauri-drag-region` on toolbar
- `ssr: false` in nuxt.config.ts (SPA mode for Tauri)
- Never use CSS `transform: scale()` on SVG — it rasterizes. Change SVG width/height instead.

## Known Pitfalls

- vtracer default `mode` is `PathSimplifyMode::None` (pixelated). Must use `PathSimplifyMode::Spline` (from `visioncortex` crate, not `vtracer`).
- vtracer's `color_precision`/`layer_difference` don't guarantee exact color counts — pre-quantize instead.
- `invoke()` blocks the JS thread. Use double `requestAnimationFrame` before IPC calls to ensure loading UI paints.
- kurbo's `simplify_bezpath` re-fits curves from scratch — even small accuracy values can change path structure significantly. Use quadratic mapping for gentler low-end.
