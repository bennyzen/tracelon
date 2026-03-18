use kurbo::BezPath;
use kurbo::simplify::{SimplifyOptions, SimplifyOptLevel, simplify_bezpath};

/// Simplify an SVG path string using kurbo's built-in Bezier simplification.
/// Works directly on curves — no lossy flatten-to-polyline step.
/// `smoothness` is 0.0-1.0.
pub fn simplify_svg_path(svg_path: &str, smoothness: f64) -> Result<String, String> {
    if smoothness < 0.01 {
        return Ok(svg_path.to_string());
    }

    let bez = BezPath::from_svg(svg_path).map_err(|e| format!("Invalid SVG path: {e}"))?;

    // Map smoothness to accuracy (max error in path units/pixels).
    // Use quadratic curve so low values have very gentle effect:
    // 0.01 → 0.1 px, 0.1 → 0.2 px, 0.5 → 2.1 px, 1.0 → 8.0 px
    let accuracy = 0.1 + smoothness * smoothness * 7.9;

    let options = SimplifyOptions::default()
        .opt_level(SimplifyOptLevel::Optimize);

    let simplified = simplify_bezpath(bez.iter(), accuracy, &options);

    Ok(simplified.to_svg())
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
    fn test_smoothness_zero_returns_original() {
        let input = "M0,0 C10,20 30,40 50,0 C70,-40 90,-20 100,0";
        let result = simplify_svg_path(input, 0.0).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_smoothness_produces_valid_svg() {
        let input = "M0,0 C10,20 30,40 50,0 C70,-40 90,-20 100,0";
        let result = simplify_svg_path(input, 0.5).unwrap();
        // Should be valid SVG path
        assert!(result.starts_with('M'));
    }
}
