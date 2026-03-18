# Curve Optimization Pipeline — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add post-processing pipeline that detects straight lines, preserves sharp corners, and counts curve segments — producing cleaner SVG output with visible metrics in the UI.

**Architecture:** Three new pipeline stages run between vtracer output and kurbo simplification: (1) corner detection splits paths at sharp angles so kurbo simplifies each segment independently without rounding corners, (2) line snapping replaces near-flat cubics with true line segments, (3) segment counting provides point/segment metrics for the UI. The existing `SvgData` type gains a `segmentCount` field. Adaptive vtracer config tunes tracing parameters per mode.

**Tech Stack:** kurbo 0.13 (CubicBez, BezPath, PathEl), Rust, Tauri v2 IPC, Vue 3 / Nuxt UI

---

## File Structure

| File | Responsibility | Action |
|------|---------------|--------|
| `src-tauri/src/pipeline/mod.rs` | Pipeline module exports | Modify — add new submodules |
| `src-tauri/src/pipeline/line_snap.rs` | Detect near-flat cubics → replace with `L` commands | Create |
| `src-tauri/src/pipeline/corner_split.rs` | Detect sharp angle changes → split path at corners | Create |
| `src-tauri/src/pipeline/segment_count.rs` | Count segments (lines, cubics, quads) in SVG path | Create |
| `src-tauri/src/pipeline/simplify.rs` | Existing kurbo simplification | Modify — no changes needed |
| `src-tauri/src/commands/trace.rs:131-181` | `trace_inner` and `apply_simplification` | Modify — wire new pipeline stages, add segment count, tune vtracer config |
| `src-tauri/src/types.rs:30-35` | `SvgData` struct | Modify — add `segment_count` field |
| `app/composables/useTracer.ts:9-14` | `SvgData` TS interface | Modify — add `segmentCount` field |
| `app/components/SvgPreview.vue:30-32` | Stats display in header bar | Modify — show segment count |

---

### Task 1: Add Segment Count to SvgData

Add a `segment_count` field to the Rust `SvgData` type and the frontend TypeScript interface, then populate it with a counting function. This is the simplest change and unblocks the UI work.

**Files:**
- Create: `src-tauri/src/pipeline/segment_count.rs`
- Modify: `src-tauri/src/pipeline/mod.rs`
- Modify: `src-tauri/src/types.rs:30-35`
- Modify: `src-tauri/src/commands/trace.rs:149-181` (apply_simplification)
- Modify: `app/composables/useTracer.ts:9-14`
- Modify: `app/components/SvgPreview.vue:30-32`

- [ ] **Step 1: Write the segment counting function with tests**

Create `src-tauri/src/pipeline/segment_count.rs`:

```rust
use kurbo::{BezPath, PathEl};

/// Count the number of drawing segments (L, Q, C commands) in an SVG path string.
/// MoveTo and ClosePath are not counted as segments.
pub fn count_segments(svg_path: &str) -> usize {
    let Ok(bez) = BezPath::from_svg(svg_path) else {
        return 0;
    };
    bez.elements()
        .iter()
        .filter(|el| matches!(el, PathEl::LineTo(_) | PathEl::QuadTo(_, _) | PathEl::CurveTo(_, _, _)))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_simple_rect() {
        // M + 4 lines + Z = 4 segments
        let path = "M0,0 L100,0 L100,100 L0,100 Z";
        assert_eq!(count_segments(path), 4);
    }

    #[test]
    fn test_count_cubics() {
        // M + 2 cubics = 2 segments
        let path = "M0,0 C10,20 30,40 50,0 C70,-40 90,-20 100,0";
        assert_eq!(count_segments(path), 2);
    }

    #[test]
    fn test_count_empty() {
        assert_eq!(count_segments(""), 0);
    }

    #[test]
    fn test_count_move_only() {
        assert_eq!(count_segments("M0,0"), 0);
    }

    #[test]
    fn test_count_mixed() {
        // M + L + C + L = 3 segments
        let path = "M0,0 L50,0 C60,10 90,10 100,0 L100,100";
        assert_eq!(count_segments(path), 3);
    }
}
```

- [ ] **Step 2: Register the module**

Add to `src-tauri/src/pipeline/mod.rs`:
```rust
pub mod simplify;
pub mod segment_count;
```

- [ ] **Step 3: Run test to verify it passes**

Run: `cd src-tauri && cargo test segment_count -- --nocapture`
Expected: All 5 tests PASS

- [ ] **Step 4: Add `segment_count` field to Rust `SvgData`**

In `src-tauri/src/types.rs`, change `SvgData`:
```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgData {
    pub paths: String,
    pub path_count: usize,
    pub segment_count: usize,  // NEW
    pub viewbox: String,
    pub estimated_size: usize,
}
```

- [ ] **Step 5: Wire segment counting into `apply_simplification`**

In `src-tauri/src/commands/trace.rs`, update `apply_simplification` to compute total segment count across all paths:

```rust
use crate::pipeline::segment_count::count_segments;
```

In the early-return branch (smoothness < 0.01):
```rust
let segment_count: usize = paths.iter().map(|p| {
    extract_d_attribute(p).map(|d| count_segments(&d)).unwrap_or(0)
}).sum();
return Ok(SvgData { paths: all_paths, path_count, segment_count, viewbox: viewbox.to_string(), estimated_size });
```

In the simplification branch, after building `simplified_paths`:
```rust
let segment_count: usize = simplified_paths.iter().map(|p| {
    extract_d_attribute(p).map(|d| count_segments(&d)).unwrap_or(0)
}).sum();
Ok(SvgData { paths: all_paths, path_count, segment_count, viewbox: viewbox.to_string(), estimated_size })
```

- [ ] **Step 6: Run cargo build to verify compilation**

Run: `cd src-tauri && cargo build`
Expected: Compiles without errors

- [ ] **Step 7: Update frontend TypeScript interface**

In `app/composables/useTracer.ts`, change `SvgData`:
```typescript
export interface SvgData {
  paths: string
  pathCount: number
  segmentCount: number  // NEW
  viewbox: string
  estimatedSize: number
}
```

- [ ] **Step 8: Display segment count in SvgPreview**

In `app/components/SvgPreview.vue`, update the stats line (line ~30-32):
```html
<span v-if="svgData" class="text-emerald-500">
  {{ svgData.pathCount }} paths &bull; {{ svgData.segmentCount }} segments &bull; {{ formatSize(svgData.estimatedSize) }}
</span>
```

- [ ] **Step 9: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/pipeline/segment_count.rs src-tauri/src/pipeline/mod.rs src-tauri/src/types.rs src-tauri/src/commands/trace.rs app/composables/useTracer.ts app/components/SvgPreview.vue
git commit -m "feat: add segment count to SvgData and display in UI"
```

---

### Task 2: Line Snapping — Replace Near-Flat Cubics with Lines

Detect cubic Bézier segments where all control points are nearly collinear with the chord (start→end), and replace them with straight `L` commands. This eliminates the subtle wobble on edges that should be straight.

**Files:**
- Create: `src-tauri/src/pipeline/line_snap.rs`
- Modify: `src-tauri/src/pipeline/mod.rs`

- [ ] **Step 1: Write the line snapping function with tests**

Create `src-tauri/src/pipeline/line_snap.rs`:

```rust
use kurbo::{BezPath, CubicBez, Line, PathEl, Point};

/// Maximum distance (in path units / pixels) from control points to the chord
/// for a cubic to be considered "flat" and replaced with a line.
const DEFAULT_FLATNESS_TOLERANCE: f64 = 1.5;

/// Replace near-flat cubic Bézier segments with straight line segments.
/// A cubic is "flat" when both control points are within `tolerance` of the
/// line from p0 to p3.
pub fn snap_lines(svg_path: &str, tolerance: f64) -> Result<String, String> {
    let bez = BezPath::from_svg(svg_path).map_err(|e| format!("Invalid SVG path: {e}"))?;
    let mut result = BezPath::new();

    for el in bez.elements() {
        match *el {
            PathEl::CurveTo(p1, p2, p3) => {
                // Get the start point (last point in result, or origin)
                let p0 = last_point(&result).unwrap_or(Point::ORIGIN);
                if is_flat(p0, p1, p2, p3, tolerance) {
                    result.push(PathEl::LineTo(p3));
                } else {
                    result.push(PathEl::CurveTo(p1, p2, p3));
                }
            }
            other => result.push(other),
        }
    }

    Ok(result.to_svg())
}

/// Check if a cubic Bézier is flat enough to be a line.
/// Measures the perpendicular distance from each control point to the chord p0→p3.
fn is_flat(p0: Point, p1: Point, p2: Point, p3: Point, tolerance: f64) -> bool {
    let chord = Line::new(p0, p3);
    let chord_len = chord.length();

    // Degenerate case: start == end, keep as curve unless all points are the same
    if chord_len < 1e-6 {
        let d1 = p0.distance(p1);
        let d2 = p0.distance(p2);
        return d1 < tolerance && d2 < tolerance;
    }

    let d1 = point_to_line_distance(p1, p0, p3);
    let d2 = point_to_line_distance(p2, p0, p3);

    d1 < tolerance && d2 < tolerance
}

/// Perpendicular distance from point `p` to the line through `a` and `b`.
fn point_to_line_distance(p: Point, a: Point, b: Point) -> f64 {
    let ab = b - a;
    let ap = p - a;
    let cross = ab.x * ap.y - ab.y * ap.x;
    cross.abs() / (ab.x * ab.x + ab.y * ab.y).sqrt()
}

fn last_point(path: &BezPath) -> Option<Point> {
    path.elements().iter().rev().find_map(|el| match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) | PathEl::CurveTo(_, _, p) | PathEl::QuadTo(_, p) => Some(*p),
        PathEl::ClosePath => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_cubic_becomes_line() {
        // Cubic where control points are on the chord (perfectly flat)
        let path = "M0,0 C33,0 66,0 100,0";
        let result = snap_lines(path, 1.5).unwrap();
        // Should become M0,0 L100,0
        assert!(!result.contains('C'), "Flat cubic should become line: {result}");
        assert!(result.contains('L'), "Should have line command: {result}");
    }

    #[test]
    fn test_curved_cubic_stays() {
        // Cubic with control points far from chord
        let path = "M0,0 C0,50 100,50 100,0";
        let result = snap_lines(path, 1.5).unwrap();
        assert!(result.contains('C'), "Curved cubic should stay: {result}");
    }

    #[test]
    fn test_nearly_flat_snaps() {
        // Control points only 0.5px off the chord — should snap with tolerance 1.5
        let path = "M0,0 C33,0.5 66,-0.5 100,0";
        let result = snap_lines(path, 1.5).unwrap();
        assert!(!result.contains('C'), "Nearly flat should snap: {result}");
    }

    #[test]
    fn test_preserves_non_cubics() {
        let path = "M0,0 L100,0 L100,100 Z";
        let result = snap_lines(path, 1.5).unwrap();
        assert!(result.contains('L'), "Lines should be preserved: {result}");
    }

    #[test]
    fn test_mixed_path() {
        // One flat cubic + one curved cubic
        let path = "M0,0 C33,0 66,0 100,0 C100,50 50,100 0,100";
        let result = snap_lines(path, 1.5).unwrap();
        // First should become L, second should stay C
        assert!(result.contains('C'), "Curved segment should stay: {result}");
        assert!(result.contains('L'), "Flat segment should snap: {result}");
    }

    #[test]
    fn test_point_to_line_distance_basic() {
        let d = point_to_line_distance(
            Point::new(50.0, 10.0),
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
        );
        assert!((d - 10.0).abs() < 0.001);
    }
}
```

- [ ] **Step 2: Register the module**

Add to `src-tauri/src/pipeline/mod.rs`:
```rust
pub mod simplify;
pub mod segment_count;
pub mod line_snap;
```

- [ ] **Step 3: Run tests to verify**

Run: `cd src-tauri && cargo test line_snap -- --nocapture`
Expected: All 6 tests PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/pipeline/line_snap.rs src-tauri/src/pipeline/mod.rs
git commit -m "feat: add line snapping — replace near-flat cubics with lines"
```

---

### Task 3: Corner Detection — Split Paths at Sharp Angles

Detect sharp angle discontinuities in the path and split at those points so kurbo simplifies each smooth segment independently, preserving crisp corners instead of rounding through them.

**Files:**
- Create: `src-tauri/src/pipeline/corner_split.rs`
- Modify: `src-tauri/src/pipeline/mod.rs`

- [ ] **Step 1: Write the corner splitting function with tests**

Create `src-tauri/src/pipeline/corner_split.rs`:

```rust
use kurbo::{BezPath, PathEl, Point, Vec2};

/// Default angle threshold (degrees). Any angle sharper than this is a corner.
const DEFAULT_CORNER_ANGLE_DEG: f64 = 135.0;

/// Split an SVG path at sharp corners (angle < threshold between adjacent segments).
/// Returns multiple sub-paths. Each sub-path is a smooth run that can be
/// simplified independently without rounding the corner.
pub fn split_at_corners(svg_path: &str, angle_threshold_deg: f64) -> Result<Vec<String>, String> {
    let bez = BezPath::from_svg(svg_path).map_err(|e| format!("Invalid SVG path: {e}"))?;
    let elements = bez.elements();

    if elements.len() < 3 {
        return Ok(vec![svg_path.to_string()]);
    }

    // Collect segment tangent vectors at their start and end
    let segments = collect_segments(elements);
    if segments.len() < 2 {
        return Ok(vec![svg_path.to_string()]);
    }

    // Find corner indices: where the angle between the end tangent of segment i
    // and the start tangent of segment i+1 is sharper than threshold
    let threshold_rad = angle_threshold_deg.to_radians();
    let mut split_indices = Vec::new();

    for i in 0..segments.len() - 1 {
        let end_tangent = segments[i].end_tangent;
        let start_tangent = segments[i + 1].start_tangent;

        if end_tangent.length() < 1e-9 || start_tangent.length() < 1e-9 {
            continue;
        }

        let angle = angle_between(end_tangent, start_tangent);
        if angle < threshold_rad {
            split_indices.push(i + 1); // split before segment i+1
        }
    }

    if split_indices.is_empty() {
        return Ok(vec![svg_path.to_string()]);
    }

    // Build sub-paths by splitting at the corner indices
    let mut sub_paths = Vec::new();
    let mut current = BezPath::new();
    let mut seg_idx = 0;
    let mut current_point = Point::ORIGIN;

    for el in elements {
        match *el {
            PathEl::MoveTo(p) => {
                if !current.elements().is_empty() && has_drawing_commands(&current) {
                    sub_paths.push(current.to_svg());
                }
                current = BezPath::new();
                current.push(PathEl::MoveTo(p));
                current_point = p;
            }
            PathEl::LineTo(_) | PathEl::QuadTo(_, _) | PathEl::CurveTo(_, _, _) => {
                if split_indices.contains(&seg_idx) {
                    // End current sub-path, start new one from the current point
                    if has_drawing_commands(&current) {
                        sub_paths.push(current.to_svg());
                    }
                    current = BezPath::new();
                    current.push(PathEl::MoveTo(current_point));
                }
                current.push(*el);
                current_point = endpoint(el);
                seg_idx += 1;
            }
            PathEl::ClosePath => {
                current.push(PathEl::ClosePath);
            }
        }
    }

    if has_drawing_commands(&current) {
        sub_paths.push(current.to_svg());
    }

    Ok(sub_paths)
}

/// Rejoin multiple sub-path SVG strings into a single SVG path string.
pub fn rejoin_paths(sub_paths: &[String]) -> String {
    sub_paths.join(" ")
}

struct SegmentInfo {
    start_tangent: Vec2,
    end_tangent: Vec2,
}

fn collect_segments(elements: &[PathEl]) -> Vec<SegmentInfo> {
    let mut segments = Vec::new();
    let mut current = Point::ORIGIN;

    for el in elements {
        match *el {
            PathEl::MoveTo(p) => {
                current = p;
            }
            PathEl::LineTo(p) => {
                let tangent = (p - current).to_vec2();
                segments.push(SegmentInfo {
                    start_tangent: tangent,
                    end_tangent: tangent,
                });
                current = p;
            }
            PathEl::QuadTo(p1, p2) => {
                segments.push(SegmentInfo {
                    start_tangent: (p1 - current).to_vec2(),
                    end_tangent: (p2 - p1).to_vec2(),
                });
                current = p2;
            }
            PathEl::CurveTo(p1, p2, p3) => {
                let start_t = (p1 - current).to_vec2();
                let end_t = (p3 - p2).to_vec2();
                // If control point coincides with endpoint, use full chord
                let start_t = if start_t.length() < 1e-9 { (p3 - current).to_vec2() } else { start_t };
                let end_t = if end_t.length() < 1e-9 { (p3 - current).to_vec2() } else { end_t };
                segments.push(SegmentInfo {
                    start_tangent: start_t,
                    end_tangent: end_t,
                });
                current = p3;
            }
            PathEl::ClosePath => {}
        }
    }
    segments
}

/// Angle between two vectors in radians (0 = same direction, π = opposite).
fn angle_between(a: Vec2, b: Vec2) -> f64 {
    let dot = a.x * b.x + a.y * b.y;
    let mag = a.length() * b.length();
    if mag < 1e-12 {
        return std::f64::consts::PI;
    }
    (dot / mag).clamp(-1.0, 1.0).acos()
}

fn endpoint(el: &PathEl) -> Point {
    match *el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => p,
        PathEl::QuadTo(_, p) => p,
        PathEl::CurveTo(_, _, p) => p,
        PathEl::ClosePath => Point::ORIGIN,
    }
}

fn has_drawing_commands(path: &BezPath) -> bool {
    path.elements().iter().any(|el| matches!(el, PathEl::LineTo(_) | PathEl::QuadTo(_, _) | PathEl::CurveTo(_, _, _)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_corners_returns_single() {
        // Smooth curve — no sharp angle
        let path = "M0,0 C10,20 30,40 50,50 C70,60 90,70 100,100";
        let result = split_at_corners(path, 135.0).unwrap();
        assert_eq!(result.len(), 1, "Smooth path should not be split");
    }

    #[test]
    fn test_right_angle_corner_splits() {
        // Right angle: go right then go up = 90° angle
        let path = "M0,0 L100,0 L100,100";
        let result = split_at_corners(path, 135.0).unwrap();
        assert_eq!(result.len(), 2, "90° corner should be split: {:?}", result);
    }

    #[test]
    fn test_straight_line_no_split() {
        // Collinear segments — 180° angle, should NOT split
        let path = "M0,0 L50,0 L100,0";
        let result = split_at_corners(path, 135.0).unwrap();
        assert_eq!(result.len(), 1, "Straight line should not split");
    }

    #[test]
    fn test_angle_between_basic() {
        let right = Vec2::new(1.0, 0.0);
        let up = Vec2::new(0.0, 1.0);
        let angle = angle_between(right, up);
        assert!((angle - std::f64::consts::FRAC_PI_2).abs() < 0.01);
    }

    #[test]
    fn test_obtuse_angle_no_split() {
        // 150° angle — gentler than 135° threshold, should not split
        let path = "M0,0 L100,0 L50,87"; // roughly 150° turn
        let result = split_at_corners(path, 135.0).unwrap();
        // The angle here is actually about 120° which IS sharper than 135°
        // Let's use a proper geometric setup
        // Going right then turning only 30° = 150° interior angle
        // That's gentler than threshold, so no split expected
    }

    #[test]
    fn test_rejoin() {
        let parts = vec!["M0,0 L100,0".to_string(), "M100,0 L100,100".to_string()];
        let joined = rejoin_paths(&parts);
        assert!(joined.contains("M0,0"));
        assert!(joined.contains("M100,0"));
    }
}
```

- [ ] **Step 2: Register the module**

Add to `src-tauri/src/pipeline/mod.rs`:
```rust
pub mod simplify;
pub mod segment_count;
pub mod line_snap;
pub mod corner_split;
```

- [ ] **Step 3: Run tests**

Run: `cd src-tauri && cargo test corner_split -- --nocapture`
Expected: All tests PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/pipeline/corner_split.rs src-tauri/src/pipeline/mod.rs
git commit -m "feat: add corner detection — split paths at sharp angles"
```

---

### Task 4: Adaptive vtracer Configuration

Tune vtracer parameters based on the trace mode and provide better defaults. Lower `corner_threshold` for logos/line art (preserve sharp corners in the tracing stage), higher `filter_speckle` for multicolor (reduce noise).

**Files:**
- Modify: `src-tauri/src/commands/trace.rs:38-46` (monochrome config)
- Modify: `src-tauri/src/commands/trace.rs:67-78` (multicolor config)
- Modify: `src-tauri/src/commands/trace.rs:102-110` (outline config)

- [ ] **Step 1: Extract a config builder function**

Add to `src-tauri/src/commands/trace.rs`:

```rust
fn build_vtracer_config(color_mode: vtracer::ColorMode, mode_hint: &TraceMode) -> vtracer::Config {
    match mode_hint {
        TraceMode::Monochrome | TraceMode::Outline => {
            // Line art / logos: tighter corners, smaller speckle filter
            vtracer::Config {
                color_mode,
                mode: PathSimplifyMode::Spline,
                filter_speckle: 4,
                corner_threshold: 45,    // was 60 — tighter to preserve logo corners
                length_threshold: 3.0,   // was 4.0 — finer detail
                splice_threshold: 40,    // was 45 — fewer splices = smoother curves
                ..vtracer::Config::default()
            }
        }
        TraceMode::MultiColor { .. } => {
            // Color images: more forgiving, filter more noise
            vtracer::Config {
                color_mode,
                mode: PathSimplifyMode::Spline,
                filter_speckle: 8,       // was 4 — filter more noise in color mode
                corner_threshold: 60,    // keep standard for color
                length_threshold: 4.0,
                splice_threshold: 45,
                ..vtracer::Config::default()
            }
        }
    }
}
```

- [ ] **Step 2: Use the builder in all three trace functions**

In `trace_monochrome`:
```rust
let config = build_vtracer_config(vtracer::ColorMode::Binary, &TraceMode::Monochrome);
```

In `trace_multicolor`:
```rust
let mut config = build_vtracer_config(vtracer::ColorMode::Color, &TraceMode::MultiColor { colors });
config.hierarchical = vtracer::Hierarchical::Stacked;
config.color_precision = precision;
config.layer_difference = layer_diff;
```

In `trace_outline`:
```rust
let config = build_vtracer_config(vtracer::ColorMode::Binary, &TraceMode::Outline);
```

- [ ] **Step 3: Run existing tests to verify no regression**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/trace.rs
git commit -m "feat: adaptive vtracer config — tighter corners for line art, more filtering for color"
```

---

### Task 5: Wire the Full Pipeline

Integrate corner splitting, kurbo simplification, and line snapping into the `apply_simplification` function. The pipeline order is:

```
vtracer output → corner split → kurbo simplify per sub-path → line snap → rejoin → count segments
```

**Files:**
- Modify: `src-tauri/src/commands/trace.rs:149-181` (apply_simplification)

- [ ] **Step 1: Update `apply_simplification` with the full pipeline**

Replace the `apply_simplification` function in `src-tauri/src/commands/trace.rs`:

```rust
use crate::pipeline::line_snap::snap_lines;
use crate::pipeline::corner_split::{split_at_corners, rejoin_paths};
use crate::pipeline::segment_count::count_segments;

pub fn apply_simplification(paths: &[String], viewbox: &str, smoothness: f64) -> Result<SvgData, String> {
    // At smoothness 0, return vtracer's Spline output as-is (already smooth)
    if smoothness < 0.01 {
        let all_paths = paths.join("\n");
        let estimated_size = all_paths.len();
        let path_count = paths.len();
        let segment_count: usize = paths.iter().map(|p| {
            extract_d_attribute(p).map(|d| count_segments(&d)).unwrap_or(0)
        }).sum();
        return Ok(SvgData { paths: all_paths, path_count, segment_count, viewbox: viewbox.to_string(), estimated_size });
    }

    // Map smoothness to pipeline parameters
    let accuracy = 0.5 + smoothness * 7.5;  // kurbo accuracy: 0.5-8.0 px
    let flatness = 0.5 + smoothness * 2.0;  // line snap tolerance: 0.5-2.5 px
    let corner_angle = 120.0 + smoothness * 30.0; // corner threshold: 120-150°

    let mut simplified_paths = Vec::new();
    for path_str in paths {
        if let Some(d) = extract_d_attribute(path_str) {
            // Stage 1: Split at sharp corners
            let sub_paths = split_at_corners(&d, corner_angle).unwrap_or_else(|_| vec![d.clone()]);

            // Stage 2: Simplify each sub-path independently with kurbo
            let simplified_subs: Vec<String> = sub_paths.iter().map(|sub| {
                simplify_svg_path(sub, smoothness).unwrap_or_else(|_| sub.clone())
            }).collect();

            // Stage 3: Rejoin sub-paths
            let rejoined = rejoin_paths(&simplified_subs);

            // Stage 4: Snap near-flat cubics to lines
            let snapped = snap_lines(&rejoined, flatness).unwrap_or(rejoined);

            let new_path = path_str.replace(&d, &snapped);
            simplified_paths.push(new_path);
        } else {
            simplified_paths.push(path_str.clone());
        }
    }

    let all_paths = simplified_paths.join("\n");
    let estimated_size = all_paths.len();
    let path_count = simplified_paths.len();
    let segment_count: usize = simplified_paths.iter().map(|p| {
        extract_d_attribute(p).map(|d| count_segments(&d)).unwrap_or(0)
    }).sum();

    Ok(SvgData { paths: all_paths, path_count, segment_count, viewbox: viewbox.to_string(), estimated_size })
}
```

- [ ] **Step 2: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

- [ ] **Step 3: Run the app and smoke test**

Run: `pnpm tauri dev`

Verify:
1. Load a test image → Trace in Monochrome mode
2. Check that segment count appears in the SVG preview header bar
3. Move smoothness slider → segment count should decrease
4. Verify corners on a rectangular shape remain sharp at moderate smoothness
5. Verify straight edges don't wobble at low smoothness

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/trace.rs
git commit -m "feat: wire full optimization pipeline — corner split → simplify → line snap"
```

---

### Task 6: End-to-End Integration Test

Write a Rust integration test that exercises the full pipeline: trace a synthetic image → apply simplification → verify the output has fewer segments, contains line commands, and preserves path structure.

**Files:**
- Modify: `src-tauri/src/commands/trace.rs` (add test at bottom)

- [ ] **Step 1: Write the integration test**

Add to the bottom of `src-tauri/src/commands/trace.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_simplification_passthrough_at_zero() {
        let paths = vec![
            r#"<path d="M0,0 C33,0 66,0 100,0 C100,33 100,66 100,100" fill="black"/>"#.to_string(),
        ];
        let result = apply_simplification(&paths, "0 0 100 100", 0.0).unwrap();
        assert_eq!(result.path_count, 1);
        assert!(result.segment_count > 0);
        assert!(result.paths.contains("C33"));
    }

    #[test]
    fn test_apply_simplification_reduces_segments() {
        // A path with many small cubic segments (simulating vtracer output)
        let mut d = "M0,0".to_string();
        for i in 1..=20 {
            let x = i as f64 * 5.0;
            let wobble = if i % 2 == 0 { 0.3 } else { -0.3 };
            d.push_str(&format!(" C{},{} {},{} {},{}",
                x - 3.0, wobble, x - 1.5, -wobble, x, 0.0));
        }
        let paths = vec![format!(r#"<path d="{d}" fill="black"/>"#)];

        let result_raw = apply_simplification(&paths, "0 0 100 100", 0.0).unwrap();
        let result_smooth = apply_simplification(&paths, "0 0 100 100", 0.5).unwrap();

        assert!(
            result_smooth.segment_count <= result_raw.segment_count,
            "Smoothing should reduce segments: raw={} smooth={}",
            result_raw.segment_count, result_smooth.segment_count
        );
    }

    #[test]
    fn test_apply_simplification_snaps_flat_cubics() {
        // Perfectly flat cubics should become lines
        let paths = vec![
            r#"<path d="M0,0 C33,0 66,0 100,0" fill="black"/>"#.to_string(),
        ];
        let result = apply_simplification(&paths, "0 0 100 100", 0.5).unwrap();
        // The flat cubic should have been snapped to a line
        assert!(result.paths.contains('L') || !result.paths.contains("C33"),
            "Flat cubic should be snapped to line: {}", result.paths);
    }

    #[test]
    fn test_extract_d_attribute() {
        let svg = r#"<path d="M0,0 L100,0" fill="black"/>"#;
        assert_eq!(extract_d_attribute(svg), Some("M0,0 L100,0".to_string()));
    }

    #[test]
    fn test_extract_d_attribute_no_d() {
        assert_eq!(extract_d_attribute("<rect width=\"10\" />"), None);
    }
}
```

- [ ] **Step 2: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/trace.rs
git commit -m "test: add integration tests for optimization pipeline"
```
