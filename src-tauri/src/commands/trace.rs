use std::sync::Mutex;
use image::DynamicImage;
use visioncortex::PathSimplifyMode;
use crate::types::{Rect, TraceMode, SvgData};
use crate::AppState;
use crate::pipeline::simplify::simplify_svg_path;

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
    let color_img = image_to_color_image(img, rect);
    let w = color_img.width;
    let h = color_img.height;
    let config = vtracer::Config {
        color_mode: vtracer::ColorMode::Binary,
        mode: PathSimplifyMode::Spline,
        filter_speckle: 4,
        corner_threshold: 60,
        length_threshold: 4.0,
        splice_threshold: 45,
        ..vtracer::Config::default()
    };
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
    let config = vtracer::Config {
        color_mode: vtracer::ColorMode::Color,
        hierarchical: vtracer::Hierarchical::Stacked,
        mode: PathSimplifyMode::Spline,
        filter_speckle: 4,
        color_precision: precision,
        layer_difference: layer_diff,
        corner_threshold: 60,
        length_threshold: 4.0,
        splice_threshold: 45,
        ..vtracer::Config::default()
    };
    let svg_file = vtracer::convert(color_img, config).map_err(|e| format!("Trace failed: {e}"))?;
    let paths: Vec<String> = svg_file.paths.iter().map(|p| format!("{p}")).collect();
    eprintln!("[trace] multicolor: {} paths, precision={}, layer_diff={}", paths.len(), precision, layer_diff);
    Ok((paths, format!("0 0 {w} {h}")))
}

fn trace_outline(img: &DynamicImage, rect: &Rect) -> Result<(Vec<String>, String), String> {
    // Trace as monochrome filled shapes, then convert to stroke-only outlines
    let color_img = image_to_color_image(img, rect);
    let w = color_img.width;
    let h = color_img.height;
    let config = vtracer::Config {
        color_mode: vtracer::ColorMode::Binary,
        mode: PathSimplifyMode::Spline,
        filter_speckle: 4,
        corner_threshold: 60,
        length_threshold: 4.0,
        splice_threshold: 45,
        ..vtracer::Config::default()
    };
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

    apply_simplification(&paths, &viewbox, smoothness)
}

pub fn apply_simplification(paths: &[String], viewbox: &str, smoothness: f64) -> Result<SvgData, String> {
    // At smoothness 0, return vtracer's Spline output as-is (already smooth)
    if smoothness < 0.01 {
        let all_paths = paths.join("\n");
        let estimated_size = all_paths.len();
        let path_count = paths.len();
        return Ok(SvgData { paths: all_paths, path_count, viewbox: viewbox.to_string(), estimated_size });
    }

    // At smoothness > 0, use kurbo's simplify_bezpath for further curve reduction
    let mut simplified_paths = Vec::new();
    for path_str in paths {
        if let Some(d) = extract_d_attribute(path_str) {
            match simplify_svg_path(&d, smoothness) {
                Ok(simplified) => {
                    let new_path = path_str.replace(&d, &simplified);
                    simplified_paths.push(new_path);
                }
                Err(_) => {
                    simplified_paths.push(path_str.clone());
                }
            }
        } else {
            simplified_paths.push(path_str.clone());
        }
    }

    let all_paths = simplified_paths.join("\n");
    let estimated_size = all_paths.len();
    let path_count = simplified_paths.len();

    Ok(SvgData { paths: all_paths, path_count, viewbox: viewbox.to_string(), estimated_size })
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
