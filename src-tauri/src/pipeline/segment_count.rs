use kurbo::{BezPath, PathEl};

/// Count the number of drawing segments (L, Q, C commands) in an SVG path string.
/// MoveTo and ClosePath are not counted as segments.
pub fn count_segments(svg_path: &str) -> usize {
    let Ok(bez) = BezPath::from_svg(svg_path) else {
        return 0;
    };
    bez.elements()
        .iter()
        .filter(|el| matches!(el, PathEl::LineTo(_) | PathEl::QuadTo(_, _) | PathEl::CurveTo(_, _, _)))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_simple_rect() {
        // M0,0 L100,0 L100,100 L0,100 Z — 3 LineTo elements; Z (ClosePath) is not counted
        let path = "M0,0 L100,0 L100,100 L0,100 Z";
        assert_eq!(count_segments(path), 3);
    }

    #[test]
    fn test_count_cubics() {
        let path = "M0,0 C10,20 30,40 50,0 C70,-40 90,-20 100,0";
        assert_eq!(count_segments(path), 2);
    }

    #[test]
    fn test_count_empty() {
        assert_eq!(count_segments(""), 0);
    }

    #[test]
    fn test_count_move_only() {
        assert_eq!(count_segments("M0,0"), 0);
    }

    #[test]
    fn test_count_mixed() {
        let path = "M0,0 L50,0 C60,10 90,10 100,0 L100,100";
        assert_eq!(count_segments(path), 3);
    }
}
