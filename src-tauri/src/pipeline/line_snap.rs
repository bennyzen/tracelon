use kurbo::{BezPath, Line, PathEl, Point};

/// Two-pass line snapping:
/// 1. Per-segment: replace individual near-flat cubics with lines
/// 2. Run-level: detect sequences of segments that are collectively collinear
///    and merge them into a single line
pub fn snap_lines(svg_path: &str, tolerance: f64) -> Result<String, String> {
    let bez = BezPath::from_svg(svg_path).map_err(|e| format!("Invalid SVG path: {e}"))?;

    // Pass 1: per-segment flat cubic → line
    let mut pass1 = BezPath::new();
    for el in bez.elements() {
        match *el {
            PathEl::CurveTo(p1, p2, p3) => {
                let p0 = last_point(&pass1).unwrap_or(Point::ORIGIN);
                if is_flat(p0, p1, p2, p3, tolerance) {
                    pass1.push(PathEl::LineTo(p3));
                } else {
                    pass1.push(PathEl::CurveTo(p1, p2, p3));
                }
            }
            other => pass1.push(other),
        }
    }

    // Pass 2: merge runs of collinear segments into single lines
    let result = merge_collinear_runs(&pass1, tolerance);

    Ok(result.to_svg())
}

/// Detect runs of consecutive segments (lines or near-flat cubics) where ALL
/// intermediate points lie within `tolerance` of the line from the run's start
/// to its end. Replace each such run with a single LineTo.
fn merge_collinear_runs(path: &BezPath, tolerance: f64) -> BezPath {
    let elements = path.elements();
    let mut result = BezPath::new();
    let mut i = 0;

    while i < elements.len() {
        match elements[i] {
            PathEl::MoveTo(p) => {
                result.push(PathEl::MoveTo(p));
                i += 1;
            }
            PathEl::ClosePath => {
                result.push(PathEl::ClosePath);
                i += 1;
            }
            // For any drawing command, try to extend a collinear run
            _ => {
                let run_start = last_point(&result).unwrap_or(Point::ORIGIN);
                let (run_end_idx, run_end_point) = find_collinear_run(elements, i, run_start, tolerance);

                if run_end_idx > i + 1 {
                    // Multiple segments merged into one line
                    result.push(PathEl::LineTo(run_end_point));
                    i = run_end_idx;
                } else {
                    // Single segment, keep as-is
                    result.push(elements[i]);
                    i += 1;
                }
            }
        }
    }

    result
}

/// Starting from `start_idx`, find the longest run of consecutive drawing
/// segments where every point (endpoints and control points) lies within
/// `tolerance` of the line from `run_start` to the run's current end.
/// Returns (exclusive end index, end point).
fn find_collinear_run(elements: &[PathEl], start_idx: usize, run_start: Point, tolerance: f64) -> (usize, Point) {
    let mut end_idx = start_idx;
    let mut end_point = run_start;

    for j in start_idx..elements.len() {
        match elements[j] {
            PathEl::LineTo(p) => {
                let candidate_end = p;
                if all_points_near_line(elements, start_idx, j + 1, run_start, candidate_end, tolerance) {
                    end_idx = j + 1;
                    end_point = candidate_end;
                } else {
                    break;
                }
            }
            PathEl::CurveTo(_, _, p) => {
                let candidate_end = p;
                if all_points_near_line(elements, start_idx, j + 1, run_start, candidate_end, tolerance) {
                    end_idx = j + 1;
                    end_point = candidate_end;
                } else {
                    break;
                }
            }
            // MoveTo, ClosePath, QuadTo — stop the run
            _ => break,
        }
    }

    (end_idx, end_point)
}

/// Check if ALL points (endpoints + control points) in elements[start..end]
/// are within `tolerance` of the line from `line_a` to `line_b`.
fn all_points_near_line(elements: &[PathEl], start: usize, end: usize, line_a: Point, line_b: Point, tolerance: f64) -> bool {
    let chord_len = Line::new(line_a, line_b).length();
    if chord_len < 1e-6 {
        return false; // degenerate — don't merge
    }

    for el in &elements[start..end] {
        let points = element_points(el);
        for p in points {
            if point_to_line_distance(p, line_a, line_b) > tolerance {
                return false;
            }
        }
    }
    true
}

/// Extract all points (control points + endpoint) from a path element.
fn element_points(el: &PathEl) -> Vec<Point> {
    match *el {
        PathEl::LineTo(p) => vec![p],
        PathEl::CurveTo(p1, p2, p3) => vec![p1, p2, p3],
        PathEl::QuadTo(p1, p2) => vec![p1, p2],
        _ => vec![],
    }
}

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

    #[test]
    fn test_collinear_run_merges_wobbly_segments() {
        // 5 small segments that wobble slightly around a straight horizontal line
        let path = "M0,0 L20,0.3 L40,-0.2 L60,0.4 L80,-0.1 L100,0";
        let result = snap_lines(path, 1.5).unwrap();
        // Should merge into a single line M0,0 L100,0
        let bez = BezPath::from_svg(&result).unwrap();
        let drawing_cmds: Vec<_> = bez.elements().iter().filter(|e| {
            matches!(e, PathEl::LineTo(_) | PathEl::CurveTo(_, _, _))
        }).collect();
        assert_eq!(drawing_cmds.len(), 1, "Should merge to 1 line, got {}: {result}", drawing_cmds.len());
    }

    #[test]
    fn test_collinear_run_stops_at_curve() {
        // Straight run then a curve — should merge the straight part only
        let path = "M0,0 L50,0.2 L100,0 C100,50 50,100 0,100";
        let result = snap_lines(path, 1.5).unwrap();
        assert!(result.contains('C'), "Curve should be preserved: {result}");
    }

    #[test]
    fn test_vertical_wobbly_line_merges() {
        // Vertical line with wobble — the actual problem case
        let path = "M100,0 L100.3,50 L99.8,100 L100.1,150 L100,200";
        let result = snap_lines(path, 1.5).unwrap();
        let bez = BezPath::from_svg(&result).unwrap();
        let drawing_cmds: Vec<_> = bez.elements().iter().filter(|e| {
            matches!(e, PathEl::LineTo(_) | PathEl::CurveTo(_, _, _))
        }).collect();
        assert_eq!(drawing_cmds.len(), 1, "Wobbly vertical should merge to 1 line: {result}");
    }
}
