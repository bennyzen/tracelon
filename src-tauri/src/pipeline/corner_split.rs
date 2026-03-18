use kurbo::{BezPath, PathEl, Point, Vec2};

/// Split an SVG path at sharp corners (angle < threshold between adjacent segments).
/// Returns multiple sub-paths. Each sub-path is a smooth run that can be
/// simplified independently without rounding the corner.
pub fn split_at_corners(svg_path: &str, angle_threshold_deg: f64) -> Result<Vec<String>, String> {
    let bez = BezPath::from_svg(svg_path).map_err(|e| format!("Invalid SVG path: {e}"))?;
    let elements = bez.elements();

    if elements.len() < 3 {
        return Ok(vec![svg_path.to_string()]);
    }

    let segments = collect_segments(elements);
    if segments.len() < 2 {
        return Ok(vec![svg_path.to_string()]);
    }

    let threshold_rad = angle_threshold_deg.to_radians();
    let mut split_indices = Vec::new();

    for i in 0..segments.len() - 1 {
        let end_tangent = segments[i].end_tangent;
        let start_tangent = segments[i + 1].start_tangent;

        if end_tangent.length() < 1e-9 || start_tangent.length() < 1e-9 {
            continue;
        }

        let angle = angle_between(end_tangent, start_tangent);
        if angle > std::f64::consts::PI - threshold_rad {
            split_indices.push(i + 1);
        }
    }

    if split_indices.is_empty() {
        return Ok(vec![svg_path.to_string()]);
    }

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
                let tangent = p - current;
                segments.push(SegmentInfo {
                    start_tangent: tangent,
                    end_tangent: tangent,
                });
                current = p;
            }
            PathEl::QuadTo(p1, p2) => {
                segments.push(SegmentInfo {
                    start_tangent: p1 - current,
                    end_tangent: p2 - p1,
                });
                current = p2;
            }
            PathEl::CurveTo(p1, p2, p3) => {
                let start_t = p1 - current;
                let end_t = p3 - p2;
                let start_t = if start_t.length() < 1e-9 { p3 - current } else { start_t };
                let end_t = if end_t.length() < 1e-9 { p3 - current } else { end_t };
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
    path.elements()
        .iter()
        .any(|el| matches!(el, PathEl::LineTo(_) | PathEl::QuadTo(_, _) | PathEl::CurveTo(_, _, _)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_corners_returns_single() {
        let path = "M0,0 C10,20 30,40 50,50 C70,60 90,70 100,100";
        let result = split_at_corners(path, 135.0).unwrap();
        assert_eq!(result.len(), 1, "Smooth path should not be split");
    }

    #[test]
    fn test_right_angle_corner_splits() {
        // Right angle: go right then go up = 90° turn
        let path = "M0,0 L100,0 L100,100";
        let result = split_at_corners(path, 135.0).unwrap();
        assert_eq!(result.len(), 2, "90° corner should be split: {:?}", result);
    }

    #[test]
    fn test_straight_line_no_split() {
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
    fn test_rejoin() {
        let parts = vec!["M0,0 L100,0".to_string(), "M100,0 L100,100".to_string()];
        let joined = rejoin_paths(&parts);
        assert!(joined.contains("M0,0"));
        assert!(joined.contains("M100,0"));
    }

    #[test]
    fn test_single_segment_no_split() {
        let path = "M0,0 L100,0";
        let result = split_at_corners(path, 135.0).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_multiple_corners() {
        // Square: 4 right angles
        let path = "M0,0 L100,0 L100,100 L0,100 L0,0";
        let result = split_at_corners(path, 135.0).unwrap();
        assert!(result.len() >= 3, "Square should have multiple splits: {:?}", result);
    }
}
