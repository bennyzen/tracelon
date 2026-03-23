use std::sync::{Arc, Mutex};
use image::DynamicImage;
use visioncortex::PathSimplifyMode;
use crate::types::{Rect, TraceMode, SvgData, PipelineParams};
use crate::AppState;
use crate::pipeline::simplify::simplify_svg_path;
use crate::pipeline::segment_count::count_segments;
use crate::pipeline::line_snap::snap_lines;

macro_rules! trace_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!($($arg)*);
    };
}

fn build_vtracer_config(color_mode: vtracer::ColorMode, mode_hint: &TraceMode) -> vtracer::Config {
    match mode_hint {
        TraceMode::Monochrome | TraceMode::Outline => {
            // Line art / logos: tighter corners, finer detail
            vtracer::Config {
                color_mode,
                mode: PathSimplifyMode::Spline,
                filter_speckle: 4,
                corner_threshold: 45,
                length_threshold: 3.0,
                splice_threshold: 40,
                ..vtracer::Config::default()
            }
        }
        TraceMode::MultiColor { .. } => {
            // Color images: more forgiving, filter more noise
            vtracer::Config {
                color_mode,
                mode: PathSimplifyMode::Spline,
                filter_speckle: 8,
                corner_threshold: 60,
                length_threshold: 4.0,
                splice_threshold: 45,
                ..vtracer::Config::default()
            }
        }
    }
}


fn trace_monochrome(img: &DynamicImage, rect: &Rect) -> Result<(Vec<String>, String), String> {
    use imageproc::contrast::{otsu_level, threshold, ThresholdType};

    // Pre-process: Otsu's threshold to get a clean black/white bitmap.
    // This handles images where both foreground and background are dark (or both light).
    let cropped = img.crop_imm(rect.x, rect.y, rect.width, rect.height);
    let gray = cropped.to_luma8();
    let otsu = otsu_level(&gray);
    trace_log!("[trace] Otsu threshold: {}", otsu);
    let binary = threshold(&gray, otsu, ThresholdType::Binary);

    let w = binary.width() as usize;
    let h = binary.height() as usize;
    let rgba = DynamicImage::ImageLuma8(binary).to_rgba8();
    let color_img = vtracer::ColorImage {
        pixels: rgba.into_raw(),
        width: w,
        height: h,
    };
    let config = build_vtracer_config(vtracer::ColorMode::Binary, &TraceMode::Monochrome);
    let svg_file = vtracer::convert(color_img, config).map_err(|e| format!("Trace failed: {e}"))?;
    let paths: Vec<String> = svg_file.paths.iter().map(|p| p.to_string()).collect();
    trace_log!("[trace] monochrome: {} paths", paths.len());
    Ok((paths, format!("0 0 {w} {h}")))
}

/// Pre-quantize an RGBA image to exactly `n_colors` using median-cut, then
/// replace every pixel with its nearest palette color. This guarantees vtracer
/// sees at most N distinct colors.
fn quantize_image(rgba: &mut image::RgbaImage, n_colors: u8) {
    use color_quant::NeuQuant;

    // NeuQuant needs &[u8] of RGBA pixels
    let pixels = rgba.as_raw();
    // sample_factor: 1 = best quality (sample every pixel), 10 = fast
    let nq = NeuQuant::new(10, n_colors.max(2) as usize, pixels);

    for pixel in rgba.pixels_mut() {
        let idx = nq.index_of(&pixel.0);
        let mapped = nq.lookup(idx).unwrap_or([0, 0, 0, 255]);
        pixel.0 = [mapped[0], mapped[1], mapped[2], pixel.0[3]];
    }
    trace_log!("[trace] quantized to {} colors", n_colors);
}

fn trace_multicolor(img: &DynamicImage, rect: &Rect, colors: u8, cutout: bool, filter_speckle: u32, color_precision: u8) -> Result<(Vec<String>, String), String> {
    let cropped = img.crop_imm(rect.x, rect.y, rect.width, rect.height);
    let mut rgba = cropped.to_rgba8();
    let (w, h) = rgba.dimensions();

    // Pre-quantize to exact color count so vtracer can't introduce extra colors
    quantize_image(&mut rgba, colors);

    let color_img = vtracer::ColorImage {
        pixels: rgba.into_raw(),
        width: w as usize,
        height: h as usize,
    };

    let mode = TraceMode::MultiColor { colors, cutout, filter_speckle, color_precision };
    let mut config = build_vtracer_config(vtracer::ColorMode::Color, &mode);
    config.hierarchical = if cutout { vtracer::Hierarchical::Cutout } else { vtracer::Hierarchical::Stacked };
    config.filter_speckle = filter_speckle as usize;
    config.color_precision = color_precision as i32;
    config.layer_difference = 4;

    let svg_file = vtracer::convert(color_img, config).map_err(|e| format!("Trace failed: {e}"))?;
    let paths: Vec<String> = svg_file.paths.iter().map(|p| p.to_string()).collect();
    trace_log!("[trace] multicolor: {} paths, {} colors requested", paths.len(), colors);
    Ok((paths, format!("0 0 {w} {h}")))
}

fn trace_outline(img: &DynamicImage, rect: &Rect) -> Result<(Vec<String>, String), String> {
    use imageproc::contrast::{otsu_level, threshold, ThresholdType};

    // Same Otsu pre-processing as monochrome, then convert fills to strokes
    let cropped = img.crop_imm(rect.x, rect.y, rect.width, rect.height);
    let gray = cropped.to_luma8();
    let otsu = otsu_level(&gray);
    let binary = threshold(&gray, otsu, ThresholdType::Binary);

    let w = binary.width() as usize;
    let h = binary.height() as usize;
    let rgba = DynamicImage::ImageLuma8(binary).to_rgba8();
    let color_img = vtracer::ColorImage {
        pixels: rgba.into_raw(),
        width: w,
        height: h,
    };
    let config = build_vtracer_config(vtracer::ColorMode::Binary, &TraceMode::Outline);
    let svg_file = vtracer::convert(color_img, config).map_err(|e| format!("Trace failed: {e}"))?;

    // Convert filled paths to stroked outlines:
    // Replace fill="..." with fill="none" stroke="black" stroke-width="1"
    let paths: Vec<String> = svg_file.paths.iter().map(|p| {
        let s = p.to_string();
        if let Some(fill_start) = s.find(" fill=\"") {
            let attr_start = fill_start;
            let val_start = fill_start + 7; // len of ` fill="`
            let val_end = s[val_start..].find('"').map(|e| val_start + e + 1).unwrap_or(s.len());
            format!("{} fill=\"none\" stroke=\"black\" stroke-width=\"1\"{}", &s[..attr_start], &s[val_end..])
        } else {
            format!("{} fill=\"none\" stroke=\"black\" stroke-width=\"1\"", s.trim_end_matches("/>").trim_end())
                + "/>"
        }
    }).collect();
    trace_log!("[trace] outline: {} paths", paths.len());
    Ok((paths, format!("0 0 {w} {h}")))
}

pub fn trace_inner(state: &Mutex<AppState>, selection: Rect, mode: TraceMode, smoothness: f64) -> Result<SvgData, String> {
    trace_log!("[trace] Starting: mode={:?} smoothness={}", mode, smoothness);
    let img = {
        let app = state.lock().map_err(|e| format!("Lock error: {e}"))?;
        app.loaded_image.as_ref().ok_or("No image loaded")?.clone()
    };
    // Lock is dropped here — trace computation runs without holding the mutex

    let (paths, viewbox) = match mode {
        TraceMode::Monochrome => trace_monochrome(&img, &selection)?,
        TraceMode::MultiColor { colors, cutout, filter_speckle, color_precision } => trace_multicolor(&img, &selection, colors, cutout, filter_speckle, color_precision)?,
        TraceMode::Outline => trace_outline(&img, &selection)?,
    };

    // Re-acquire lock only to store results
    let mut app = state.lock().map_err(|e| format!("Lock error: {e}"))?;
    app.cached_trace_paths = Some(paths.clone());
    app.cached_trace_viewbox = Some(viewbox.clone());
    drop(app);

    apply_simplification(&paths, &viewbox, &PipelineParams::from_smoothness(smoothness))
}

pub fn apply_simplification(paths: &[String], viewbox: &str, params: &PipelineParams) -> Result<SvgData, String> {
    // Count raw segments before any processing
    let raw_segment_count: usize = paths.iter().map(|p| {
        extract_d_attribute(p).map(|(_, _, d)| count_segments(&d)).unwrap_or(0)
    }).sum();

    let flatness = params.line_snap;
    let do_simplify = params.smoothness >= 0.01;

    let mut simplified_paths = Vec::new();
    for path_str in paths {
        if let Some((d_start, d_end, d)) = extract_d_attribute(path_str) {
            // Stage 1: Simplify the whole path with kurbo (skip at smoothness 0)
            let simplified = if do_simplify {
                simplify_svg_path(&d, params.smoothness).unwrap_or_else(|_| d.clone())
            } else {
                d.clone()
            };

            // Stage 2: Snap near-flat cubics/runs to lines (always runs)
            let snapped = snap_lines(&simplified, flatness).unwrap_or(simplified);

            // Splice by byte offset instead of string replace
            let new_path = format!("{}{}{}", &path_str[..d_start], &snapped, &path_str[d_end..]);
            simplified_paths.push(new_path);
        } else {
            simplified_paths.push(path_str.clone());
        }
    }

    let all_paths = simplified_paths.join("\n");
    let estimated_size = all_paths.len();
    let path_count = simplified_paths.len();
    let segment_count: usize = simplified_paths.iter().map(|p| {
        extract_d_attribute(p).map(|(_, _, d)| count_segments(&d)).unwrap_or(0)
    }).sum();

    Ok(SvgData { paths: all_paths, path_count, segment_count, raw_segment_count, viewbox: viewbox.to_string(), estimated_size })
}

fn extract_d_attribute(svg_path_element: &str) -> Option<(usize, usize, String)> {
    // Search for ' d="' to avoid matching 'id="', 'stroke-width="' etc.
    let marker = " d=\"";
    let d_attr_start = svg_path_element.find(marker)?;
    let d_start = d_attr_start + marker.len();
    let d_end = svg_path_element[d_start..].find('"')? + d_start;
    Some((d_start, d_end, svg_path_element[d_start..d_end].to_string()))
}

#[tauri::command]
pub async fn trace(state: tauri::State<'_, Arc<Mutex<AppState>>>, selection: Rect, mode: TraceMode, smoothness: f64) -> Result<SvgData, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        trace_inner(&state, selection, mode, smoothness)
    })
    .await
    .map_err(|e| format!("Task error: {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(smoothness: f64) -> PipelineParams {
        PipelineParams::from_smoothness(smoothness)
    }

    #[test]
    fn test_apply_simplification_at_zero_still_snaps_lines() {
        // At smoothness 0, kurbo simplification is skipped but line snap still runs
        let paths = vec![
            r#"<path d="M0,0 C33,0 66,0 100,0 C100,33 100,66 100,100" fill="black"/>"#.to_string(),
        ];
        let result = apply_simplification(&paths, "0 0 100 100", &params(0.0)).unwrap();
        assert_eq!(result.path_count, 1);
        assert!(result.segment_count > 0);
        // Flat cubics should be snapped to lines even at smoothness 0
        assert!(result.paths.contains('L'), "Flat cubics should become lines: {}", result.paths);
    }

    #[test]
    fn test_apply_simplification_reduces_segments() {
        let mut d = "M0,0".to_string();
        for i in 1..=20 {
            let x = i as f64 * 5.0;
            let wobble = if i % 2 == 0 { 0.3 } else { -0.3 };
            d.push_str(&format!(" C{},{} {},{} {},{}", x - 3.0, wobble, x - 1.5, -wobble, x, 0.0));
        }
        let paths = vec![format!(r#"<path d="{d}" fill="black"/>"#)];

        let result_raw = apply_simplification(&paths, "0 0 100 100", &params(0.0)).unwrap();
        let result_smooth = apply_simplification(&paths, "0 0 100 100", &params(0.5)).unwrap();

        assert!(
            result_smooth.segment_count <= result_raw.segment_count,
            "Smoothing should reduce segments: raw={} smooth={}",
            result_raw.segment_count, result_smooth.segment_count
        );
    }

    #[test]
    fn test_apply_simplification_snaps_flat_cubics() {
        let paths = vec![
            r#"<path d="M0,0 C33,0 66,0 100,0" fill="black"/>"#.to_string(),
        ];
        let result = apply_simplification(&paths, "0 0 100 100", &params(0.5)).unwrap();
        assert!(
            result.paths.contains('L') || !result.paths.contains("C33"),
            "Flat cubic should be snapped to line: {}", result.paths
        );
    }

    #[test]
    fn test_extract_d_attribute() {
        let svg = r#"<path d="M0,0 L100,0" fill="black"/>"#;
        assert_eq!(extract_d_attribute(svg), Some((9, 20, "M0,0 L100,0".to_string())));
    }

    #[test]
    fn test_extract_d_attribute_no_d() {
        assert_eq!(extract_d_attribute(r#"<rect width="10" />"#), None);
    }

    #[test]
    fn test_segment_count_populated() {
        let paths = vec![
            r#"<path d="M0,0 L100,0 L100,100 L0,100 Z" fill="black"/>"#.to_string(),
        ];
        let result = apply_simplification(&paths, "0 0 100 100", &params(0.0)).unwrap();
        assert!(result.segment_count >= 3, "Should count line segments: {}", result.segment_count);
    }
}
