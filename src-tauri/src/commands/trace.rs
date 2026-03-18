use std::sync::Mutex;
use image::DynamicImage;
use visioncortex::PathSimplifyMode;
use crate::types::{Rect, TraceMode, SvgData, PipelineParams};
use crate::AppState;
use crate::pipeline::simplify::simplify_svg_path;
use crate::pipeline::segment_count::count_segments;
use crate::pipeline::line_snap::snap_lines;

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

fn image_to_color_image(img: &DynamicImage, rect: &Rect) -> vtracer::ColorImage {
    let cropped = img.crop_imm(rect.x, rect.y, rect.width, rect.height);
    let rgba = cropped.to_rgba8();
    let (w, h) = rgba.dimensions();
    vtracer::ColorImage {
        pixels: rgba.into_raw(),
        width: w as usize,
        height: h as usize,
    }
}

fn trace_monochrome(img: &DynamicImage, rect: &Rect) -> Result<(Vec<String>, String), String> {
    use imageproc::contrast::{otsu_level, threshold, ThresholdType};

    // Pre-process: Otsu's threshold to get a clean black/white bitmap.
    // This handles images where both foreground and background are dark (or both light).
    let cropped = img.crop_imm(rect.x, rect.y, rect.width, rect.height);
    let gray = cropped.to_luma8();
    let otsu = otsu_level(&gray);
    eprintln!("[trace] Otsu threshold: {}", otsu);
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
    let paths: Vec<String> = svg_file.paths.iter().map(|p| format!("{p}")).collect();
    eprintln!("[trace] monochrome: {} paths", paths.len());
    Ok((paths, format!("0 0 {w} {h}")))
}

fn trace_multicolor(img: &DynamicImage, rect: &Rect, colors: u8) -> Result<(Vec<String>, String), String> {
    let color_img = image_to_color_image(img, rect);
    let w = color_img.width;
    let h = color_img.height;
    // color_precision: higher = more color detail (1-8, where 8-val = bits lost)
    // layer_difference: higher = fewer layers/colors
    // For fewer colors (2-4), use large layer_difference to merge similar colors
    // For more colors (8+), use small layer_difference to keep detail
    let (precision, layer_diff) = match colors {
        2..=3 => (6, 64),
        4..=6 => (6, 32),
        7..=10 => (8, 16),
        _ => (8, 8),
    };
    let mut config = build_vtracer_config(vtracer::ColorMode::Color, &TraceMode::MultiColor { colors });
    config.hierarchical = vtracer::Hierarchical::Stacked;
    config.color_precision = precision;
    config.layer_difference = layer_diff;
    let svg_file = vtracer::convert(color_img, config).map_err(|e| format!("Trace failed: {e}"))?;
    let paths: Vec<String> = svg_file.paths.iter().map(|p| format!("{p}")).collect();
    eprintln!("[trace] multicolor: {} paths, precision={}, layer_diff={}", paths.len(), precision, layer_diff);
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
        let s = format!("{p}");
        // Remove existing fill, add stroke
        let s = if let Some(start) = s.find("fill=\"") {
            let end = s[start + 6..].find('"').map(|e| start + 6 + e + 1).unwrap_or(s.len());
            format!("{}fill=\"none\" stroke=\"black\" stroke-width=\"1\"{}", &s[..start], &s[end..])
        } else {
            // No fill attribute, just add stroke
            s.replace("/>", " fill=\"none\" stroke=\"black\" stroke-width=\"1\"/>")
        };
        s
    }).collect();
    eprintln!("[trace] outline: {} paths", paths.len());
    Ok((paths, format!("0 0 {w} {h}")))
}

pub fn trace_inner(state: &Mutex<AppState>, selection: Rect, mode: TraceMode, smoothness: f64) -> Result<SvgData, String> {
    eprintln!("[trace] Starting: mode={:?} smoothness={}", mode, smoothness);
    let mut app = state.lock().map_err(|e| format!("Lock error: {e}"))?;
    let img = app.loaded_image.as_ref().ok_or("No image loaded")?;

    let (paths, viewbox) = match mode {
        TraceMode::Monochrome => trace_monochrome(img, &selection)?,
        TraceMode::MultiColor { colors } => trace_multicolor(img, &selection, colors)?,
        TraceMode::Outline => trace_outline(img, &selection)?,
    };

    app.cached_trace_paths = Some(paths.clone());
    app.cached_trace_viewbox = Some(viewbox.clone());
    drop(app);

    apply_simplification(&paths, &viewbox, &PipelineParams::from_smoothness(smoothness))
}

pub fn apply_simplification(paths: &[String], viewbox: &str, params: &PipelineParams) -> Result<SvgData, String> {
    // Count raw segments before any processing
    let raw_segment_count: usize = paths.iter().map(|p| {
        extract_d_attribute(p).map(|d| count_segments(&d)).unwrap_or(0)
    }).sum();

    // At smoothness 0, return vtracer's Spline output as-is (already smooth)
    if params.smoothness < 0.01 {
        let all_paths = paths.join("\n");
        let estimated_size = all_paths.len();
        let path_count = paths.len();
        return Ok(SvgData { paths: all_paths, path_count, segment_count: raw_segment_count, raw_segment_count, viewbox: viewbox.to_string(), estimated_size });
    }

    let flatness = params.line_snap;

    let mut simplified_paths = Vec::new();
    for path_str in paths {
        if let Some(d) = extract_d_attribute(path_str) {
            // Stage 1: Simplify the whole path with kurbo (preserves path continuity)
            let simplified = simplify_svg_path(&d, params.smoothness).unwrap_or_else(|_| d.clone());

            // Stage 2: Snap near-flat cubics to lines
            let snapped = snap_lines(&simplified, flatness).unwrap_or(simplified);

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

    Ok(SvgData { paths: all_paths, path_count, segment_count, raw_segment_count, viewbox: viewbox.to_string(), estimated_size })
}

fn extract_d_attribute(svg_path_element: &str) -> Option<String> {
    let d_start = svg_path_element.find("d=\"")? + 3;
    let d_end = svg_path_element[d_start..].find('"')? + d_start;
    Some(svg_path_element[d_start..d_end].to_string())
}

#[tauri::command]
pub fn trace(state: tauri::State<'_, Mutex<AppState>>, selection: Rect, mode: TraceMode, smoothness: f64) -> Result<SvgData, String> {
    trace_inner(&state, selection, mode, smoothness)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(smoothness: f64) -> PipelineParams {
        PipelineParams::from_smoothness(smoothness)
    }

    #[test]
    fn test_apply_simplification_passthrough_at_zero() {
        let paths = vec![
            r#"<path d="M0,0 C33,0 66,0 100,0 C100,33 100,66 100,100" fill="black"/>"#.to_string(),
        ];
        let result = apply_simplification(&paths, "0 0 100 100", &params(0.0)).unwrap();
        assert_eq!(result.path_count, 1);
        assert!(result.segment_count > 0);
        assert!(result.paths.contains("C33"));
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
        assert_eq!(extract_d_attribute(svg), Some("M0,0 L100,0".to_string()));
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
