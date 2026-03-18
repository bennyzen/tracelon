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
