use kurbo::{BezPath, PathEl, Point};
use crate::pipeline::corner_detection::split_at_corners;
use crate::pipeline::douglas_peucker::douglas_peucker;
use crate::pipeline::schneider::fit_curve;

pub fn simplify_svg_path(svg_path: &str, smoothness: f64) -> Result<String, String> {
    let bez = BezPath::from_svg(svg_path).map_err(|e| format!("Invalid SVG path: {e}"))?;
    let subpaths = extract_subpaths(&bez);
    let mut result = BezPath::new();

    let dp_epsilon = 0.5 + smoothness * 9.5;
    let fit_error = 1.0 + smoothness * 49.0;
    let corner_threshold = (60.0 + smoothness * 30.0).to_radians();

    for (points, closed) in subpaths {
        if points.len() < 2 {
            if let Some(&p) = points.first() {
                result.move_to(p);
            }
            continue;
        }

        let simplified = douglas_peucker(&points, dp_epsilon);
        if simplified.len() < 2 {
            result.move_to(simplified[0]);
            continue;
        }

        let segments = split_at_corners(&simplified, corner_threshold);

        result.move_to(segments[0][0]);
        for segment in &segments {
            if segment.len() < 2 {
                continue;
            }
            let fitted = fit_curve(segment, fit_error);
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

fn extract_subpaths(path: &BezPath) -> Vec<(Vec<Point>, bool)> {
    let mut subpaths = Vec::new();
    let mut current_points: Vec<Point> = Vec::new();
    let mut closed = false;

    let mut flat_els = Vec::new();
    kurbo::flatten(path.iter(), 0.25, |el| flat_els.push(el));

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
            _ => {}
        }
    }
    if !current_points.is_empty() {
        subpaths.push((current_points, closed));
    }
    subpaths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_svg_path_roundtrip() {
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
        assert!(low.len() >= high.len());
    }
}
