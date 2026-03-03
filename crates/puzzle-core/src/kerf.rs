use kurbo::{BezPath, PathEl, Point, Vec2};

/// Offset all paths outward by `kerf_width / 2.0` for kerf compensation.
///
/// When a laser cuts material, it removes a thin strip (the kerf). By
/// offsetting cut paths outward by half the kerf width, the resulting
/// pieces fit together with correct dimensions.
///
/// The offset is computed by:
/// 1. Flattening curves to line segments (tolerance 0.01 mm)
/// 2. Computing outward normals for each segment
/// 3. Offsetting endpoints along normals by `kerf_width / 2.0`
/// 4. Using miter joins between adjacent segments (bevel if miter ratio > 2.0)
///
/// If `kerf_width <= 0.0`, returns a clone of the original path unchanged.
pub fn offset_path(path: &BezPath, kerf_width: f64) -> BezPath {
    if kerf_width <= 0.0 {
        return path.clone();
    }

    let offset = kerf_width / 2.0;

    // Flatten to polylines
    let mut subpaths: Vec<Vec<Point>> = Vec::new();
    let mut current_subpath: Vec<Point> = Vec::new();
    let mut is_closed: Vec<bool> = Vec::new();

    kurbo::flatten(path.iter(), 0.01, |el| match el {
        PathEl::MoveTo(p) => {
            if !current_subpath.is_empty() {
                subpaths.push(std::mem::take(&mut current_subpath));
                is_closed.push(false);
            }
            current_subpath.push(p);
        }
        PathEl::LineTo(p) => {
            current_subpath.push(p);
        }
        PathEl::ClosePath => {
            if !current_subpath.is_empty() {
                subpaths.push(std::mem::take(&mut current_subpath));
                is_closed.push(true);
            }
        }
        _ => {} // flatten should only emit MoveTo, LineTo, ClosePath
    });

    // Push final subpath if not closed
    if !current_subpath.is_empty() {
        subpaths.push(current_subpath);
        is_closed.push(false);
    }

    let mut result = BezPath::new();

    for (sub_idx, pts) in subpaths.iter().enumerate() {
        if pts.len() < 2 {
            // Single point, can't offset
            if let Some(&p) = pts.first() {
                result.move_to(p);
            }
            continue;
        }

        let closed = is_closed[sub_idx];
        let n = pts.len();

        // Compute segment normals (outward = left side for the path direction)
        let normals: Vec<Vec2> = (0..n - 1)
            .map(|i| {
                let dx = pts[i + 1].x - pts[i].x;
                let dy = pts[i + 1].y - pts[i].y;
                let len = (dx * dx + dy * dy).sqrt();
                if len < 1e-12 {
                    Vec2::ZERO
                } else {
                    // Left-side normal (outward for clockwise paths)
                    Vec2::new(dy / len, -dx / len)
                }
            })
            .collect();

        if normals.is_empty() {
            continue;
        }

        // Generate offset points with miter joins
        let mut offset_pts: Vec<Point> = Vec::with_capacity(n);

        for i in 0..n {
            let is_first = i == 0;
            let is_last = i == n - 1;

            if closed {
                // For closed paths, every vertex has two adjacent segments
                let prev_idx = if is_first { normals.len() - 1 } else { i - 1 };
                let next_idx = if is_last { 0 } else { i };

                if next_idx < normals.len() && prev_idx < normals.len() {
                    let avg_normal = miter_normal(&normals[prev_idx], &normals[next_idx], offset);
                    offset_pts.push(Point::new(pts[i].x + avg_normal.x, pts[i].y + avg_normal.y));
                } else {
                    offset_pts.push(Point::new(
                        pts[i].x + normals[0].x * offset,
                        pts[i].y + normals[0].y * offset,
                    ));
                }
            } else if is_first {
                // Open path: first point uses first segment normal
                offset_pts.push(Point::new(
                    pts[i].x + normals[0].x * offset,
                    pts[i].y + normals[0].y * offset,
                ));
            } else if is_last {
                // Open path: last point uses last segment normal
                let last_n = &normals[normals.len() - 1];
                offset_pts.push(Point::new(
                    pts[i].x + last_n.x * offset,
                    pts[i].y + last_n.y * offset,
                ));
            } else {
                // Interior point: miter join between two adjacent segments
                let avg_normal = miter_normal(&normals[i - 1], &normals[i], offset);
                offset_pts.push(Point::new(pts[i].x + avg_normal.x, pts[i].y + avg_normal.y));
            }
        }

        // Emit offset polyline
        if let Some(&first) = offset_pts.first() {
            result.move_to(first);
            for &p in &offset_pts[1..] {
                result.line_to(p);
            }
            if closed {
                result.close_path();
            }
        }
    }

    result
}

/// Compute miter join offset vector for the junction of two segments.
///
/// If the miter ratio exceeds 2.0, falls back to bevel (average of normals).
fn miter_normal(n1: &Vec2, n2: &Vec2, offset: f64) -> Vec2 {
    let avg = Vec2::new((n1.x + n2.x) / 2.0, (n1.y + n2.y) / 2.0);
    let avg_len = avg.hypot();

    if avg_len < 1e-12 {
        // Normals cancel out (180° turn) — use first normal
        return Vec2::new(n1.x * offset, n1.y * offset);
    }

    let miter_ratio = 1.0 / avg_len;

    if miter_ratio > 2.0 {
        // Miter too acute — use bevel (just average direction, unit length * offset)
        Vec2::new(avg.x / avg_len * offset, avg.y / avg_len * offset)
    } else {
        // Miter join: scale average to produce correct offset distance
        Vec2::new(
            avg.x / (avg_len * avg_len) * offset,
            avg.y / (avg_len * avg_len) * offset,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a simple square path (clockwise: TR → BR → BL → TL).
    fn square_path(size: f64) -> BezPath {
        let mut path = BezPath::new();
        path.move_to(Point::new(0.0, 0.0));
        path.line_to(Point::new(size, 0.0));
        path.line_to(Point::new(size, size));
        path.line_to(Point::new(0.0, size));
        path.close_path();
        path
    }

    #[test]
    fn test_zero_kerf_returns_original() {
        let path = square_path(100.0);
        let result = offset_path(&path, 0.0);
        assert_eq!(path, result, "zero kerf should return identical path");
    }

    #[test]
    fn test_negative_kerf_returns_original() {
        let path = square_path(100.0);
        let result = offset_path(&path, -0.5);
        assert_eq!(path, result, "negative kerf should return identical path");
    }

    #[test]
    fn test_positive_kerf_offsets_outward() {
        let path = square_path(100.0);
        let kerf = 0.2;
        let result = offset_path(&path, kerf);

        // The offset path should be larger than the original.
        // Extract points from the offset path.
        let mut offset_points: Vec<Point> = Vec::new();
        for el in result.iter() {
            match el {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => offset_points.push(p),
                _ => {}
            }
        }

        // For a clockwise square, left-side normals point outward.
        // With the normals we computed (dy/len, -dx/len):
        // Top edge (0,0)→(100,0): normal = (0, 0) wait... let me check.
        // dx=100, dy=0, normal = (0/100, -100/100) = (0, -1) ← upward
        // That means offset goes upward (y decreases for top edge).
        // The offset square should have corners outside the original.

        // Check that some dimension is larger:
        // Find min/max x and y of offset points
        let min_x = offset_points
            .iter()
            .map(|p| p.x)
            .fold(f64::INFINITY, f64::min);
        let max_x = offset_points
            .iter()
            .map(|p| p.x)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = offset_points
            .iter()
            .map(|p| p.y)
            .fold(f64::INFINITY, f64::min);
        let max_y = offset_points
            .iter()
            .map(|p| p.y)
            .fold(f64::NEG_INFINITY, f64::max);

        let offset_width = max_x - min_x;
        let offset_height = max_y - min_y;

        assert!(
            offset_width > 100.0,
            "offset width {} should exceed original 100.0",
            offset_width
        );
        assert!(
            offset_height > 100.0,
            "offset height {} should exceed original 100.0",
            offset_height
        );
    }

    #[test]
    fn test_kerf_preserves_subpath_structure() {
        // Create two separate subpaths
        let mut path = BezPath::new();
        // Subpath 1: open line
        path.move_to(Point::new(0.0, 0.0));
        path.line_to(Point::new(100.0, 0.0));
        // Subpath 2: open line
        path.move_to(Point::new(0.0, 50.0));
        path.line_to(Point::new(100.0, 50.0));

        let result = offset_path(&path, 0.2);

        // Count MoveTo commands — should still be 2 (two separate subpaths)
        let move_count = result
            .iter()
            .filter(|el| matches!(el, PathEl::MoveTo(_)))
            .count();
        assert_eq!(
            move_count, 2,
            "offset should preserve 2 separate subpaths, got {}",
            move_count
        );
    }
}
