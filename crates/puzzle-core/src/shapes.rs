//! Reusable shape constructors for puzzle piece masking and whimsy placement.
//!
//! Each function returns a closed `kurbo::BezPath` that fits within the
//! specified bounding rectangle. Closedness is required by linesweeper for
//! boolean operations — open paths produce silently wrong results.

use kurbo::{BezPath, Point, Vec2};
use std::f64::consts::PI;

/// Cubic-bezier approximation constant for a quarter circle:
/// `4/3 * (√2 − 1) = 4/3 * tan(π/8)`. The control handle length for a
/// quarter-circle arc of radius `r` is `r * KAPPA`.
const KAPPA: f64 = 0.5522847498307936;

/// Threshold below which a polygon corner is treated as straight.
/// Prevents divide-by-zero in the `r / tan(θ/2)` arc-radius formula.
const STRAIGHT_CORNER_EPSILON: f64 = 1e-6;

// ───── Basic shapes ──────────────────────────────────────────────────

/// Rectangle with sharp corners, walking clockwise from the top-left.
/// Bounding box is `(0, 0)` to `(width, height)`.
pub fn rect_path(width: f64, height: f64) -> BezPath {
    let mut path = BezPath::new();
    path.move_to(Point::new(0.0, 0.0));
    path.line_to(Point::new(width, 0.0));
    path.line_to(Point::new(width, height));
    path.line_to(Point::new(0.0, height));
    path.close_path();
    path
}

/// Rectangle with rounded corners. `radius` is clamped to half the
/// shorter side so arcs never overlap. If `radius <= 0`, falls back to
/// sharp `rect_path`.
pub fn rounded_rect_path(width: f64, height: f64, radius: f64) -> BezPath {
    let r = radius.min(width.min(height) / 2.0);
    if r <= 0.0 {
        return rect_path(width, height);
    }
    let h = r * KAPPA;
    let mut path = BezPath::new();
    path.move_to(Point::new(r, 0.0));
    path.line_to(Point::new(width - r, 0.0));
    path.curve_to(
        Point::new(width - r + h, 0.0),
        Point::new(width, r - h),
        Point::new(width, r),
    );
    path.line_to(Point::new(width, height - r));
    path.curve_to(
        Point::new(width, height - r + h),
        Point::new(width - r + h, height),
        Point::new(width - r, height),
    );
    path.line_to(Point::new(r, height));
    path.curve_to(
        Point::new(r - h, height),
        Point::new(0.0, height - r + h),
        Point::new(0.0, height - r),
    );
    path.line_to(Point::new(0.0, r));
    path.curve_to(
        Point::new(0.0, r - h),
        Point::new(r - h, 0.0),
        Point::new(r, 0.0),
    );
    path.close_path();
    path
}

/// Circle (or ellipse) inscribed in the bounding box `(0,0)–(width,height)`,
/// approximated by four cubic Béziers.
pub fn circle_path(width: f64, height: f64) -> BezPath {
    let rx = width / 2.0;
    let ry = height / 2.0;
    let cx = rx;
    let cy = ry;
    let hx = rx * KAPPA;
    let hy = ry * KAPPA;
    let mut path = BezPath::new();
    path.move_to(Point::new(cx, cy - ry));
    path.curve_to(
        Point::new(cx + hx, cy - ry),
        Point::new(cx + rx, cy - hy),
        Point::new(cx + rx, cy),
    );
    path.curve_to(
        Point::new(cx + rx, cy + hy),
        Point::new(cx + hx, cy + ry),
        Point::new(cx, cy + ry),
    );
    path.curve_to(
        Point::new(cx - hx, cy + ry),
        Point::new(cx - rx, cy + hy),
        Point::new(cx - rx, cy),
    );
    path.curve_to(
        Point::new(cx - rx, cy - hy),
        Point::new(cx - hx, cy - ry),
        Point::new(cx, cy - ry),
    );
    path.close_path();
    path
}

// ───── Heart (purely curved, no rounding needed) ─────────────────────

/// Heart shape as a closed cubic-Bézier path, centered in the
/// bounding rectangle `(0, 0)` to `(width, height)`.
pub fn heart_path(width: f64, height: f64) -> BezPath {
    let cx = width / 2.0;

    let mut path = BezPath::new();
    path.move_to(Point::new(cx, height));
    path.curve_to(
        Point::new(cx - width * 0.02, height * 0.65),
        Point::new(0.0, height * 0.55),
        Point::new(0.0, height * 0.35),
    );
    path.curve_to(
        Point::new(0.0, height * 0.1),
        Point::new(cx * 0.6, 0.0),
        Point::new(cx, height * 0.25),
    );
    path.curve_to(
        Point::new(cx + cx * 0.4, 0.0),
        Point::new(width, height * 0.1),
        Point::new(width, height * 0.35),
    );
    path.curve_to(
        Point::new(width, height * 0.55),
        Point::new(cx + width * 0.02, height * 0.65),
        Point::new(cx, height),
    );
    path.close_path();
    path
}

// ───── Rounded polygons ──────────────────────────────────────────────

/// Construct a closed polygon from `vertices`, rounding each corner
/// with a target trim distance `radius` (the distance along each edge
/// that the arc replaces with a smooth curve).
///
/// At each vertex the effective radius is clamped to half the shorter
/// adjacent edge so arcs from neighboring corners never overlap.
///
/// # Panics
///
/// Panics if `vertices.len() < 3`.
pub fn rounded_polygon(vertices: &[Point], radius: f64) -> BezPath {
    assert!(
        vertices.len() >= 3,
        "rounded_polygon requires at least 3 vertices"
    );

    let n = vertices.len();
    let mut path = BezPath::new();

    if radius <= 0.0 {
        path.move_to(vertices[0]);
        for p in &vertices[1..] {
            path.line_to(*p);
        }
        path.close_path();
        return path;
    }

    // Precompute unit edge directions and lengths.
    let mut dirs: Vec<Vec2> = Vec::with_capacity(n);
    let mut lens: Vec<f64> = Vec::with_capacity(n);
    for i in 0..n {
        let a = vertices[i];
        let b = vertices[(i + 1) % n];
        let v = Vec2::new(b.x - a.x, b.y - a.y);
        let len = v.hypot();
        if len > 0.0 {
            dirs.push(Vec2::new(v.x / len, v.y / len));
        } else {
            dirs.push(Vec2::new(1.0, 0.0));
        }
        lens.push(len);
    }

    let mut started = false;
    for i in 0..n {
        let prev_idx = (i + n - 1) % n;
        let d_in = dirs[prev_idx]; // v_{i-1} → v_i
        let d_out = dirs[i]; // v_i → v_{i+1}

        // Effective trim distance: bounded by half each adjacent edge.
        let max_r = lens[prev_idx].min(lens[i]) * 0.5;
        let r = radius.min(max_r);

        let v = vertices[i];
        let dot = (d_in.x * d_out.x + d_in.y * d_out.y).clamp(-1.0, 1.0);
        let theta = dot.acos(); // turn angle (exterior); 0 = straight

        if r <= 0.0 || theta < STRAIGHT_CORNER_EPSILON {
            // Straight corner or no radius available → pass through the vertex.
            if !started {
                path.move_to(v);
                started = true;
            } else {
                path.line_to(v);
            }
            continue;
        }

        let p0 = Point::new(v.x - d_in.x * r, v.y - d_in.y * r);
        let p3 = Point::new(v.x + d_out.x * r, v.y + d_out.y * r);

        // Inscribed arc radius: R = r / tan(θ/2). Handle length for a
        // cubic-bezier approximation of an arc of radius R sweeping θ:
        //   h = R · (4/3) · tan(θ/4)
        let big_r = r / (theta * 0.5).tan();
        let h = big_r * (4.0 / 3.0) * (theta * 0.25).tan();

        let p1 = Point::new(p0.x + d_in.x * h, p0.y + d_in.y * h);
        let p2 = Point::new(p3.x - d_out.x * h, p3.y - d_out.y * h);

        if !started {
            path.move_to(p0);
            started = true;
        } else {
            path.line_to(p0);
        }
        path.curve_to(p1, p2, p3);
    }

    path.close_path();
    path
}

/// Star polygon with `points` outer tips inscribed in the bounding box.
/// Inner radius is 40 % of the outer radius. Corners are softened with
/// `corner_radius` applied uniformly to both tips and valleys.
///
/// # Panics
///
/// Panics if `points < 2`.
pub fn star_path(width: f64, height: f64, points: usize, corner_radius: f64) -> BezPath {
    assert!(points >= 2, "star must have at least 2 points");

    let cx = width / 2.0;
    let cy = height / 2.0;
    let outer_rx = width / 2.0;
    let outer_ry = height / 2.0;
    let inner_rx = outer_rx * 0.4;
    let inner_ry = outer_ry * 0.4;
    let total = points * 2;
    let step = 2.0 * PI / total as f64;
    let start = -PI / 2.0; // first outer vertex at the top

    let vertices: Vec<Point> = (0..total)
        .map(|i| {
            let angle = start + i as f64 * step;
            let (rx, ry) = if i % 2 == 0 {
                (outer_rx, outer_ry)
            } else {
                (inner_rx, inner_ry)
            };
            Point::new(cx + rx * angle.cos(), cy + ry * angle.sin())
        })
        .collect();

    rounded_polygon(&vertices, corner_radius)
}

/// Equilateral-ish triangle (tip up) fitted to the bounding box,
/// optionally rounded at all three corners.
pub fn triangle_path(width: f64, height: f64, corner_radius: f64) -> BezPath {
    let vertices = [
        Point::new(width * 0.5, 0.0),
        Point::new(width, height),
        Point::new(0.0, height),
    ];
    rounded_polygon(&vertices, corner_radius)
}

/// Diamond / rhombus with vertices at each side midpoint of the
/// bounding box, optionally rounded.
pub fn diamond_path(width: f64, height: f64, corner_radius: f64) -> BezPath {
    let vertices = [
        Point::new(width * 0.5, 0.0),
        Point::new(width, height * 0.5),
        Point::new(width * 0.5, height),
        Point::new(0.0, height * 0.5),
    ];
    rounded_polygon(&vertices, corner_radius)
}

/// Flat-top hexagon inscribed in the bounding box, rounded corners.
pub fn hexagon_path(width: f64, height: f64, corner_radius: f64) -> BezPath {
    let vertices = [
        Point::new(width * 0.25, 0.0),
        Point::new(width * 0.75, 0.0),
        Point::new(width, height * 0.5),
        Point::new(width * 0.75, height),
        Point::new(width * 0.25, height),
        Point::new(0.0, height * 0.5),
    ];
    rounded_polygon(&vertices, corner_radius)
}

/// Right-pointing arrow with a rectangular shaft and triangular head.
/// Rounded corners smooth both the convex (shaft corners, tip) and
/// concave (flare notches) points.
pub fn arrow_path(width: f64, height: f64, corner_radius: f64) -> BezPath {
    // Proportions: shaft occupies left 60 %, head occupies right 40 %.
    // Shaft spans vertical 30 %–70 %; flare spans 10 %–90 %.
    let vertices = [
        Point::new(0.0, height * 0.3),
        Point::new(width * 0.6, height * 0.3),
        Point::new(width * 0.6, height * 0.1),
        Point::new(width, height * 0.5),
        Point::new(width * 0.6, height * 0.9),
        Point::new(width * 0.6, height * 0.7),
        Point::new(0.0, height * 0.7),
    ];
    rounded_polygon(&vertices, corner_radius)
}

// ───── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::{PathEl, Shape};

    fn assert_closed(path: &BezPath, name: &str) {
        let els: Vec<PathEl> = path.elements().to_vec();
        assert!(!els.is_empty(), "{name} path must have elements");
        assert_eq!(
            *els.last().unwrap(),
            PathEl::ClosePath,
            "{name} path must end with ClosePath"
        );
    }

    fn assert_fits(path: &BezPath, w: f64, h: f64, tol: f64, name: &str) {
        let bb = path.bounding_box();
        assert!(
            bb.x0 >= -tol && bb.y0 >= -tol && bb.x1 <= w + tol && bb.y1 <= h + tol,
            "{name} bounding box {bb:?} must fit within ({w}, {h}) ± {tol}"
        );
    }

    #[test]
    fn heart_path_is_closed_and_fits() {
        let p = heart_path(100.0, 80.0);
        assert_closed(&p, "heart");
        assert_fits(&p, 100.0, 80.0, 2.0, "heart");
    }

    #[test]
    fn star_path_is_closed_and_fits() {
        let p = star_path(100.0, 100.0, 5, 0.0);
        assert_closed(&p, "star");
        assert_fits(&p, 100.0, 100.0, 1.0, "star");
    }

    #[test]
    fn rounded_star_is_closed_and_fits() {
        let p = star_path(100.0, 100.0, 5, 4.0);
        assert_closed(&p, "rounded star");
        assert_fits(&p, 100.0, 100.0, 2.0, "rounded star");
    }

    #[test]
    fn rounded_rect_is_closed_and_fits() {
        let p = rounded_rect_path(100.0, 60.0, 12.0);
        assert_closed(&p, "rounded_rect");
        assert_fits(&p, 100.0, 60.0, 0.5, "rounded_rect");
    }

    #[test]
    fn rounded_rect_zero_radius_equals_rect() {
        let a = rect_path(80.0, 50.0);
        let b = rounded_rect_path(80.0, 50.0, 0.0);
        assert_eq!(a.elements().len(), b.elements().len());
    }

    #[test]
    fn circle_is_closed_and_fits() {
        let p = circle_path(100.0, 60.0);
        assert_closed(&p, "circle");
        assert_fits(&p, 100.0, 60.0, 0.5, "circle");
        // Center should be inside the shape.
        assert!(p.winding(Point::new(50.0, 30.0)) != 0);
    }

    #[test]
    fn triangle_is_closed_and_fits() {
        let p = triangle_path(100.0, 100.0, 8.0);
        assert_closed(&p, "triangle");
        assert_fits(&p, 100.0, 100.0, 0.5, "triangle");
        // Centroid is inside the triangle.
        assert!(p.winding(Point::new(50.0, 66.0)) != 0);
    }

    #[test]
    fn diamond_is_closed_and_fits() {
        let p = diamond_path(100.0, 60.0, 6.0);
        assert_closed(&p, "diamond");
        assert_fits(&p, 100.0, 60.0, 0.5, "diamond");
        assert!(p.winding(Point::new(50.0, 30.0)) != 0);
    }

    #[test]
    fn hexagon_is_closed_and_fits() {
        let p = hexagon_path(100.0, 100.0, 6.0);
        assert_closed(&p, "hexagon");
        assert_fits(&p, 100.0, 100.0, 0.5, "hexagon");
        assert!(p.winding(Point::new(50.0, 50.0)) != 0);
    }

    #[test]
    fn arrow_is_closed_and_fits() {
        let p = arrow_path(120.0, 80.0, 3.0);
        assert_closed(&p, "arrow");
        assert_fits(&p, 120.0, 80.0, 0.5, "arrow");
        // A point well inside the shaft should be inside the shape.
        assert!(p.winding(Point::new(20.0, 40.0)) != 0);
    }

    #[test]
    fn rounded_polygon_zero_radius_is_plain_polygon() {
        let verts = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];
        let p = rounded_polygon(&verts, 0.0);
        let line_count = p
            .elements()
            .iter()
            .filter(|el| matches!(el, PathEl::LineTo(_)))
            .count();
        assert_eq!(line_count, 3); // first is MoveTo, last three are LineTo
    }

    #[test]
    fn rounded_polygon_small_radius_adds_curves() {
        let verts = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];
        let p = rounded_polygon(&verts, 2.0);
        let curve_count = p
            .elements()
            .iter()
            .filter(|el| matches!(el, PathEl::CurveTo(..)))
            .count();
        assert_eq!(curve_count, 4, "one rounded arc per corner");
    }
}
