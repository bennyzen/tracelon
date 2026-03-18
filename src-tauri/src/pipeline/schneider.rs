use kurbo::{BezPath, CubicBez, Point, Vec2};

/// Fit a sequence of points with cubic Bezier curves using Schneider's algorithm.
/// `max_error` is the maximum squared distance tolerance.
pub fn fit_curve(points: &[Point], max_error: f64) -> BezPath {
    let mut path = BezPath::new();
    if points.is_empty() {
        return path;
    }
    if points.len() == 1 {
        path.move_to(points[0]);
        return path;
    }
    path.move_to(points[0]);
    let tan1 = compute_left_tangent(points, 0);
    let tan2 = compute_right_tangent(points, points.len() - 1);
    fit_cubic(&mut path, points, tan1, tan2, max_error);
    path
}

fn fit_cubic(path: &mut BezPath, points: &[Point], tan1: Vec2, tan2: Vec2, error: f64) {
    let n = points.len();
    if n == 2 {
        let dist = points[0].distance(points[1]) / 3.0;
        path.curve_to(
            points[0] + tan1 * dist,
            points[1] + tan2 * dist,
            points[1],
        );
        return;
    }

    let mut u = chord_length_parameterize(points);
    let mut bez = generate_bezier(points, &u, tan1, tan2);
    let (mut max_err, mut split_point) = compute_max_error(points, &bez, &u);

    if max_err < error {
        path.curve_to(bez.p1, bez.p2, bez.p3);
        return;
    }

    for _ in 0..4 {
        let u_prime = reparameterize(&bez, points, &u);
        bez = generate_bezier(points, &u_prime, tan1, tan2);
        let (err, sp) = compute_max_error(points, &bez, &u_prime);
        max_err = err;
        split_point = sp;
        if max_err < error {
            path.curve_to(bez.p1, bez.p2, bez.p3);
            return;
        }
        u = u_prime;
    }

    let tan_center = compute_center_tangent(points, split_point);
    fit_cubic(path, &points[..=split_point], tan1, tan_center * -1.0, error);
    fit_cubic(path, &points[split_point..], tan_center, tan2, error);
}

fn generate_bezier(points: &[Point], params: &[f64], tan1: Vec2, tan2: Vec2) -> CubicBez {
    let n = points.len();
    let first = points[0];
    let last = points[n - 1];

    let mut c = [[0.0f64; 2]; 2];
    let mut x = [0.0f64; 2];

    for i in 0..n {
        let t = params[i];
        let b1 = 3.0 * t * (1.0 - t) * (1.0 - t);
        let b2 = 3.0 * t * t * (1.0 - t);
        let a1 = tan1 * b1;
        let a2 = tan2 * b2;

        c[0][0] += a1.dot(a1);
        c[0][1] += a1.dot(a2);
        c[1][0] = c[0][1];
        c[1][1] += a2.dot(a2);

        let b0 = (1.0 - t).powi(3);
        let b3 = t.powi(3);
        let tmp = points[i].to_vec2() - (first.to_vec2() * (b0 + b1) + last.to_vec2() * (b2 + b3));

        x[0] += a1.dot(tmp);
        x[1] += a2.dot(tmp);
    }

    let det_c = c[0][0] * c[1][1] - c[0][1] * c[1][0];
    let det_x_c1 = x[0] * c[1][1] - x[1] * c[0][1];
    let det_c0_x = c[0][0] * x[1] - c[0][1] * x[0];

    let seg_length = first.distance(last);
    let epsilon = 1.0e-6 * seg_length;

    let alpha_l = if det_c.abs() < 1e-12 { 0.0 } else { det_x_c1 / det_c };
    let alpha_r = if det_c.abs() < 1e-12 { 0.0 } else { det_c0_x / det_c };

    if alpha_l < epsilon || alpha_r < epsilon {
        let dist = seg_length / 3.0;
        CubicBez::new(first, first + tan1 * dist, last + tan2 * dist, last)
    } else {
        CubicBez::new(first, first + tan1 * alpha_l, last + tan2 * alpha_r, last)
    }
}

fn reparameterize(bez: &CubicBez, points: &[Point], params: &[f64]) -> Vec<f64> {
    params.iter().zip(points.iter())
        .map(|(&t, &p)| newton_raphson(bez, p, t))
        .collect()
}

fn newton_raphson(bez: &CubicBez, point: Point, u: f64) -> f64 {
    let q_u = eval_bezier(bez, u);
    let d1 = bezier_derivative(bez);
    let q_prime = eval_quad(&d1, u);
    let d2 = quad_derivative(&d1);
    let q_prime_prime = eval_linear(&d2, u);

    let diff = q_u - point.to_vec2();
    let numerator = diff.dot(q_prime);
    let denominator = q_prime.dot(q_prime) + diff.dot(q_prime_prime);

    if denominator.abs() < 1e-12 { u } else { (u - numerator / denominator).clamp(0.0, 1.0) }
}

fn eval_bezier(bez: &CubicBez, t: f64) -> Vec2 {
    let mt = 1.0 - t;
    Vec2::new(
        mt.powi(3) * bez.p0.x + 3.0 * mt * mt * t * bez.p1.x
            + 3.0 * mt * t * t * bez.p2.x + t.powi(3) * bez.p3.x,
        mt.powi(3) * bez.p0.y + 3.0 * mt * mt * t * bez.p1.y
            + 3.0 * mt * t * t * bez.p2.y + t.powi(3) * bez.p3.y,
    )
}

fn bezier_derivative(bez: &CubicBez) -> [Vec2; 3] {
    [(bez.p1 - bez.p0) * 3.0, (bez.p2 - bez.p1) * 3.0, (bez.p3 - bez.p2) * 3.0]
}

fn eval_quad(d: &[Vec2; 3], t: f64) -> Vec2 {
    let mt = 1.0 - t;
    d[0] * (mt * mt) + d[1] * (2.0 * mt * t) + d[2] * (t * t)
}

fn quad_derivative(d: &[Vec2; 3]) -> [Vec2; 2] {
    [(d[1] - d[0]) * 2.0, (d[2] - d[1]) * 2.0]
}

fn eval_linear(d: &[Vec2; 2], t: f64) -> Vec2 {
    d[0] * (1.0 - t) + d[1] * t
}

fn chord_length_parameterize(points: &[Point]) -> Vec<f64> {
    let mut u = vec![0.0; points.len()];
    for i in 1..points.len() {
        u[i] = u[i - 1] + points[i].distance(points[i - 1]);
    }
    let total = *u.last().unwrap();
    if total > 1e-12 {
        for v in u.iter_mut() { *v /= total; }
    }
    u
}

fn compute_max_error(points: &[Point], bez: &CubicBez, params: &[f64]) -> (f64, usize) {
    let mut max_dist = 0.0;
    let mut split_point = points.len() / 2;
    for i in 1..points.len() - 1 {
        let p = eval_bezier(bez, params[i]).to_point();
        let dist = points[i].distance_squared(p);
        if dist > max_dist {
            max_dist = dist;
            split_point = i;
        }
    }
    (max_dist, split_point)
}

fn compute_left_tangent(points: &[Point], idx: usize) -> Vec2 {
    let tan = points[idx + 1] - points[idx];
    if tan.length() < 1e-12 { Vec2::new(1.0, 0.0) } else { tan.normalize() }
}

fn compute_right_tangent(points: &[Point], idx: usize) -> Vec2 {
    let tan = points[idx - 1] - points[idx];
    if tan.length() < 1e-12 { Vec2::new(-1.0, 0.0) } else { tan.normalize() }
}

fn compute_center_tangent(points: &[Point], idx: usize) -> Vec2 {
    let v1 = points[idx - 1] - points[idx];
    let v2 = points[idx] - points[idx + 1];
    let tan = (v1 + v2) * 0.5;
    if tan.length() < 1e-12 {
        (points[idx + 1] - points[idx - 1]).normalize()
    } else {
        tan.normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Point;

    #[test]
    fn test_two_points_produces_single_cubic() {
        let points = vec![Point::new(0.0, 0.0), Point::new(10.0, 5.0)];
        let path = fit_curve(&points, 1.0);
        let svg = path.to_svg();
        assert!(svg.contains('C'), "Expected cubic bezier in: {svg}");
    }

    #[test]
    fn test_straight_line_points() {
        let points: Vec<Point> = (0..10).map(|i| Point::new(i as f64, 0.0)).collect();
        let path = fit_curve(&points, 1.0);
        assert!(!path.to_svg().is_empty());
    }

    #[test]
    fn test_semicircle_fit() {
        let points: Vec<Point> = (0..=20)
            .map(|i| {
                let t = std::f64::consts::PI * (i as f64) / 20.0;
                Point::new(t.cos() * 100.0, t.sin() * 100.0)
            })
            .collect();
        let path = fit_curve(&points, 4.0);
        let svg = path.to_svg();
        assert!(svg.contains('C'), "Expected cubics for semicircle: {svg}");
        let c_count = svg.matches('C').count();
        assert!(c_count <= 8, "Too many cubics ({c_count}) for semicircle");
    }

    #[test]
    fn test_single_point() {
        let points = vec![Point::new(5.0, 5.0)];
        let path = fit_curve(&points, 1.0);
        assert!(path.to_svg().contains('M'));
    }
}
