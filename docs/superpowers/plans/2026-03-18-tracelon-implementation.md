# Tracelon Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri v2 desktop app with a Nuxt 4 + Nuxt UI frontend that loads raster images, lets users select an area, traces to SVG via vtracer, and smooths the output with a custom curve simplification pipeline.

**Architecture:** Tauri v2 Rust backend handles image loading, vtracer tracing, and a custom simplification pipeline (Douglas-Peucker + Schneider Bezier fitting + corner detection). The Nuxt 4 SPA frontend provides a two-panel UI (source image + SVG preview) with a smoothness slider that calls a fast `simplify` command on cached trace data. IPC is via Tauri `invoke()`.

**Tech Stack:** Tauri v2, Nuxt 4 (SPA), Nuxt UI, Rust (vtracer 0.6.5, kurbo 0.13.0, image 0.25, imageproc 0.26, color_quant 1.1, svg 0.18)

**Spec:** `docs/superpowers/specs/2026-03-18-tracelon-design.md`

---

## File Structure

```
tracelon/
  app/                          # Nuxt 4 app directory
    app.vue                     # Root: UApp wrapper
    pages/
      index.vue                 # Main editor page (two-panel layout)
    components/
      AppToolbar.vue            # Toolbar: open, mode, slider, export
      SourceCanvas.vue          # Left panel: image display + rectangle selection
      SvgPreview.vue            # Right panel: SVG preview + overlays
    composables/
      useTracer.ts              # Tauri IPC wrapper: load, trace, simplify, export
    assets/
      css/
        main.css                # Tailwind + Nuxt UI imports
  src-tauri/
    src/
      main.rs                   # Tauri entry point
      lib.rs                    # Tauri command registration + app state
      commands/
        mod.rs                  # Re-exports
        load.rs                 # load_image command
        trace.rs                # trace command (vtracer + simplify)
        simplify.rs             # simplify command (re-run on cached data)
        export.rs               # export_svg command
      pipeline/
        mod.rs                  # Re-exports
        douglas_peucker.rs      # Douglas-Peucker polyline simplification
        schneider.rs            # Schneider cubic Bezier fitting
        corner_detection.rs     # Corner detection + path splitting
        simplify.rs             # Pipeline orchestrator: flatten → DP → corners → fit
      types.rs                  # Shared types: Rect, TraceMode, ImageInfo, SvgData
    Cargo.toml                  # Rust dependencies
    tauri.conf.json             # Tauri config (Nuxt SPA, window size)
    capabilities/
      default.json              # Permissions
  nuxt.config.ts                # Nuxt config: ssr false, Nuxt UI, Tauri vite settings
  app.config.ts                 # Nuxt UI theme config
  package.json
  tsconfig.json
```

---

## Task 1: Project Scaffolding

**Files:**
- Create: `package.json`, `nuxt.config.ts`, `app.config.ts`, `app/app.vue`, `app/pages/index.vue`, `app/assets/css/main.css`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`

- [ ] **Step 1: Initialize Nuxt project**

```bash
cd /home/ben/repos/tracelon
npx nuxi@latest init . --force --packageManager pnpm
```

- [ ] **Step 2: Install frontend dependencies**

```bash
pnpm add @nuxt/ui tailwindcss @tauri-apps/api
pnpm add -D @tauri-apps/cli
```

- [ ] **Step 3: Configure nuxt.config.ts for Tauri SPA**

```typescript
// nuxt.config.ts
export default defineNuxtConfig({
  compatibilityDate: '2025-05-15',
  modules: ['@nuxt/ui'],
  css: ['~/assets/css/main.css'],
  ssr: false,
  devServer: { host: '0.0.0.0' },
  vite: {
    clearScreen: false,
    envPrefix: ['VITE_', 'TAURI_'],
    server: { strictPort: true },
  },
  ignore: ['**/src-tauri/**'],
})
```

- [ ] **Step 4: Create CSS entry point**

```css
/* app/assets/css/main.css */
@import "tailwindcss";
@import "@nuxt/ui";
```

- [ ] **Step 5: Create app.config.ts with dark theme**

```typescript
// app.config.ts
export default defineAppConfig({
  ui: {
    colors: {
      primary: 'violet',
      neutral: 'zinc',
    },
  },
})
```

- [ ] **Step 6: Create minimal app.vue**

```vue
<!-- app/app.vue -->
<template>
  <UApp>
    <NuxtPage />
  </UApp>
</template>
```

- [ ] **Step 7: Create placeholder index page**

```vue
<!-- app/pages/index.vue -->
<template>
  <div class="h-screen flex flex-col bg-zinc-950 text-white">
    <div class="p-4 border-b border-zinc-800">
      <h1 class="text-lg font-bold">Tracelon</h1>
    </div>
    <div class="flex-1 flex items-center justify-center text-zinc-500">
      App shell working
    </div>
  </div>
</template>
```

- [ ] **Step 8: Initialize Tauri in the project**

```bash
pnpm tauri init
```

During prompts:
- App name: `tracelon`
- Window title: `Tracelon`
- Frontend dev URL: `http://localhost:3000`
- Frontend dev command: `pnpm dev`
- Frontend build command: `pnpm generate`

- [ ] **Step 9: Configure tauri.conf.json**

Edit `src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-config-schema/schema.json",
  "productName": "Tracelon",
  "version": "0.1.0",
  "identifier": "com.tracelon.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm generate",
    "devUrl": "http://localhost:3000",
    "frontendDist": "../.output/public"
  },
  "app": {
    "title": "Tracelon",
    "windows": [
      {
        "title": "Tracelon",
        "width": 1400,
        "height": 900,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": { "csp": null }
  },
  "bundle": {
    "active": true,
    "targets": "all"
  }
}
```

- [ ] **Step 10: Add Rust dependencies to Cargo.toml**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
image = "0.25"
imageproc = "0.26"
vtracer = "0.6"
svg = "0.18"
kurbo = "0.13"
color_quant = "1.1"
base64 = "0.22"
```

- [ ] **Step 11: Create minimal lib.rs**

```rust
// src-tauri/src/lib.rs

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 12: Verify the app launches**

```bash
pnpm tauri dev
```

Expected: A window opens showing "Tracelon" header and "App shell working" text. Close the window.

- [ ] **Step 13: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri + Nuxt + Nuxt UI project"
```

---

## Task 2: Rust Types and App State

**Files:**
- Create: `src-tauri/src/types.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create types.rs**

```rust
// src-tauri/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum TraceMode {
    Monochrome,
    MultiColor { colors: u8 },
    Outline,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub thumbnail_base64: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgData {
    pub paths: String,
    pub path_count: usize,
    pub viewbox: String,
    pub estimated_size: usize,
}
```

- [ ] **Step 2: Add AppState to lib.rs**

```rust
// src-tauri/src/lib.rs
mod types;

use std::sync::Mutex;
use image::DynamicImage;

pub struct AppState {
    pub loaded_image: Option<DynamicImage>,
    pub cached_trace_paths: Option<Vec<String>>, // raw SVG path d-attributes from vtracer
    pub cached_trace_viewbox: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            loaded_image: None,
            cached_trace_paths: None,
            cached_trace_viewbox: None,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(AppState::default()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check
```

Expected: compiles with no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/types.rs src-tauri/src/lib.rs
git commit -m "feat: add Rust types and app state"
```

---

## Task 3: Image Loading Command

**Files:**
- Create: `src-tauri/src/commands/mod.rs`, `src-tauri/src/commands/load.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write test for load_image**

Add to bottom of `src-tauri/src/commands/load.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbaImage, ImageBuffer};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_png() {
        let img: RgbaImage = ImageBuffer::from_pixel(100, 80, image::Rgba([255, 0, 0, 255]));
        let mut tmp = NamedTempFile::new().unwrap();
        img.save(tmp.path()).unwrap();

        let state = Mutex::new(AppState::default());
        let info = load_image_inner(&state, tmp.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(info.width, 100);
        assert_eq!(info.height, 80);
        assert!(!info.thumbnail_base64.is_empty());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let state = Mutex::new(AppState::default());
        let result = load_image_inner(&state, "/tmp/does_not_exist_12345.png".to_string());
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Add tempfile dev-dependency**

Add to `src-tauri/Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Implement load.rs**

```rust
// src-tauri/src/commands/load.rs
use std::sync::Mutex;
use base64::Engine;
use image::GenericImageView;
use crate::types::ImageInfo;
use crate::AppState;

/// Inner function for testing without Tauri state wrapper.
pub fn load_image_inner(
    state: &Mutex<AppState>,
    path: String,
) -> Result<ImageInfo, String> {
    let img = image::open(&path).map_err(|e| format!("Failed to open image: {e}"))?;
    let (width, height) = img.dimensions();

    // Generate thumbnail: max 512px on longest side, JPEG quality 80
    let thumb = img.thumbnail(512, 512);
    let mut jpeg_buf = std::io::Cursor::new(Vec::new());
    thumb
        .write_to(&mut jpeg_buf, image::ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode thumbnail: {e}"))?;
    let thumbnail_base64 = base64::engine::general_purpose::STANDARD.encode(jpeg_buf.into_inner());

    // Store in state
    let mut state = state.lock().map_err(|e| format!("State lock error: {e}"))?;
    state.loaded_image = Some(img);
    state.cached_trace_paths = None;
    state.cached_trace_viewbox = None;

    Ok(ImageInfo {
        width,
        height,
        thumbnail_base64,
    })
}

#[tauri::command]
pub fn load_image(
    state: tauri::State<'_, Mutex<AppState>>,
    path: String,
) -> Result<ImageInfo, String> {
    load_image_inner(&state, path)
}

// ... tests at bottom (from Step 1)
```

- [ ] **Step 4: Create commands/mod.rs**

```rust
// src-tauri/src/commands/mod.rs
pub mod load;
```

- [ ] **Step 5: Register command in lib.rs**

Update `lib.rs`:

```rust
mod types;
mod commands;

// ... (AppState stays the same)

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            commands::load::load_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Run tests**

```bash
cd src-tauri && cargo test
```

Expected: 2 tests pass.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands/ src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: add load_image command with thumbnail generation"
```

---

## Task 4: Curve Simplification Pipeline — Douglas-Peucker

**Files:**
- Create: `src-tauri/src/pipeline/mod.rs`, `src-tauri/src/pipeline/douglas_peucker.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod pipeline`)

- [ ] **Step 1: Write tests for Douglas-Peucker**

```rust
// Bottom of src-tauri/src/pipeline/douglas_peucker.rs
#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Point;

    #[test]
    fn test_collinear_points_simplified() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
            Point::new(3.0, 3.0),
        ];
        let result = douglas_peucker(&points, 0.1);
        assert_eq!(result.len(), 2); // only endpoints
    }

    #[test]
    fn test_sharp_corner_preserved() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.0),
            Point::new(5.0, 5.0),
        ];
        let result = douglas_peucker(&points, 0.1);
        assert_eq!(result.len(), 3); // corner kept
    }

    #[test]
    fn test_fewer_than_3_points_unchanged() {
        let points = vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
        let result = douglas_peucker(&points, 0.1);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_high_tolerance_collapses() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.5),
            Point::new(2.0, 0.1),
            Point::new(3.0, 0.0),
        ];
        let result = douglas_peucker(&points, 10.0);
        assert_eq!(result.len(), 2); // only endpoints
    }
}
```

- [ ] **Step 2: Implement Douglas-Peucker**

```rust
// src-tauri/src/pipeline/douglas_peucker.rs
use kurbo::Point;

fn perpendicular_distance(p: Point, start: Point, end: Point) -> f64 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq < 1e-12 {
        return p.distance(start);
    }
    let t = ((p.x - start.x) * dx + (p.y - start.y) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let proj = Point::new(start.x + t * dx, start.y + t * dy);
    p.distance(proj)
}

pub fn douglas_peucker(points: &[Point], epsilon: f64) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    dp_recursive(points, 0, points.len() - 1, epsilon, &mut keep);
    points
        .iter()
        .zip(keep.iter())
        .filter_map(|(&p, &k)| if k { Some(p) } else { None })
        .collect()
}

fn dp_recursive(points: &[Point], start: usize, end: usize, epsilon: f64, keep: &mut [bool]) {
    if end <= start + 1 {
        return;
    }
    let mut max_dist = 0.0;
    let mut max_idx = start;
    for i in (start + 1)..end {
        let d = perpendicular_distance(points[i], points[start], points[end]);
        if d > max_dist {
            max_dist = d;
            max_idx = i;
        }
    }
    if max_dist > epsilon {
        keep[max_idx] = true;
        dp_recursive(points, start, max_idx, epsilon, keep);
        dp_recursive(points, max_idx, end, epsilon, keep);
    }
}
```

- [ ] **Step 3: Create pipeline/mod.rs and wire up**

```rust
// src-tauri/src/pipeline/mod.rs
pub mod douglas_peucker;
```

Add to `src-tauri/src/lib.rs`:
```rust
mod pipeline;
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test douglas_peucker
```

Expected: 4 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/pipeline/ src-tauri/src/lib.rs
git commit -m "feat: add Douglas-Peucker polyline simplification"
```

---

## Task 5: Curve Simplification Pipeline — Schneider Bezier Fitting

**Files:**
- Create: `src-tauri/src/pipeline/schneider.rs`
- Modify: `src-tauri/src/pipeline/mod.rs`

- [ ] **Step 1: Write tests for Schneider fitting**

```rust
// Bottom of src-tauri/src/pipeline/schneider.rs
#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Point;

    #[test]
    fn test_two_points_produces_single_cubic() {
        let points = vec![Point::new(0.0, 0.0), Point::new(10.0, 5.0)];
        let path = fit_curve(&points, 1.0);
        let svg = path.to_svg();
        assert!(svg.contains('C'), "Expected cubic bezier in: {svg}");
    }

    #[test]
    fn test_straight_line_points() {
        let points: Vec<Point> = (0..10).map(|i| Point::new(i as f64, 0.0)).collect();
        let path = fit_curve(&points, 1.0);
        // Should produce a path (may be one or few cubics)
        assert!(!path.to_svg().is_empty());
    }

    #[test]
    fn test_semicircle_fit() {
        // Generate semicircle points
        let points: Vec<Point> = (0..=20)
            .map(|i| {
                let t = std::f64::consts::PI * (i as f64) / 20.0;
                Point::new(t.cos() * 100.0, t.sin() * 100.0)
            })
            .collect();
        let path = fit_curve(&points, 4.0);
        let svg = path.to_svg();
        // Should have cubic bezier commands
        assert!(svg.contains('C'), "Expected cubics for semicircle: {svg}");
        // Should not have too many segments (semicircle ~ 2-4 cubics)
        let c_count = svg.matches('C').count();
        assert!(c_count <= 8, "Too many cubics ({c_count}) for semicircle");
    }

    #[test]
    fn test_single_point() {
        let points = vec![Point::new(5.0, 5.0)];
        let path = fit_curve(&points, 1.0);
        assert!(path.to_svg().contains('M'));
    }
}
```

- [ ] **Step 2: Implement Schneider fitting**

```rust
// src-tauri/src/pipeline/schneider.rs
use kurbo::{BezPath, CubicBez, Point, Vec2};

/// Fit a sequence of points with cubic Bezier curves using Schneider's algorithm.
/// `max_error` is the maximum squared distance tolerance.
pub fn fit_curve(points: &[Point], max_error: f64) -> BezPath {
    let mut path = BezPath::new();
    if points.is_empty() {
        return path;
    }
    if points.len() == 1 {
        path.move_to(points[0]);
        return path;
    }
    path.move_to(points[0]);
    let tan1 = compute_left_tangent(points, 0);
    let tan2 = compute_right_tangent(points, points.len() - 1);
    fit_cubic(&mut path, points, tan1, tan2, max_error);
    path
}

fn fit_cubic(path: &mut BezPath, points: &[Point], tan1: Vec2, tan2: Vec2, error: f64) {
    let n = points.len();
    if n == 2 {
        let dist = points[0].distance(points[1]) / 3.0;
        path.curve_to(
            points[0] + tan1 * dist,
            points[1] + tan2 * dist,
            points[1],
        );
        return;
    }

    let mut u = chord_length_parameterize(points);
    let mut bez = generate_bezier(points, &u, tan1, tan2);
    let (mut max_err, mut split_point) = compute_max_error(points, &bez, &u);

    if max_err < error {
        path.curve_to(bez.p1, bez.p2, bez.p3);
        return;
    }

    let iteration_error = error * 4.0;
    if max_err < iteration_error {
        for _ in 0..4 {
            let u_prime = reparameterize(&bez, points, &u);
            bez = generate_bezier(points, &u_prime, tan1, tan2);
            let (err, sp) = compute_max_error(points, &bez, &u_prime);
            max_err = err;
            split_point = sp;
            if max_err < error {
                path.curve_to(bez.p1, bez.p2, bez.p3);
                return;
            }
            u = u_prime;
        }
    }

    let tan_center = compute_center_tangent(points, split_point);
    fit_cubic(path, &points[..=split_point], tan1, tan_center * -1.0, error);
    fit_cubic(path, &points[split_point..], tan_center, tan2, error);
}

fn generate_bezier(points: &[Point], params: &[f64], tan1: Vec2, tan2: Vec2) -> CubicBez {
    let n = points.len();
    let first = points[0];
    let last = points[n - 1];

    let mut c = [[0.0f64; 2]; 2];
    let mut x = [0.0f64; 2];

    for i in 0..n {
        let t = params[i];
        let b1 = 3.0 * t * (1.0 - t) * (1.0 - t);
        let b2 = 3.0 * t * t * (1.0 - t);
        let a1 = tan1 * b1;
        let a2 = tan2 * b2;

        c[0][0] += a1.dot(a1);
        c[0][1] += a1.dot(a2);
        c[1][0] = c[0][1];
        c[1][1] += a2.dot(a2);

        let b0 = (1.0 - t).powi(3);
        let b3 = t.powi(3);
        let tmp = points[i].to_vec2() - (first.to_vec2() * (b0 + b1) + last.to_vec2() * (b2 + b3));

        x[0] += a1.dot(tmp);
        x[1] += a2.dot(tmp);
    }

    let det_c = c[0][0] * c[1][1] - c[0][1] * c[1][0];
    let det_x_c1 = x[0] * c[1][1] - x[1] * c[0][1];
    let det_c0_x = c[0][0] * x[1] - c[0][1] * x[0];

    let seg_length = first.distance(last);
    let epsilon = 1.0e-6 * seg_length;

    let alpha_l = if det_c.abs() < 1e-12 { 0.0 } else { det_x_c1 / det_c };
    let alpha_r = if det_c.abs() < 1e-12 { 0.0 } else { det_c0_x / det_c };

    if alpha_l < epsilon || alpha_r < epsilon {
        let dist = seg_length / 3.0;
        CubicBez::new(first, first + tan1 * dist, last + tan2 * dist, last)
    } else {
        CubicBez::new(first, first + tan1 * alpha_l, last + tan2 * alpha_r, last)
    }
}

fn reparameterize(bez: &CubicBez, points: &[Point], params: &[f64]) -> Vec<f64> {
    params
        .iter()
        .zip(points.iter())
        .map(|(&t, &p)| newton_raphson(bez, p, t))
        .collect()
}

fn newton_raphson(bez: &CubicBez, point: Point, u: f64) -> f64 {
    let q_u = eval_bezier(bez, u);
    let d1 = bezier_derivative(bez);
    let q_prime = eval_quad(&d1, u);
    let d2 = quad_derivative(&d1);
    let q_prime_prime = eval_linear(&d2, u);

    let diff = q_u - point.to_vec2();
    let numerator = diff.dot(q_prime);
    let denominator = q_prime.dot(q_prime) + diff.dot(q_prime_prime);

    if denominator.abs() < 1e-12 {
        u
    } else {
        (u - numerator / denominator).clamp(0.0, 1.0)
    }
}

fn eval_bezier(bez: &CubicBez, t: f64) -> Vec2 {
    let mt = 1.0 - t;
    Vec2::new(
        mt.powi(3) * bez.p0.x + 3.0 * mt * mt * t * bez.p1.x
            + 3.0 * mt * t * t * bez.p2.x + t.powi(3) * bez.p3.x,
        mt.powi(3) * bez.p0.y + 3.0 * mt * mt * t * bez.p1.y
            + 3.0 * mt * t * t * bez.p2.y + t.powi(3) * bez.p3.y,
    )
}

fn bezier_derivative(bez: &CubicBez) -> [Vec2; 3] {
    [
        (bez.p1 - bez.p0) * 3.0,
        (bez.p2 - bez.p1) * 3.0,
        (bez.p3 - bez.p2) * 3.0,
    ]
}

fn eval_quad(d: &[Vec2; 3], t: f64) -> Vec2 {
    let mt = 1.0 - t;
    d[0] * (mt * mt) + d[1] * (2.0 * mt * t) + d[2] * (t * t)
}

fn quad_derivative(d: &[Vec2; 3]) -> [Vec2; 2] {
    [(d[1] - d[0]) * 2.0, (d[2] - d[1]) * 2.0]
}

fn eval_linear(d: &[Vec2; 2], t: f64) -> Vec2 {
    d[0] * (1.0 - t) + d[1] * t
}

fn chord_length_parameterize(points: &[Point]) -> Vec<f64> {
    let mut u = vec![0.0; points.len()];
    for i in 1..points.len() {
        u[i] = u[i - 1] + points[i].distance(points[i - 1]);
    }
    let total = *u.last().unwrap();
    if total > 1e-12 {
        for v in u.iter_mut() {
            *v /= total;
        }
    }
    u
}

fn compute_max_error(points: &[Point], bez: &CubicBez, params: &[f64]) -> (f64, usize) {
    let mut max_dist = 0.0;
    let mut split_point = points.len() / 2;
    for i in 1..points.len() - 1 {
        let p = eval_bezier(bez, params[i]).to_point();
        let dist = points[i].distance_squared(p);
        if dist > max_dist {
            max_dist = dist;
            split_point = i;
        }
    }
    (max_dist, split_point)
}

fn compute_left_tangent(points: &[Point], idx: usize) -> Vec2 {
    let tan = points[idx + 1] - points[idx];
    if tan.length() < 1e-12 { Vec2::new(1.0, 0.0) } else { tan.normalize() }
}

fn compute_right_tangent(points: &[Point], idx: usize) -> Vec2 {
    let tan = points[idx - 1] - points[idx];
    if tan.length() < 1e-12 { Vec2::new(-1.0, 0.0) } else { tan.normalize() }
}

fn compute_center_tangent(points: &[Point], idx: usize) -> Vec2 {
    let v1 = points[idx - 1] - points[idx];
    let v2 = points[idx] - points[idx + 1];
    let tan = (v1 + v2) * 0.5;
    if tan.length() < 1e-12 {
        (points[idx + 1] - points[idx - 1]).normalize()
    } else {
        tan.normalize()
    }
}
```

- [ ] **Step 3: Add to pipeline/mod.rs**

```rust
pub mod douglas_peucker;
pub mod schneider;
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test schneider
```

Expected: 4 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/pipeline/
git commit -m "feat: add Schneider cubic Bezier curve fitting"
```

---

## Task 6: Curve Simplification Pipeline — Corner Detection and Orchestrator

**Files:**
- Create: `src-tauri/src/pipeline/corner_detection.rs`, `src-tauri/src/pipeline/simplify.rs`
- Modify: `src-tauri/src/pipeline/mod.rs`

- [ ] **Step 1: Write tests for corner detection**

```rust
// Bottom of src-tauri/src/pipeline/corner_detection.rs
#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Point;

    #[test]
    fn test_sharp_90_degree_corner() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.0),
            Point::new(5.0, 5.0),
        ];
        let corners = detect_corners(&points, 60.0_f64.to_radians());
        assert_eq!(corners, vec![1]);
    }

    #[test]
    fn test_smooth_curve_no_corners() {
        let points: Vec<Point> = (0..=20)
            .map(|i| {
                let t = std::f64::consts::PI * (i as f64) / 20.0;
                Point::new(t.cos() * 100.0, t.sin() * 100.0)
            })
            .collect();
        let corners = detect_corners(&points, 60.0_f64.to_radians());
        assert!(corners.is_empty(), "Smooth semicircle should have no corners");
    }

    #[test]
    fn test_split_at_corners() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.0),
            Point::new(5.0, 5.0),
            Point::new(10.0, 5.0),
        ];
        let segments = split_at_corners(&points, 60.0_f64.to_radians());
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].len(), 2); // [p0, p1]  (corner p1 is end of first segment)
        assert_eq!(segments[1].len(), 3); // [p1, p2, p3]  (corner p1 is start of second)
    }
}
```

- [ ] **Step 2: Implement corner detection**

```rust
// src-tauri/src/pipeline/corner_detection.rs
use kurbo::Point;

/// Detect corner indices where the angle between consecutive segments
/// exceeds the threshold (in radians).
pub fn detect_corners(points: &[Point], angle_threshold: f64) -> Vec<usize> {
    if points.len() < 3 {
        return vec![];
    }
    let mut corners = Vec::new();
    for i in 1..points.len() - 1 {
        let v1 = points[i] - points[i - 1];
        let v2 = points[i + 1] - points[i];
        let dot = v1.x * v2.x + v1.y * v2.y;
        let cross = v1.x * v2.y - v1.y * v2.x;
        let angle = cross.atan2(dot).abs();
        if angle > angle_threshold {
            corners.push(i);
        }
    }
    corners
}

/// Split a polyline at detected corners. Each segment includes the corner
/// point at both ends (shared between adjacent segments).
pub fn split_at_corners(points: &[Point], angle_threshold: f64) -> Vec<Vec<Point>> {
    let corners = detect_corners(points, angle_threshold);
    if corners.is_empty() {
        return vec![points.to_vec()];
    }
    let mut segments = Vec::new();
    let mut start = 0;
    for &corner in &corners {
        segments.push(points[start..=corner].to_vec());
        start = corner;
    }
    segments.push(points[start..].to_vec());
    // Remove segments with fewer than 2 points
    segments.retain(|s| s.len() >= 2);
    segments
}
```

- [ ] **Step 3: Write tests for the simplify orchestrator**

```rust
// Bottom of src-tauri/src/pipeline/simplify.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_svg_path_roundtrip() {
        // Simple triangle path
        let input = "M0,0 L100,0 L100,100 L0,100 Z";
        let result = simplify_svg_path(input, 0.5);
        assert!(result.is_ok());
        let svg = result.unwrap();
        assert!(svg.contains('M'), "Result should have move command: {svg}");
    }

    #[test]
    fn test_smoothness_zero_preserves_detail() {
        let input = "M0,0 C10,20 30,40 50,0 C70,-40 90,-20 100,0";
        let low = simplify_svg_path(input, 0.0).unwrap();
        let high = simplify_svg_path(input, 1.0).unwrap();
        // Higher smoothness should produce fewer or equal cubic commands
        assert!(low.len() >= high.len());
    }
}
```

- [ ] **Step 4: Implement simplify orchestrator**

```rust
// src-tauri/src/pipeline/simplify.rs
use kurbo::{BezPath, PathEl, Point};
use crate::pipeline::corner_detection::split_at_corners;
use crate::pipeline::douglas_peucker::douglas_peucker;
use crate::pipeline::schneider::fit_curve;

/// Simplify an SVG path string. `smoothness` is 0.0-1.0.
/// Returns a new SVG path string with optimized curves.
pub fn simplify_svg_path(svg_path: &str, smoothness: f64) -> Result<String, String> {
    let bez = BezPath::from_svg(svg_path).map_err(|e| format!("Invalid SVG path: {e}"))?;

    // Extract subpaths (sequences of points between MoveTo commands)
    let subpaths = extract_subpaths(&bez);
    let mut result = BezPath::new();

    // Map smoothness (0.0-1.0) to algorithm parameters
    let dp_epsilon = 0.5 + smoothness * 9.5;       // 0.5 to 10.0
    let fit_error = 1.0 + smoothness * 49.0;        // 1.0 to 50.0 (squared distance)
    let corner_threshold = (60.0 + smoothness * 30.0).to_radians(); // 60° to 90°

    for (points, closed) in subpaths {
        if points.len() < 2 {
            if let Some(&p) = points.first() {
                result.move_to(p);
            }
            continue;
        }

        // Step 1: Douglas-Peucker simplification
        let simplified = douglas_peucker(&points, dp_epsilon);
        if simplified.len() < 2 {
            result.move_to(simplified[0]);
            continue;
        }

        // Step 2: Split at corners
        let segments = split_at_corners(&simplified, corner_threshold);

        // Step 3: Fit curves to each segment
        result.move_to(segments[0][0]);
        for segment in &segments {
            if segment.len() < 2 {
                continue;
            }
            let fitted = fit_curve(segment, fit_error);
            // Append fitted path elements (skip the initial MoveTo)
            for el in fitted.elements().iter().skip(1) {
                result.push(*el);
            }
        }

        if closed {
            result.close_path();
        }
    }

    Ok(result.to_svg())
}

/// Extract subpaths as vectors of points, along with whether each subpath is closed.
fn extract_subpaths(path: &BezPath) -> Vec<(Vec<Point>, bool)> {
    let mut subpaths = Vec::new();
    let mut current_points: Vec<Point> = Vec::new();
    let mut closed = false;

    // Flatten curves to polyline at high resolution
    let mut flat_els = Vec::new();
    // NOTE: verify kurbo 0.13 API — may be path.flatten(0.25, |el| ...) method
    // or kurbo::flatten(path.iter(), 0.25, |el| ...) free function depending on version.
    path.flatten(0.25, |el| flat_els.push(el));

    for el in flat_els {
        match el {
            PathEl::MoveTo(p) => {
                if !current_points.is_empty() {
                    subpaths.push((std::mem::take(&mut current_points), closed));
                    closed = false;
                }
                current_points.push(p);
            }
            PathEl::LineTo(p) => {
                current_points.push(p);
            }
            PathEl::ClosePath => {
                closed = true;
            }
            _ => {} // flatten only produces MoveTo/LineTo
        }
    }
    if !current_points.is_empty() {
        subpaths.push((current_points, closed));
    }
    subpaths
}
```

- [ ] **Step 5: Update pipeline/mod.rs**

```rust
pub mod corner_detection;
pub mod douglas_peucker;
pub mod schneider;
pub mod simplify;
```

- [ ] **Step 6: Run all pipeline tests**

```bash
cd src-tauri && cargo test pipeline
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/pipeline/
git commit -m "feat: add corner detection and simplification pipeline orchestrator"
```

---

## Task 7: Trace and Simplify Commands

**Files:**
- Create: `src-tauri/src/commands/trace.rs`, `src-tauri/src/commands/simplify.rs`, `src-tauri/src/commands/export.rs`
- Modify: `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Implement trace command**

```rust
// src-tauri/src/commands/trace.rs
use std::sync::Mutex;
use image::{DynamicImage, GenericImageView};
// NOTE: Verify vtracer 0.6 public API at compile time. The expected API is:
//   vtracer::{ColorImage, Config, ColorMode, convert}
// If the public API differs, use vtracer::convert_image_to_svg() with temp files,
// or parse vtracer's SVG string output directly.
use vtracer::{ColorImage, Config, ColorMode, convert};
use crate::types::{Rect, TraceMode, SvgData};
use crate::AppState;
use crate::pipeline::simplify::simplify_svg_path;

fn image_to_color_image(img: &DynamicImage, rect: &Rect) -> ColorImage {
    let cropped = img.crop_imm(rect.x, rect.y, rect.width, rect.height);
    let rgba = cropped.to_rgba8();
    let (w, h) = rgba.dimensions();
    ColorImage {
        pixels: rgba.into_raw(),
        width: w as usize,
        height: h as usize,
    }
}

fn trace_monochrome(img: &DynamicImage, rect: &Rect) -> Result<(Vec<String>, String), String> {
    let color_img = image_to_color_image(img, rect);
    let w = color_img.width;
    let h = color_img.height;
    let config = Config {
        color_mode: ColorMode::Binary,
        ..Config::default()
    };
    let svg_file = convert(color_img, config).map_err(|e| format!("Trace failed: {e}"))?;
    let paths: Vec<String> = svg_file.paths.iter().map(|p| p.to_string()).collect();
    Ok((paths, format!("0 0 {w} {h}")))
}

fn trace_multicolor(
    img: &DynamicImage,
    rect: &Rect,
    colors: u8,
) -> Result<(Vec<String>, String), String> {
    let color_img = image_to_color_image(img, rect);
    let w = color_img.width;
    let h = color_img.height;
    let config = Config {
        color_mode: ColorMode::Color,
        // Map user's color count to vtracer's color_precision parameter.
        // Lower precision = fewer colors. color_precision 1-8, where 8-precision = bits lost.
        // Rough mapping: 2-4 colors → precision 2, 5-8 → precision 4, 9-16 → precision 6
        color_precision: match colors {
            2..=4 => 2,
            5..=8 => 4,
            9..=12 => 6,
            _ => 8,
        },
        ..Config::default()
    };
    let svg_file = convert(color_img, config).map_err(|e| format!("Trace failed: {e}"))?;
    let paths: Vec<String> = svg_file.paths.iter().map(|p| p.to_string()).collect();
    Ok((paths, format!("0 0 {w} {h}")))
}

fn trace_outline(img: &DynamicImage, rect: &Rect) -> Result<(Vec<String>, String), String> {
    use image::{GrayImage, ImageBuffer, Luma};
    use imageproc::gradients::sobel_gradients;

    let cropped = img.crop_imm(rect.x, rect.y, rect.width, rect.height);
    let gray = cropped.to_luma8();
    // sobel_gradients returns ImageBuffer<Luma<u16>, Vec<u16>>
    let edges = sobel_gradients(&gray);

    // Threshold the 16-bit edge image to create a binary 8-bit bitmap
    let binary: GrayImage = ImageBuffer::from_fn(edges.width(), edges.height(), |x, y| {
        // u16 range: 0-65535. Typical edge threshold ~1000-3000.
        if edges.get_pixel(x, y).0[0] > 2000 {
            Luma([0u8]) // edge = black
        } else {
            Luma([255u8]) // background = white
        }
    });

    let w = binary.width() as usize;
    let h = binary.height() as usize;
    // Convert to RGBA for vtracer
    let rgba = DynamicImage::ImageLuma8(binary).to_rgba8();
    let color_img = ColorImage {
        pixels: rgba.into_raw(),
        width: w,
        height: h,
    };
    let config = Config {
        color_mode: ColorMode::Binary,
        ..Config::default()
    };
    let svg_file = convert(color_img, config).map_err(|e| format!("Trace failed: {e}"))?;
    let paths: Vec<String> = svg_file.paths.iter().map(|p| p.to_string()).collect();
    Ok((paths, format!("0 0 {w} {h}")))
}

pub fn trace_inner(
    state: &Mutex<AppState>,
    selection: Rect,
    mode: TraceMode,
    smoothness: f64,
) -> Result<SvgData, String> {
    let mut app = state.lock().map_err(|e| format!("Lock error: {e}"))?;
    let img = app.loaded_image.as_ref().ok_or("No image loaded")?;

    let (paths, viewbox) = match mode {
        TraceMode::Monochrome => trace_monochrome(img, &selection)?,
        TraceMode::MultiColor { colors } => trace_multicolor(img, &selection, colors)?,
        TraceMode::Outline => trace_outline(img, &selection)?,
    };

    // Cache raw trace
    app.cached_trace_paths = Some(paths.clone());
    app.cached_trace_viewbox = Some(viewbox.clone());

    // Drop lock before simplification (which doesn't need state)
    drop(app);

    // Apply simplification
    apply_simplification(&paths, &viewbox, smoothness)
}

pub fn apply_simplification(
    paths: &[String],
    viewbox: &str,
    smoothness: f64,
) -> Result<SvgData, String> {
    let mut simplified_paths = Vec::new();
    for path_str in paths {
        // Extract the d attribute from the SVG path element
        if let Some(d) = extract_d_attribute(path_str) {
            let simplified = simplify_svg_path(&d, smoothness)?;
            // Reconstruct path element with simplified d
            let new_path = path_str.replace(&d, &simplified);
            simplified_paths.push(new_path);
        } else {
            simplified_paths.push(path_str.clone());
        }
    }

    let all_paths = simplified_paths.join("\n");
    let estimated_size = all_paths.len();
    let path_count = simplified_paths.len();

    Ok(SvgData {
        paths: all_paths,
        path_count,
        viewbox: viewbox.to_string(),
        estimated_size,
    })
}

fn extract_d_attribute(svg_path_element: &str) -> Option<String> {
    // Extract d="..." from a <path d="..." .../> element
    let d_start = svg_path_element.find("d=\"")? + 3;
    let d_end = svg_path_element[d_start..].find('"')? + d_start;
    Some(svg_path_element[d_start..d_end].to_string())
}

#[tauri::command]
pub fn trace(
    state: tauri::State<'_, Mutex<AppState>>,
    selection: Rect,
    mode: TraceMode,
    smoothness: f64,
) -> Result<SvgData, String> {
    trace_inner(&state, selection, mode, smoothness)
}
```

- [ ] **Step 2: Implement simplify command**

```rust
// src-tauri/src/commands/simplify.rs
use std::sync::Mutex;
use crate::types::SvgData;
use crate::AppState;
use crate::commands::trace::apply_simplification;

#[tauri::command]
pub fn simplify(
    state: tauri::State<'_, Mutex<AppState>>,
    smoothness: f64,
) -> Result<SvgData, String> {
    let app = state.lock().map_err(|e| format!("Lock error: {e}"))?;
    let paths = app
        .cached_trace_paths
        .as_ref()
        .ok_or("No trace cached — run trace first")?
        .clone();
    let viewbox = app
        .cached_trace_viewbox
        .as_ref()
        .ok_or("No trace cached")?
        .clone();
    drop(app);

    apply_simplification(&paths, &viewbox, smoothness)
}
```

- [ ] **Step 3: Implement export command**

```rust
// src-tauri/src/commands/export.rs
use std::fs;

#[tauri::command]
pub fn export_svg(svg_data: String, viewbox: String, output_path: String) -> Result<(), String> {
    let svg_doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="{viewbox}">
{svg_data}
</svg>"#
    );
    fs::write(&output_path, svg_doc).map_err(|e| format!("Failed to write SVG: {e}"))
}
```

- [ ] **Step 4: Update commands/mod.rs**

```rust
// src-tauri/src/commands/mod.rs
pub mod export;
pub mod load;
pub mod simplify;
pub mod trace;
```

- [ ] **Step 5: Register all commands in lib.rs**

```rust
// Update the invoke_handler in lib.rs
.invoke_handler(tauri::generate_handler![
    commands::load::load_image,
    commands::trace::trace,
    commands::simplify::simplify,
    commands::export::export_svg,
])
```

- [ ] **Step 6: Verify it compiles**

```bash
cd src-tauri && cargo check
```

Expected: compiles with no errors. (Full integration tests require an actual image, tested via the UI.)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands/ src-tauri/src/lib.rs
git commit -m "feat: add trace, simplify, and export commands"
```

---

## Task 8: Frontend — Tauri IPC Composable

**Files:**
- Create: `app/composables/useTracer.ts`

- [ ] **Step 1: Create the composable**

```typescript
// app/composables/useTracer.ts
import { invoke } from '@tauri-apps/api/core'

export interface ImageInfo {
  width: number
  height: number
  thumbnailBase64: string
}

export interface SvgData {
  paths: string
  pathCount: number
  viewbox: string
  estimatedSize: number
}

export interface Rect {
  x: number
  y: number
  width: number
  height: number
}

export type TraceMode =
  | { type: 'Monochrome' }
  | { type: 'MultiColor'; colors: number }
  | { type: 'Outline' }

export function useTracer() {
  const imageInfo = ref<ImageInfo | null>(null)
  const svgData = ref<SvgData | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadImage(path: string) {
    loading.value = true
    error.value = null
    try {
      imageInfo.value = await invoke<ImageInfo>('load_image', { path })
      svgData.value = null
    }
    catch (e) {
      error.value = String(e)
    }
    finally {
      loading.value = false
    }
  }

  async function trace(selection: Rect, mode: TraceMode, smoothness: number) {
    loading.value = true
    error.value = null
    try {
      svgData.value = await invoke<SvgData>('trace', { selection, mode, smoothness })
    }
    catch (e) {
      error.value = String(e)
    }
    finally {
      loading.value = false
    }
  }

  async function simplify(smoothness: number) {
    error.value = null
    try {
      svgData.value = await invoke<SvgData>('simplify', { smoothness })
    }
    catch (e) {
      error.value = String(e)
    }
  }

  async function exportSvg(outputPath: string) {
    if (!svgData.value) return
    error.value = null
    try {
      await invoke('export_svg', {
        svgData: svgData.value.paths,
        viewbox: svgData.value.viewbox,
        outputPath,
      })
    }
    catch (e) {
      error.value = String(e)
    }
  }

  return { imageInfo, svgData, loading, error, loadImage, trace, simplify, exportSvg }
}
```

- [ ] **Step 2: Commit**

```bash
git add app/composables/useTracer.ts
git commit -m "feat: add useTracer composable for Tauri IPC"
```

---

## Task 9: Frontend — Toolbar Component

**Files:**
- Create: `app/components/AppToolbar.vue`

- [ ] **Step 1: Create the toolbar**

```vue
<!-- app/components/AppToolbar.vue -->
<script setup lang="ts">
import type { TraceMode } from '~/composables/useTracer'

const props = defineProps<{
  hasImage: boolean
  hasSvg: boolean
  loading: boolean
}>()

const emit = defineEmits<{
  open: []
  export: []
  trace: []
  smoothnessChange: [value: number]
}>()

const mode = defineModel<TraceMode>('mode', {
  default: () => ({ type: 'Monochrome' as const }),
})
const smoothness = defineModel<number>('smoothness', { default: 50 })
const colorCount = ref(6)

const modeItems = [
  { label: 'Mono', value: 'Monochrome' },
  { label: 'Color', value: 'MultiColor' },
  { label: 'Outline', value: 'Outline' },
]
const selectedModeValue = ref('Monochrome')

watch(selectedModeValue, (val) => {
  if (val === 'MultiColor') {
    mode.value = { type: 'MultiColor', colors: colorCount.value }
  }
  else if (val === 'Outline') {
    mode.value = { type: 'Outline' }
  }
  else {
    mode.value = { type: 'Monochrome' }
  }
})

watch(colorCount, (val) => {
  if (selectedModeValue.value === 'MultiColor') {
    mode.value = { type: 'MultiColor', colors: val }
  }
})

// Debounce smoothness changes
let smoothnessTimeout: ReturnType<typeof setTimeout> | null = null
watch(smoothness, (val) => {
  if (smoothnessTimeout) clearTimeout(smoothnessTimeout)
  smoothnessTimeout = setTimeout(() => {
    emit('smoothnessChange', val)
  }, 100)
})
</script>

<template>
  <div class="flex items-center gap-3 px-4 py-2 border-b border-zinc-800 bg-zinc-900">
    <UButton
      icon="i-lucide-folder-open"
      label="Open"
      variant="soft"
      @click="$emit('open')"
    />

    <div class="w-px h-6 bg-zinc-700" />

    <span class="text-xs text-zinc-500">Mode:</span>
    <UTabs
      v-model="selectedModeValue"
      :items="modeItems"
      variant="pill"
      size="xs"
      :content="false"
    />

    <template v-if="selectedModeValue === 'MultiColor'">
      <span class="text-xs text-zinc-500">Colors:</span>
      <input
        v-model.number="colorCount"
        type="number"
        min="2"
        max="16"
        class="w-14 px-2 py-1 text-xs bg-zinc-800 border border-zinc-700 rounded text-white"
      />
    </template>

    <div class="w-px h-6 bg-zinc-700" />

    <span class="text-xs text-zinc-500">Smoothness:</span>
    <USlider
      v-model="smoothness"
      :min="0"
      :max="100"
      :step="1"
      class="w-40"
      :disabled="!hasSvg"
    />
    <span class="text-xs text-zinc-400 w-8">{{ smoothness }}%</span>

    <UButton
      icon="i-lucide-play"
      label="Trace"
      color="primary"
      :disabled="!hasImage"
      :loading="loading"
      @click="$emit('trace')"
    />

    <div class="flex-1" />

    <UButton
      icon="i-lucide-download"
      label="Export SVG"
      color="success"
      variant="soft"
      :disabled="!hasSvg"
      @click="$emit('export')"
    />
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add app/components/AppToolbar.vue
git commit -m "feat: add toolbar component with mode toggle and smoothness slider"
```

---

## Task 10: Frontend — Source Canvas with Rectangle Selection

**Files:**
- Create: `app/components/SourceCanvas.vue`

- [ ] **Step 1: Create the component**

```vue
<!-- app/components/SourceCanvas.vue -->
<script setup lang="ts">
import type { Rect } from '~/composables/useTracer'

const props = defineProps<{
  thumbnailBase64: string | null
  imageWidth: number
  imageHeight: number
}>()

const selection = defineModel<Rect | null>('selection', { default: null })

const canvasRef = ref<HTMLCanvasElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)

// Canvas display state
const displayScale = ref(1)
const imageLoaded = ref(false)
const img = ref<HTMLImageElement | null>(null)

// Selection drawing state
const isDragging = ref(false)
const dragStart = ref({ x: 0, y: 0 })

watch(() => props.thumbnailBase64, async (b64) => {
  if (!b64 || !canvasRef.value) return
  const image = new Image()
  image.onload = () => {
    img.value = image
    imageLoaded.value = true
    fitAndDraw()
  }
  image.src = `data:image/jpeg;base64,${b64}`
})

function fitAndDraw() {
  const canvas = canvasRef.value
  const container = containerRef.value
  if (!canvas || !container || !img.value) return

  const cw = container.clientWidth
  const ch = container.clientHeight
  const scale = Math.min(cw / props.imageWidth, ch / props.imageHeight)
  displayScale.value = scale

  canvas.width = cw
  canvas.height = ch

  const ctx = canvas.getContext('2d')!
  ctx.clearRect(0, 0, cw, ch)

  const dw = props.imageWidth * scale
  const dh = props.imageHeight * scale
  const dx = (cw - dw) / 2
  const dy = (ch - dh) / 2

  ctx.drawImage(img.value, dx, dy, dw, dh)
  drawSelection(ctx, dx, dy, scale)
}

function drawSelection(ctx: CanvasRenderingContext2D, offX: number, offY: number, scale: number) {
  if (!selection.value) return
  const s = selection.value
  const x = offX + s.x * scale
  const y = offY + s.y * scale
  const w = s.width * scale
  const h = s.height * scale

  ctx.strokeStyle = '#7dd3fc'
  ctx.lineWidth = 2
  ctx.setLineDash([6, 4])
  ctx.strokeRect(x, y, w, h)
  ctx.setLineDash([])

  // Corner handles
  const hs = 6
  ctx.fillStyle = '#7dd3fc'
  for (const [cx, cy] of [[x, y], [x + w, y], [x, y + h], [x + w, y + h]]) {
    ctx.fillRect(cx - hs / 2, cy - hs / 2, hs, hs)
  }
}

function canvasToImage(cx: number, cy: number): { x: number; y: number } {
  const canvas = canvasRef.value!
  const container = containerRef.value!
  const cw = container.clientWidth
  const ch = container.clientHeight
  const scale = displayScale.value
  const dw = props.imageWidth * scale
  const dh = props.imageHeight * scale
  const dx = (cw - dw) / 2
  const dy = (ch - dh) / 2
  return {
    x: Math.round((cx - dx) / scale),
    y: Math.round((cy - dy) / scale),
  }
}

function onMouseDown(e: MouseEvent) {
  const rect = canvasRef.value!.getBoundingClientRect()
  const pos = canvasToImage(e.clientX - rect.left, e.clientY - rect.top)
  isDragging.value = true
  dragStart.value = pos
}

function onMouseMove(e: MouseEvent) {
  if (!isDragging.value) return
  const rect = canvasRef.value!.getBoundingClientRect()
  const pos = canvasToImage(e.clientX - rect.left, e.clientY - rect.top)
  const x = Math.max(0, Math.min(dragStart.value.x, pos.x))
  const y = Math.max(0, Math.min(dragStart.value.y, pos.y))
  const w = Math.min(Math.abs(pos.x - dragStart.value.x), props.imageWidth - x)
  const h = Math.min(Math.abs(pos.y - dragStart.value.y), props.imageHeight - y)
  selection.value = { x, y, width: w, height: h }
  fitAndDraw()
}

function onMouseUp() {
  isDragging.value = false
}

onMounted(() => {
  window.addEventListener('resize', fitAndDraw)
})

onUnmounted(() => {
  window.removeEventListener('resize', fitAndDraw)
})
</script>

<template>
  <div class="flex flex-col flex-1 border-r border-zinc-800">
    <div class="px-3 py-1.5 bg-zinc-900 border-b border-zinc-800 text-xs text-zinc-500 uppercase tracking-wider">
      Source Image
    </div>
    <div
      ref="containerRef"
      class="flex-1 relative overflow-hidden bg-zinc-950"
    >
      <canvas
        v-if="thumbnailBase64"
        ref="canvasRef"
        class="absolute inset-0 cursor-crosshair"
        @mousedown="onMouseDown"
        @mousemove="onMouseMove"
        @mouseup="onMouseUp"
        @mouseleave="onMouseUp"
      />
      <div
        v-else
        class="flex items-center justify-center h-full text-zinc-600"
      >
        Open an image to start
      </div>
    </div>
    <div class="px-3 py-1.5 bg-zinc-900 border-t border-zinc-800 text-xs text-zinc-600">
      <template v-if="thumbnailBase64">
        {{ imageWidth }}&times;{{ imageHeight }}
        <template v-if="selection">
          &mdash; Selection: {{ selection.width }}&times;{{ selection.height }}
        </template>
      </template>
      <template v-else>No image loaded</template>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add app/components/SourceCanvas.vue
git commit -m "feat: add source canvas component with rectangle selection"
```

---

## Task 11: Frontend — SVG Preview Component

**Files:**
- Create: `app/components/SvgPreview.vue`

- [ ] **Step 1: Create the component**

```vue
<!-- app/components/SvgPreview.vue -->
<script setup lang="ts">
import type { SvgData } from '~/composables/useTracer'

const props = defineProps<{
  svgData: SvgData | null
  thumbnailBase64: string | null
  loading: boolean
}>()

const showOverlay = ref(true)
const showControlPoints = ref(false)

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  return `${(bytes / 1024).toFixed(1)} KB`
}
</script>

<template>
  <div class="flex flex-col flex-1">
    <div class="px-3 py-1.5 bg-zinc-900 border-b border-zinc-800 text-xs uppercase tracking-wider flex justify-between">
      <span class="text-zinc-500">SVG Preview</span>
      <span v-if="svgData" class="text-emerald-500">
        {{ svgData.pathCount }} paths &bull; {{ formatSize(svgData.estimatedSize) }}
      </span>
    </div>
    <div class="flex-1 relative overflow-hidden bg-zinc-950 flex items-center justify-center">
      <div v-if="loading" class="text-zinc-500">
        <UButton loading variant="ghost" label="Tracing..." />
      </div>
      <div
        v-else-if="svgData"
        class="relative max-w-full max-h-full"
      >
        <!-- Overlay: original image behind SVG -->
        <img
          v-if="showOverlay && thumbnailBase64"
          :src="`data:image/jpeg;base64,${thumbnailBase64}`"
          class="absolute inset-0 w-full h-full object-contain opacity-20 pointer-events-none"
        />
        <!-- SVG output -->
        <div
          class="relative bg-white rounded"
          v-html="`<svg xmlns='http://www.w3.org/2000/svg' viewBox='${svgData.viewbox}' style='max-width:100%;max-height:70vh;'>${svgData.paths}</svg>`"
        />
      </div>
      <div v-else class="text-zinc-600">
        Trace an image to see the preview
      </div>
    </div>
    <div class="px-3 py-1.5 bg-zinc-900 border-t border-zinc-800 text-xs text-zinc-600 flex gap-4">
      <label class="flex items-center gap-1.5 cursor-pointer">
        <input v-model="showOverlay" type="checkbox" class="accent-violet-500" />
        Show overlay
      </label>
      <!-- TODO: Stretch goal — implement control point rendering by parsing SVG path
           cubic bezier commands and drawing handles as circles/lines overlaid on the SVG -->
      <label class="flex items-center gap-1.5 cursor-pointer opacity-50" title="Coming soon">
        <input v-model="showControlPoints" type="checkbox" class="accent-violet-500" disabled />
        Control points
      </label>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add app/components/SvgPreview.vue
git commit -m "feat: add SVG preview component with overlay toggle"
```

---

## Task 12: Frontend — Main Page (Wire Everything Together)

**Files:**
- Modify: `app/pages/index.vue`

- [ ] **Step 1: Implement the main page**

```vue
<!-- app/pages/index.vue -->
<script setup lang="ts">
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import type { Rect, TraceMode } from '~/composables/useTracer'

const { imageInfo, svgData, loading, error, loadImage, trace, simplify, exportSvg } = useTracer()
const toast = useToast()

const selection = ref<Rect | null>(null)
const mode = ref<TraceMode>({ type: 'Monochrome' })
const smoothness = ref(50)
const hasTraced = ref(false)

async function handleOpen() {
  const path = await openDialog({
    filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
  })
  if (path) {
    selection.value = null
    hasTraced.value = false
    await loadImage(path as string)
    if (error.value) {
      toast.add({ title: 'Error', description: error.value, color: 'error' })
    }
  }
}

async function handleTrace() {
  if (!imageInfo.value) return
  const sel = selection.value ?? {
    x: 0,
    y: 0,
    width: imageInfo.value.width,
    height: imageInfo.value.height,
  }
  await trace(sel, mode.value, smoothness.value / 100)
  hasTraced.value = true
  if (error.value) {
    toast.add({ title: 'Trace failed', description: error.value, color: 'error' })
  }
}

async function handleSmoothnessChange(value: number) {
  if (!hasTraced.value) return
  await simplify(value / 100)
  if (error.value) {
    toast.add({ title: 'Simplify failed', description: error.value, color: 'error' })
  }
}

async function handleExport() {
  const path = await saveDialog({
    filters: [{ name: 'SVG', extensions: ['svg'] }],
    defaultPath: 'traced.svg',
  })
  if (path) {
    await exportSvg(path as string)
    if (error.value) {
      toast.add({ title: 'Export failed', description: error.value, color: 'error' })
    }
    else {
      toast.add({ title: 'Exported', description: 'SVG saved successfully', color: 'success' })
    }
  }
}
</script>

<template>
  <div class="h-screen flex flex-col bg-zinc-950 text-white">
    <AppToolbar
      v-model:mode="mode"
      v-model:smoothness="smoothness"
      :has-image="!!imageInfo"
      :has-svg="!!svgData"
      :loading="loading"
      @open="handleOpen"
      @trace="handleTrace"
      @export="handleExport"
      @smoothness-change="handleSmoothnessChange"
    />
    <div class="flex-1 flex min-h-0">
      <SourceCanvas
        v-model:selection="selection"
        :thumbnail-base64="imageInfo?.thumbnailBase64 ?? null"
        :image-width="imageInfo?.width ?? 0"
        :image-height="imageInfo?.height ?? 0"
      />
      <SvgPreview
        :svg-data="svgData"
        :thumbnail-base64="imageInfo?.thumbnailBase64 ?? null"
        :loading="loading"
      />
    </div>
  </div>
</template>
```

- [ ] **Step 2: Add drag-and-drop support**

In `app/pages/index.vue`, add to `<script setup>`:

```typescript
import { getCurrentWindow } from '@tauri-apps/api/window'

onMounted(async () => {
  const currentWindow = getCurrentWindow()
  await currentWindow.onDragDropEvent(async (event) => {
    if (event.payload.type === 'drop' && event.payload.paths.length > 0) {
      const path = event.payload.paths[0]
      if (/\.(png|jpe?g|webp)$/i.test(path)) {
        selection.value = null
        hasTraced.value = false
        await loadImage(path)
        if (error.value) {
          toast.add({ title: 'Error', description: error.value, color: 'error' })
        }
      }
    }
  })
})
```

- [ ] **Step 3: Add filename tracking**

Add to `app/pages/index.vue` script:
```typescript
const filename = ref<string | null>(null)
```

Update `handleOpen` and the drag-drop handler to set `filename.value` from the file path (extract basename). Pass `:filename="filename"` to `SourceCanvas`. Add a `filename` prop to `SourceCanvas` and display it in the status bar.

- [ ] **Step 4: Install the Tauri dialog plugin**

```bash
pnpm add @tauri-apps/plugin-dialog
```

Add to `src-tauri/Cargo.toml` dependencies:
```toml
tauri-plugin-dialog = "2"
```

Register in `lib.rs`:
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .manage(Mutex::new(AppState::default()))
    // ... rest stays the same
```

Add capability in `src-tauri/capabilities/default.json`:
```json
{
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "dialog:allow-open",
    "dialog:allow-save"
  ]
}
```

- [ ] **Step 5: Verify the full app launches and renders**

```bash
pnpm tauri dev
```

Expected: Window opens with toolbar, two-panel layout. "Open" button should trigger a file picker. Drag-and-drop an image onto the window — should load it.

- [ ] **Step 6: Commit**

```bash
git add app/pages/index.vue app/components/ app/composables/ src-tauri/ package.json pnpm-lock.yaml
git commit -m "feat: wire up main page with toolbar, canvas, and SVG preview"
```

---

## Task 13: End-to-End Smoke Test

**Files:** None (manual testing)

- [ ] **Step 1: Launch the app**

```bash
pnpm tauri dev
```

- [ ] **Step 2: Test the full workflow**

1. Click "Open" and select a PNG image
2. Draw a rectangle selection on the source image
3. Click "Trace" — SVG should appear in right panel
4. Move the smoothness slider — SVG should update (gets smoother at higher values)
5. Switch to "Color" mode, click "Trace" — multi-color SVG
6. Switch to "Outline" mode, click "Trace" — outline-only SVG
7. Click "Export SVG" — save dialog, verify .svg file is valid
8. Toggle "Show overlay" checkbox — original image overlay appears/disappears

- [ ] **Step 3: Fix any issues found during smoke test**

Address bugs found during manual testing. Verify each fix.

- [ ] **Step 4: Commit any fixes**

```bash
git add -A
git commit -m "fix: address smoke test issues"
```

---

## Task Summary

| Task | Description | Estimated Complexity |
|------|-------------|---------------------|
| 1 | Project scaffolding (Tauri + Nuxt + Nuxt UI) | Setup |
| 2 | Rust types and app state | Small |
| 3 | Image loading command | Small |
| 4 | Douglas-Peucker simplification | Medium (algorithm) |
| 5 | Schneider Bezier fitting | Medium (algorithm) |
| 6 | Corner detection + pipeline orchestrator | Medium |
| 7 | Trace, simplify, and export commands | Medium |
| 8 | Frontend IPC composable | Small |
| 9 | Toolbar component | Small |
| 10 | Source canvas with rectangle selection | Medium (canvas) |
| 11 | SVG preview component | Small |
| 12 | Main page integration | Medium |
| 13 | End-to-end smoke test | Testing |
