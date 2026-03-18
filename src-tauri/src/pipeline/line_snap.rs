use kurbo::{BezPath, Line, PathEl, Point};

/// Replace near-flat cubic Bézier segments with straight line segments.
/// A cubic is "flat" when both control points are within `tolerance` of the
/// line from p0 to p3.
pub fn snap_lines(svg_path: &str, tolerance: f64) -> Result<String, String> {
    let bez = BezPath::from_svg(svg_path).map_err(|e| format!("Invalid SVG path: {e}"))?;
    let mut result = BezPath::new();

    for el in bez.elements() {
        match *el {
            PathEl::CurveTo(p1, p2, p3) => {
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
fn is_flat(p0: Point, p1: Point, p2: Point, p3: Point, tolerance: f64) -> bool {
    let chord = Line::new(p0, p3);
    let chord_len = chord.length();

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
        let path = "M0,0 C33,0 66,0 100,0";
        let result = snap_lines(path, 1.5).unwrap();
        assert!(!result.contains('C'), "Flat cubic should become line: {result}");
        assert!(result.contains('L'), "Should have line command: {result}");
    }

    #[test]
    fn test_curved_cubic_stays() {
        let path = "M0,0 C0,50 100,50 100,0";
        let result = snap_lines(path, 1.5).unwrap();
        assert!(result.contains('C'), "Curved cubic should stay: {result}");
    }

    #[test]
    fn test_nearly_flat_snaps() {
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
        let path = "M0,0 C33,0 66,0 100,0 C100,50 50,100 0,100";
        let result = snap_lines(path, 1.5).unwrap();
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
