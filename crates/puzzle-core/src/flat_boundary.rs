//! Fast point-in-region test against a pre-flattened polygon-with-holes
//! representation.
//!
//! `BezPath::winding(p)` walks the entire path on every call and pays
//! curve-evaluation overhead even when the path has already been
//! converted to lines. For hot loops that test the same boundary
//! against many points (CVT seed scatter, Lloyd relaxation, knob
//! safety probes), it's much cheaper to flatten the boundary once to
//! `Vec<Vec<Point>>` (which the CVT pipeline already does for its
//! linesweeper clip path) and ray-cast against the polylines directly.
//!
//! Even-odd ray casting matches `BezPath::winding(p) != 0` for our
//! shapes: outer boundary CCW + holes (whimsies) CW means crossings
//! flip inside/outside correctly across both orientations.
//!
//! For very hot loops (Lloyd's per-cell vertex tests at 1000+ pieces),
//! [`BoundaryIndex`] adds a Y-bucket spatial index over the boundary
//! segments. Ray-cast queries then visit only the segments whose Y
//! range covers the query point's Y, dropping per-call cost from
//! O(boundary_segments) to O(boundary_segments / num_buckets) — about
//! 10–60× fewer segment-checks for our typical boundaries.

use kurbo::{BezPath, PathEl, Point};

/// Flatten a closed `BezPath` to per-subpath polyline vertices at the
/// given tolerance. Drops the trailing duplicate-of-first vertex that
/// `close_path` produces, so each subpath's edges are
/// `(sub[i], sub[(i + 1) % n])`.
pub fn flatten_polygon(path: &BezPath, tolerance: f64) -> Vec<Vec<Point>> {
    let mut subpaths: Vec<Vec<Point>> = Vec::new();
    let mut current: Vec<Point> = Vec::new();
    kurbo::flatten(path.iter(), tolerance, |el| match el {
        PathEl::MoveTo(p) => {
            if !current.is_empty() {
                subpaths.push(std::mem::take(&mut current));
            }
            current.push(p);
        }
        PathEl::LineTo(p) => current.push(p),
        _ => {}
    });
    if !current.is_empty() {
        subpaths.push(current);
    }
    for sub in &mut subpaths {
        if sub.len() >= 2 && (sub[sub.len() - 1] - sub[0]).hypot() < 1e-6 {
            sub.pop();
        }
    }
    subpaths
}

/// Even-odd ray casting against polyline subpaths. Returns true when
/// `p` is inside the polygon-with-holes — equivalent to
/// `BezPath::winding(p) != 0` for boundaries with consistent
/// outer-CCW / hole-CW orientation, but with no curve-evaluation
/// overhead and no virtual-dispatch tax.
///
/// Caller is responsible for the bounding-box early-out if a cheap
/// rejection helps the workload (most callers test points already
/// clamped to the bbox).
pub fn polygon_contains(subpaths: &[Vec<Point>], p: Point) -> bool {
    let mut inside = false;
    for sub in subpaths {
        let n = sub.len();
        if n < 3 {
            continue;
        }
        let mut j = n - 1;
        for i in 0..n {
            let pi = sub[i];
            let pj = sub[j];
            if (pi.y > p.y) != (pj.y > p.y) {
                let x_inter =
                    pj.x + (p.y - pj.y) * (pi.x - pj.x) / (pi.y - pj.y);
                if p.x < x_inter {
                    inside = !inside;
                }
            }
            j = i;
        }
    }
    inside
}

/// Y-bucket spatial index over polygon edges, speeding up
/// [`polygon_contains_indexed`] queries from O(N) to ~O(N / buckets)
/// per call. Build once when the boundary changes, reuse across the
/// thousands of point-in-polygon tests Lloyd does per generation.
///
/// The index records, for each Y-bucket, the list of edges whose
/// vertical extent overlaps that bucket. A ray-cast at query Y only
/// needs to check edges in that one bucket.
///
/// X-direction queries are unaffected — we don't index by X because
/// horizontal ray casting already does its own X-ordering work.
pub struct BoundaryIndex {
    bbox_min_y: f64,
    bbox_max_y: f64,
    /// Reciprocal of bucket height, precomputed so the per-query
    /// bucket lookup is `(y - min_y) * inv_bucket_height`.
    inv_bucket_height: f64,
    /// One slot per bucket. Each entry is `(subpath_idx, edge_start_idx)`
    /// — the edge runs from `subpaths[spi][si]` to
    /// `subpaths[spi][(si + 1) % subpaths[spi].len()]`.
    buckets: Vec<Vec<(u32, u32)>>,
}

impl BoundaryIndex {
    /// Build a Y-bucket index over `subpaths`. Bucket count scales
    /// with edge count (capped at 256) so each bucket holds a handful
    /// of edges on average.
    pub fn new(subpaths: &[Vec<Point>]) -> Self {
        let mut total_edges = 0usize;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for sub in subpaths {
            if sub.len() < 3 {
                continue;
            }
            total_edges += sub.len();
            for p in sub {
                if p.y < min_y {
                    min_y = p.y;
                }
                if p.y > max_y {
                    max_y = p.y;
                }
            }
        }
        // Aim for ~4 edges per bucket on average. Empty boundaries
        // get a single bucket so contains() is well-defined.
        let num_buckets = ((total_edges / 4).max(1)).min(256);
        let bbox_height = (max_y - min_y).max(1e-9);
        let inv_bucket_height = num_buckets as f64 / bbox_height;
        let mut buckets: Vec<Vec<(u32, u32)>> = vec![Vec::new(); num_buckets];

        for (spi, sub) in subpaths.iter().enumerate() {
            let n = sub.len();
            if n < 3 {
                continue;
            }
            for si in 0..n {
                let pi = sub[si];
                let pj = sub[(si + 1) % n];
                let lo = pi.y.min(pj.y);
                let hi = pi.y.max(pj.y);
                let lo_b = (((lo - min_y) * inv_bucket_height).floor() as isize)
                    .clamp(0, num_buckets as isize - 1) as usize;
                let hi_b = (((hi - min_y) * inv_bucket_height).floor() as isize)
                    .clamp(0, num_buckets as isize - 1) as usize;
                let entry = (spi as u32, si as u32);
                for b in lo_b..=hi_b {
                    buckets[b].push(entry);
                }
            }
        }

        Self {
            bbox_min_y: min_y,
            bbox_max_y: max_y,
            inv_bucket_height,
            buckets,
        }
    }
}

/// Y-bucket-accelerated point-in-polygon test. Equivalent to
/// [`polygon_contains`] but only visits boundary edges whose Y-range
/// covers `p.y`. Use whenever you have a [`BoundaryIndex`] handy and
/// are calling many tests against the same boundary.
pub fn polygon_contains_indexed(
    subpaths: &[Vec<Point>],
    index: &BoundaryIndex,
    p: Point,
) -> bool {
    if p.y < index.bbox_min_y || p.y > index.bbox_max_y {
        return false;
    }
    let bucket_idx =
        (((p.y - index.bbox_min_y) * index.inv_bucket_height).floor() as isize)
            .clamp(0, index.buckets.len() as isize - 1) as usize;
    let mut inside = false;
    for &(spi, si) in &index.buckets[bucket_idx] {
        let sub = &subpaths[spi as usize];
        let n = sub.len();
        let pi = sub[si as usize];
        let pj = sub[(si as usize + 1) % n];
        if (pi.y > p.y) != (pj.y > p.y) {
            let x_inter = pj.x + (p.y - pj.y) * (pi.x - pj.x) / (pi.y - pj.y);
            if p.x < x_inter {
                inside = !inside;
            }
        }
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::{Rect, Shape};

    #[test]
    fn matches_kurbo_winding_on_rect() {
        let rect: BezPath = Rect::new(0.0, 0.0, 100.0, 50.0).into_path(0.1);
        let subs = flatten_polygon(&rect, 0.25);
        for i in 0..50 {
            let x = (i as f64) * 2.5 - 10.0;
            for j in 0..30 {
                let y = (j as f64) * 2.0 - 5.0;
                let p = Point::new(x, y);
                let kurbo_inside = rect.winding(p) != 0;
                assert_eq!(
                    polygon_contains(&subs, p),
                    kurbo_inside,
                    "mismatch at {:?}",
                    p,
                );
            }
        }
    }

    #[test]
    fn indexed_matches_unindexed_on_rect() {
        let rect: BezPath = Rect::new(0.0, 0.0, 100.0, 50.0).into_path(0.1);
        let subs = flatten_polygon(&rect, 0.25);
        let idx = BoundaryIndex::new(&subs);
        for i in 0..50 {
            let x = (i as f64) * 2.5 - 10.0;
            for j in 0..30 {
                let y = (j as f64) * 2.0 - 5.0;
                let p = Point::new(x, y);
                assert_eq!(
                    polygon_contains_indexed(&subs, &idx, p),
                    polygon_contains(&subs, p),
                    "indexed/unindexed mismatch at {:?}",
                    p,
                );
            }
        }
    }

    #[test]
    fn indexed_matches_unindexed_with_hole() {
        let mut path = BezPath::new();
        path.move_to(Point::new(0.0, 0.0));
        path.line_to(Point::new(100.0, 0.0));
        path.line_to(Point::new(100.0, 100.0));
        path.line_to(Point::new(0.0, 100.0));
        path.close_path();
        path.move_to(Point::new(30.0, 30.0));
        path.line_to(Point::new(30.0, 70.0));
        path.line_to(Point::new(70.0, 70.0));
        path.line_to(Point::new(70.0, 30.0));
        path.close_path();
        let subs = flatten_polygon(&path, 0.25);
        let idx = BoundaryIndex::new(&subs);
        for i in 0..40 {
            for j in 0..40 {
                let p = Point::new(i as f64 * 3.0 - 5.0, j as f64 * 3.0 - 5.0);
                assert_eq!(
                    polygon_contains_indexed(&subs, &idx, p),
                    polygon_contains(&subs, p),
                    "indexed/unindexed mismatch at {:?}",
                    p,
                );
            }
        }
    }

    #[test]
    fn rect_with_hole() {
        // Outer 0..100, inner hole 30..70: even-odd ray casting reports
        // inside the annulus and outside the hole.
        let mut path = BezPath::new();
        path.move_to(Point::new(0.0, 0.0));
        path.line_to(Point::new(100.0, 0.0));
        path.line_to(Point::new(100.0, 100.0));
        path.line_to(Point::new(0.0, 100.0));
        path.close_path();
        path.move_to(Point::new(30.0, 30.0));
        path.line_to(Point::new(30.0, 70.0));
        path.line_to(Point::new(70.0, 70.0));
        path.line_to(Point::new(70.0, 30.0));
        path.close_path();
        let subs = flatten_polygon(&path, 0.25);
        assert!(polygon_contains(&subs, Point::new(10.0, 50.0)));
        assert!(!polygon_contains(&subs, Point::new(50.0, 50.0)));
    }
}
