use kurbo::Point;

pub fn detect_corners(points: &[Point], angle_threshold: f64) -> Vec<usize> {
    if points.len() < 3 {
        return vec![];
    }
    let mut corners = Vec::new();
    let mut skip_next = false;
    for i in 1..points.len() - 1 {
        if skip_next {
            skip_next = false;
            continue;
        }
        let v1 = points[i] - points[i - 1];
        let v2 = points[i + 1] - points[i];
        let dot = v1.x * v2.x + v1.y * v2.y;
        let cross = v1.x * v2.y - v1.y * v2.x;
        let angle = cross.atan2(dot).abs();
        if angle > angle_threshold {
            corners.push(i);
            skip_next = true;
        }
    }
    corners
}

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
    segments.retain(|s| s.len() >= 2);
    segments
}

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
        assert_eq!(segments[0].len(), 2);
        assert_eq!(segments[1].len(), 3);
    }
}
