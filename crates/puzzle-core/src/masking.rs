//! Boolean operation wrappers for shape masking.
//!
//! Provides thin wrappers around [`linesweeper::binary_op`] for intersection
//! (mask) and difference (reverse-mask) operations on closed `BezPath`s.
//! These are the core geometric primitives consumed by grid clipping, border
//! masking, and whimsy placement in downstream slices.
//!
//! Both input paths **must be closed** (end with `ClosePath`) — open paths
//! produce silently wrong results from the linesweeper backend.

use kurbo::{BezPath, PathEl};
use linesweeper::{binary_op, BinaryOp, FillRule};

/// Compute the intersection of two closed paths (the region inside **both**).
///
/// Returns a single `BezPath` that may contain multiple subpaths when the
/// intersection produces disjoint contour regions.
pub fn mask_intersection(base: &BezPath, shape: &BezPath) -> Result<BezPath, String> {
    boolean_op(base, shape, BinaryOp::Intersection)
}

/// Compute the difference of two closed paths (the region inside `base` but
/// **not** inside `shape`).
///
/// Returns a single `BezPath` that may contain multiple subpaths when the
/// difference produces disjoint contour regions.
pub fn mask_difference(base: &BezPath, shape: &BezPath) -> Result<BezPath, String> {
    boolean_op(base, shape, BinaryOp::Difference)
}

/// Shared implementation: run `binary_op` with `EvenOdd` fill rule, then
/// concatenate all resulting contours into a single `BezPath`.
fn boolean_op(base: &BezPath, shape: &BezPath, op: BinaryOp) -> Result<BezPath, String> {
    let contours = binary_op(base, shape, FillRule::EvenOdd, op).map_err(|e| e.to_string())?;

    let mut result = BezPath::new();
    for contour in contours.contours() {
        for el in contour.path.iter() {
            match el {
                PathEl::MoveTo(p) => result.move_to(p),
                PathEl::LineTo(p) => result.line_to(p),
                PathEl::QuadTo(p1, p2) => result.quad_to(p1, p2),
                PathEl::CurveTo(p1, p2, p3) => result.curve_to(p1, p2, p3),
                PathEl::ClosePath => result.close_path(),
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::{heart_path, star_path};
    use kurbo::{Point, Shape};

    /// Build a closed rectangle BezPath from (x, y) to (x+w, y+h).
    fn rect_path(x: f64, y: f64, w: f64, h: f64) -> BezPath {
        let mut path = BezPath::new();
        path.move_to(Point::new(x, y));
        path.line_to(Point::new(x + w, y));
        path.line_to(Point::new(x + w, y + h));
        path.line_to(Point::new(x, y + h));
        path.close_path();
        path
    }

    #[test]
    fn test_intersection_heart_and_rect() {
        let rect = rect_path(0.0, 0.0, 200.0, 200.0);
        // Heart centered inside the rectangle (offset by 50,50 so it sits in the middle)
        let heart = heart_path(100.0, 100.0);
        // Translate heart to center: shift by (50, 50)
        let mut shifted_heart = BezPath::new();
        for el in heart.iter() {
            match el {
                PathEl::MoveTo(p) => shifted_heart.move_to(Point::new(p.x + 50.0, p.y + 50.0)),
                PathEl::LineTo(p) => shifted_heart.line_to(Point::new(p.x + 50.0, p.y + 50.0)),
                PathEl::CurveTo(p1, p2, p3) => shifted_heart.curve_to(
                    Point::new(p1.x + 50.0, p1.y + 50.0),
                    Point::new(p2.x + 50.0, p2.y + 50.0),
                    Point::new(p3.x + 50.0, p3.y + 50.0),
                ),
                PathEl::QuadTo(p1, p2) => shifted_heart.quad_to(
                    Point::new(p1.x + 50.0, p1.y + 50.0),
                    Point::new(p2.x + 50.0, p2.y + 50.0),
                ),
                PathEl::ClosePath => shifted_heart.close_path(),
            }
        }

        let result = mask_intersection(&rect, &shifted_heart).expect("intersection should succeed");
        assert!(
            !result.elements().is_empty(),
            "intersection of overlapping shapes must be non-empty"
        );

        let result_bb = result.bounding_box();
        let rect_bb = rect.bounding_box();
        assert!(
            result_bb.width() < rect_bb.width() || result_bb.height() < rect_bb.height(),
            "intersection bounding box ({:?}) should be smaller than rectangle ({:?})",
            result_bb,
            rect_bb,
        );
    }

    #[test]
    fn test_difference_rect_minus_star() {
        let rect = rect_path(0.0, 0.0, 200.0, 200.0);
        // Star centered inside the rectangle (offset by 60,60)
        let star = star_path(80.0, 80.0, 5, 0.0);
        let mut shifted_star = BezPath::new();
        for el in star.iter() {
            match el {
                PathEl::MoveTo(p) => shifted_star.move_to(Point::new(p.x + 60.0, p.y + 60.0)),
                PathEl::LineTo(p) => shifted_star.line_to(Point::new(p.x + 60.0, p.y + 60.0)),
                PathEl::CurveTo(p1, p2, p3) => shifted_star.curve_to(
                    Point::new(p1.x + 60.0, p1.y + 60.0),
                    Point::new(p2.x + 60.0, p2.y + 60.0),
                    Point::new(p3.x + 60.0, p3.y + 60.0),
                ),
                PathEl::QuadTo(p1, p2) => shifted_star.quad_to(
                    Point::new(p1.x + 60.0, p1.y + 60.0),
                    Point::new(p2.x + 60.0, p2.y + 60.0),
                ),
                PathEl::ClosePath => shifted_star.close_path(),
            }
        }

        let result = mask_difference(&rect, &shifted_star).expect("difference should succeed");
        assert!(
            !result.elements().is_empty(),
            "difference of overlapping shapes must be non-empty"
        );

        // Bounding box of the result should be close to the rectangle
        // (we're cutting a hole inside, not removing from the edges)
        let result_bb = result.bounding_box();
        let rect_bb = rect.bounding_box();
        let tol = 1.0;
        assert!(
            (result_bb.x0 - rect_bb.x0).abs() < tol
                && (result_bb.y0 - rect_bb.y0).abs() < tol
                && (result_bb.x1 - rect_bb.x1).abs() < tol
                && (result_bb.y1 - rect_bb.y1).abs() < tol,
            "difference bounding box ({:?}) should match rectangle ({:?}) within {tol}",
            result_bb,
            rect_bb,
        );
    }

    #[test]
    fn test_intersection_deterministic() {
        let rect = rect_path(0.0, 0.0, 200.0, 200.0);
        let heart = heart_path(100.0, 100.0);

        let result1 = mask_intersection(&rect, &heart)
            .expect("first intersection should succeed");
        let result2 = mask_intersection(&rect, &heart)
            .expect("second intersection should succeed");

        let svg1 = result1.to_svg();
        let svg2 = result2.to_svg();
        assert_eq!(
            svg1, svg2,
            "identical inputs must produce identical SVG output"
        );
    }

    #[test]
    fn test_no_overlap_intersection_empty() {
        let rect_a = rect_path(0.0, 0.0, 100.0, 100.0);
        let rect_b = rect_path(500.0, 500.0, 100.0, 100.0);

        let result = mask_intersection(&rect_a, &rect_b)
            .expect("non-overlapping intersection should succeed");
        assert!(
            result.elements().is_empty(),
            "intersection of non-overlapping shapes must be empty, got {} elements",
            result.elements().len()
        );
    }
}
