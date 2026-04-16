//! Reusable shape constructors for puzzle piece masking.
//!
//! Each function returns a closed `kurbo::BezPath` that fits within the
//! specified bounding rectangle. Closedness is required by linesweeper for
//! boolean operations — open paths produce silently wrong results.

use kurbo::{BezPath, Point};
use std::f64::consts::PI;

/// Construct a heart shape as a closed cubic-Bézier path.
///
/// The heart is centered within the rectangle `(0, 0)` to `(width, height)`.
/// The top has two symmetric bumps and the bottom comes to a point.
pub fn heart_path(width: f64, height: f64) -> BezPath {
    let cx = width / 2.0;

    // Key reference points:
    //   bottom tip   = (cx, height)
    //   top dip      = (cx, height * 0.3)
    //   left peak    = (width * 0.25, 0)
    //   right peak   = (width * 0.75, 0)
    //   leftmost     = (0, height * 0.3)
    //   rightmost    = (width, height * 0.3)

    let mut path = BezPath::new();

    // Start at the bottom tip
    path.move_to(Point::new(cx, height));

    // Curve from bottom tip up to the top-dip (left side)
    // Left half of heart: bottom → left bulge → left peak → center dip
    path.curve_to(
        Point::new(cx - width * 0.02, height * 0.65),  // cp1: pulls left slightly
        Point::new(0.0, height * 0.55),                 // cp2: leftmost influence
        Point::new(0.0, height * 0.35),                 // end: left mid-height
    );

    // Left bump: from left mid up to the left peak and toward center
    path.curve_to(
        Point::new(0.0, height * 0.1),                  // cp1: up toward top-left
        Point::new(cx * 0.6, 0.0),                      // cp2: peak control
        Point::new(cx, height * 0.25),                   // end: center dip
    );

    // Right bump: from center dip to the right peak
    path.curve_to(
        Point::new(cx + cx * 0.4, 0.0),                 // cp1: peak control (right)
        Point::new(width, height * 0.1),                 // cp2: up toward top-right
        Point::new(width, height * 0.35),                // end: right mid-height
    );

    // Right side down to bottom tip
    path.curve_to(
        Point::new(width, height * 0.55),                // cp1: rightmost influence
        Point::new(cx + width * 0.02, height * 0.65),   // cp2: pulls right slightly
        Point::new(cx, height),                           // end: bottom tip
    );

    path.close_path();
    path
}

/// Construct a star polygon as a closed path of line segments.
///
/// The star has `points` outer vertices and `points` inner vertices,
/// alternating around the center of the bounding box `(0, 0)` to
/// `(width, height)`. The outer radius fills the bounding box; the inner
/// radius is 40 % of the outer radius.
///
/// # Panics
///
/// Panics if `points < 2`.
pub fn star_path(width: f64, height: f64, points: usize) -> BezPath {
    assert!(points >= 2, "star must have at least 2 points");

    let cx = width / 2.0;
    let cy = height / 2.0;
    let outer_rx = width / 2.0;
    let outer_ry = height / 2.0;
    let inner_rx = outer_rx * 0.4;
    let inner_ry = outer_ry * 0.4;
    let total_vertices = points * 2;
    let angle_step = 2.0 * PI / total_vertices as f64;
    // Start at -PI/2 so the first outer vertex is at the top
    let start_angle = -PI / 2.0;

    let mut path = BezPath::new();

    for i in 0..total_vertices {
        let angle = start_angle + i as f64 * angle_step;
        let (rx, ry) = if i % 2 == 0 {
            (outer_rx, outer_ry) // outer vertex
        } else {
            (inner_rx, inner_ry) // inner vertex
        };
        let x = cx + rx * angle.cos();
        let y = cy + ry * angle.sin();

        if i == 0 {
            path.move_to(Point::new(x, y));
        } else {
            path.line_to(Point::new(x, y));
        }
    }

    path.close_path();
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::{PathEl, Shape};

    #[test]
    fn test_heart_path_is_closed() {
        let path = heart_path(100.0, 100.0);
        let elements: Vec<PathEl> = path.elements().to_vec();
        assert!(!elements.is_empty(), "heart path must have elements");
        assert_eq!(
            *elements.last().unwrap(),
            PathEl::ClosePath,
            "heart path must end with ClosePath"
        );
    }

    #[test]
    fn test_heart_path_bounding_box() {
        let w = 100.0;
        let h = 80.0;
        let path = heart_path(w, h);
        let bb = path.bounding_box();
        // Allow a small tolerance for control-point overshoot
        let tol = 2.0;
        assert!(
            bb.x0 >= -tol && bb.y0 >= -tol && bb.x1 <= w + tol && bb.y1 <= h + tol,
            "heart bounding box {bb:?} must fit within ({w}, {h}) ± {tol}"
        );
    }

    #[test]
    fn test_star_path_is_closed() {
        let path = star_path(100.0, 100.0, 5);
        let elements: Vec<PathEl> = path.elements().to_vec();
        assert!(!elements.is_empty(), "star path must have elements");
        assert_eq!(
            *elements.last().unwrap(),
            PathEl::ClosePath,
            "star path must end with ClosePath"
        );
    }

    #[test]
    fn test_star_path_bounding_box() {
        let w = 100.0;
        let h = 100.0;
        let path = star_path(w, h, 5);
        let bb = path.bounding_box();
        let tol = 1.0;
        assert!(
            bb.x0 >= -tol && bb.y0 >= -tol && bb.x1 <= w + tol && bb.y1 <= h + tol,
            "star bounding box {bb:?} must fit within ({w}, {h}) ± {tol}"
        );
    }

    #[test]
    fn test_star_path_point_count() {
        let path = star_path(100.0, 100.0, 5);
        let elements: Vec<PathEl> = path.elements().to_vec();
        // A 5-pointed star has 10 vertices:
        //   1 MoveTo + 9 LineTo + 1 ClosePath = 11 elements total
        let line_count = elements
            .iter()
            .filter(|el| matches!(el, PathEl::LineTo(_)))
            .count();
        assert_eq!(
            line_count, 9,
            "5-pointed star should have 9 LineTo segments (first vertex is MoveTo)"
        );
    }
}
