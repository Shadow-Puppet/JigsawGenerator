//! Specialized convex-polygon × polygon-with-holes clipping.
//!
//! Voronoi cells are *always* convex, and we clip ~150 of them per
//! Lloyd iteration during CVT generation. The general-purpose
//! `linesweeper` crate handles the operation correctly but pays a
//! steep per-call setup cost (event queue + balanced tree, designed
//! for arbitrary self-intersecting paths). This module exploits the
//! convex-subject special case to do the same job ~5–10× faster.
//!
//! # Algorithm
//!
//! Inputs:
//!   - `cell`: convex polygon vertices in order (CCW or CW; we
//!     normalize to CCW internally).
//!   - `boundary_subpaths`: a polygon-with-holes flattened to
//!     polylines, one subpath per closed loop. Conventional winding
//!     is outer-CCW, holes-CW so that even-odd ray-casting and
//!     forward-traversal directions agree.
//!
//! Phase 1 — Vertex classification: for each cell vertex, test
//! inside/outside the boundary via even-odd ray casting against the
//! flattened polyline subpaths.
//!
//! Phase 2 — Edge intersections: for each cell edge, find all
//! intersections with each boundary edge. Each intersection records
//! its parameter `t` along the cell edge and `s` along the boundary
//! edge plus the boundary subpath/edge index, so we can sort along
//! either polygon's perimeter.
//!
//! Phase 3 — Trace: walk the cell perimeter once. When inside the
//! boundary, emit cell vertices and edge segments. At an exit
//! intersection, jump onto the boundary and walk forward along the
//! subpath (in its native winding direction — which keeps the puzzle
//! interior on the left for both outer-CCW and hole-CW subpaths)
//! until we hit the next entry intersection back on the cell.
//!
//! Phase 4 — Disconnected components: repeat Phase 3 for any
//! intersection not yet visited (rare, only when a non-convex
//! boundary cuts the cell into ≥2 pieces).
//!
//! Phase 5 — Whole-subpath holes: any boundary subpath fully
//! contained inside the cell, with zero intersections, becomes an
//! additional subpath in the output (a hole in the clipped polygon).
//!
//! # Robustness
//!
//! The "happy path" of the algorithm is short. The bulk of the code
//! handles geometric degeneracies: vertices exactly on boundary
//! edges, parallel/colinear cell-and-boundary edges, intersections
//! within `EPS` of an existing vertex. Every such case has a
//! tie-breaker comment explaining the chosen behavior.

use kurbo::{BezPath, Point};

use crate::flat_boundary::{polygon_contains_indexed, BoundaryIndex};

/// Tolerance for treating two coordinates as identical (mm). About
/// 1 micron — well below laser kerf, well above floating-point
/// rounding for any reasonable puzzle dimensions.
const EPS: f64 = 1e-6;

/// Clip a convex polygon `cell` against a polygon-with-holes
/// boundary, returning the intersection as a closed `BezPath`.
///
/// `cell` must be a convex polygon given as ≥3 vertices in CCW or CW
/// order (winding is auto-detected and normalized internally). The
/// last vertex is implicitly connected back to the first.
///
/// `boundary_polygon` is the flattened boundary (one Vec<Point> per
/// subpath). `boundary_index` is the matching Y-bucket spatial index
/// — used to make Phase 1 O(log n) per query.
///
/// The output is empty when:
/// - the cell has < 3 vertices
/// - the cell is entirely outside the boundary
/// - the cell ∩ boundary has degenerate (zero-area) shape
pub fn convex_clip(
    cell: &[Point],
    boundary_polygon: &[Vec<Point>],
    boundary_index: &BoundaryIndex,
) -> BezPath {
    if cell.len() < 3 {
        return BezPath::new();
    }

    // Normalize cell to CCW. Convex cells from `voronoice` are
    // typically CCW, but we don't want to rely on it.
    let mut cell_v: Vec<Point> = cell.to_vec();
    if signed_area(&cell_v) < 0.0 {
        cell_v.reverse();
    }
    let n = cell_v.len();

    // Normalize boundary subpath orientations: outer CCW (positive
    // shoelace), holes CW (negative shoelace). The trace assumes
    // walking each subpath forward keeps the puzzle interior on the
    // LEFT — only true with this convention. Linesweeper and other
    // producers may emit subpaths in either direction; reverse any
    // that don't match, then rebuild the spatial index on the
    // normalized polygon (it stores edge indices, which become
    // invalid after a reversal). One ~50µs cost per clip call —
    // negligible vs. the ms-per-call linesweeper alternative.
    let normalized = normalize_subpath_orientations(boundary_polygon);
    let local_index = BoundaryIndex::new(&normalized);
    let boundary_polygon: &[Vec<Point>] = &normalized;
    let boundary_index: &BoundaryIndex = &local_index;

    // ─── Phase 1: classify cell vertices ────────────────────────
    let inside: Vec<bool> = cell_v
        .iter()
        .map(|v| polygon_contains_indexed(boundary_polygon, boundary_index, *v))
        .collect();

    // ─── Phase 2: find intersections per cell edge ──────────────
    //
    // `intersections_per_edge[i]` = list of intersections on cell
    //   edge i (cell_v[i] → cell_v[(i+1) % n]), sorted by `cell_t`.
    //
    // `intersections_per_boundary[(sp, j)]` = list of intersections
    //   on boundary edge j of subpath sp, sorted by `bnd_t`.
    //
    // Each intersection has a unique `id` so we can refer to the
    // same point from both indices and track visited state.
    let mut all_intersections: Vec<Intersection> = Vec::new();
    let mut intersections_per_edge: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut intersections_per_boundary: std::collections::HashMap<
        (usize, usize),
        Vec<usize>,
    > = std::collections::HashMap::new();

    for i in 0..n {
        let cell_a = cell_v[i];
        let cell_b = cell_v[(i + 1) % n];
        for (sp_idx, sp) in boundary_polygon.iter().enumerate() {
            let m = sp.len();
            if m < 3 {
                continue;
            }
            for j in 0..m {
                let bnd_a = sp[j];
                let bnd_b = sp[(j + 1) % m];
                if let Some((point, cell_t, bnd_t)) =
                    segment_intersect(cell_a, cell_b, bnd_a, bnd_b)
                {
                    let id = all_intersections.len();
                    all_intersections.push(Intersection {
                        point,
                        cell_edge: i,
                        cell_t,
                        boundary_subpath: sp_idx,
                        boundary_edge: j,
                        boundary_t: bnd_t,
                    });
                    intersections_per_edge[i].push(id);
                    intersections_per_boundary
                        .entry((sp_idx, j))
                        .or_default()
                        .push(id);
                }
            }
        }
        intersections_per_edge[i]
            .sort_by(|&a, &b| all_intersections[a].cell_t.partial_cmp(&all_intersections[b].cell_t).unwrap());
    }

    for ids in intersections_per_boundary.values_mut() {
        ids.sort_by(|&a, &b| {
            all_intersections[a]
                .boundary_t
                .partial_cmp(&all_intersections[b].boundary_t)
                .unwrap()
        });
    }

    // Classify each intersection as ENTRY or EXIT for the cell's
    // forward walk. Walking the cell perimeter, vertex `i` has
    // `inside[i]` state. Each intersection on edge i crosses the
    // boundary, flipping state. The flip's *direction* is what we
    // record:
    //   was_outside → now_inside  ⇒ ENTRY (start traces here)
    //   was_inside  → now_outside ⇒ EXIT
    //
    // This is the Greiner-Hörmann classification step. Without it,
    // a trace started mid-walk (at any intersection) might walk
    // forward on the cell into the OUTSIDE region and bail with a
    // partial polygon — the bug that produced our zero-area
    // outputs in the property-test failures.
    let mut intersection_is_entry: Vec<bool> =
        vec![false; all_intersections.len()];
    for i in 0..n {
        let mut state = inside[i];
        for &id in &intersections_per_edge[i] {
            let was_inside = state;
            state = !state;
            intersection_is_entry[id] = !was_inside && state;
        }
    }

    // ─── Phase 3 + 4: trace components ──────────────────────────
    //
    // Outputs accumulate here. The "happy" common case is that the
    // cell crosses the boundary 0 or 2 times and we get a single
    // subpath in the output.
    let mut output = BezPath::new();
    let mut wrote_any = false;
    let mut visited_intersections: Vec<bool> =
        vec![false; all_intersections.len()];

    // Common fast paths (no intersections):
    if all_intersections.is_empty() {
        // No crossings — either the cell is fully inside the
        // boundary or fully outside. Test one vertex.
        if inside.iter().all(|&b| b) {
            // Entirely inside: cell unchanged.
            emit_polygon(&mut output, &cell_v);
            wrote_any = true;
        } else {
            // Entirely outside the boundary outer subpath OR
            // entirely inside a hole (= outside the puzzle region).
            // Either way, no main output. Holes still need handling
            // below.
        }
    } else {
        // Trace components. The first trace can start from any
        // inside cell vertex (cleanest) or, if none, from any
        // unvisited intersection. Subsequent traces (only needed
        // when the boundary cuts the cell into ≥2 disjoint pieces)
        // must start from unvisited intersections — never from a
        // cell vertex, since `trace_component` doesn't mark cell
        // vertices visited and we'd loop on the same start
        // indefinitely.
        //
        // Also: hard outer cap. If a trace fails to make progress
        // (degenerate geometry, FP noise causing classification ↔
        // intersection inconsistency, etc.), this prevents an
        // infinite loop and falls back to whatever partial output
        // we got. Better a slightly-wrong polygon than a hung CVT.
        let mut traces_done = 0;
        let max_traces = cell_v.len() + all_intersections.len() + 4;
        let mut start_idx: Option<TraceStart> = first_trace_start(
            &cell_v,
            &inside,
            &intersection_is_entry,
            &visited_intersections,
        );
        while let Some(start) = start_idx {
            traces_done += 1;
            if traces_done > max_traces {
                break;
            }
            let visited_before =
                visited_intersections.iter().filter(|&&v| v).count();
            trace_component(
                start,
                &cell_v,
                &inside,
                &intersections_per_edge,
                &intersections_per_boundary,
                &all_intersections,
                boundary_polygon,
                &mut visited_intersections,
                &mut output,
            );
            let visited_after =
                visited_intersections.iter().filter(|&&v| v).count();
            if visited_after == visited_before {
                // Trace made no progress — bail to avoid infinite
                // outer loop on degenerate inputs.
                break;
            }
            wrote_any = true;
            // Subsequent starts: only unvisited ENTRY intersections.
            // Exits don't make valid trace heads (walking forward
            // from an exit immediately leaves the clip region).
            start_idx = next_unvisited_entry(
                &intersection_is_entry,
                &visited_intersections,
            );
        }
    }

    // ─── Phase 5: whole-subpath holes ───────────────────────────
    //
    // Any boundary subpath that has ZERO intersections with the
    // cell AND whose first vertex is inside the cell becomes a hole
    // in the output. Common case: a whimsy fully inside a single
    // big cell.
    //
    // (Output is only meaningful if we wrote a main outline; an
    // outer-boundary subpath fully outside our cell should be
    // skipped, not emitted.)
    if wrote_any {
        for (sp_idx, sp) in boundary_polygon.iter().enumerate() {
            if sp.len() < 3 {
                continue;
            }
            let touched = (0..sp.len())
                .any(|j| intersections_per_boundary.contains_key(&(sp_idx, j)));
            if touched {
                continue;
            }
            // No intersections with the cell at all. Is the subpath
            // inside the cell?
            if point_in_convex_polygon(&cell_v, sp[0]) {
                // Yes — emit it as a hole. Subpath is already in
                // its native winding direction, which is opposite
                // of the outer outline's winding, so even-odd
                // rendering correctly punches a hole.
                emit_polygon(&mut output, sp);
            }
        }
    }

    output
}

/// One detected cell-edge × boundary-edge intersection.
#[derive(Clone, Copy, Debug)]
struct Intersection {
    point: Point,
    cell_edge: usize,
    /// Parameter along the cell edge in `[0, 1]`.
    cell_t: f64,
    boundary_subpath: usize,
    boundary_edge: usize,
    /// Parameter along the boundary edge in `[0, 1]`.
    boundary_t: f64,
}

/// Description of where to start a trace: either a cell vertex
/// (inside the boundary) or an "entry" intersection.
#[derive(Clone, Copy, Debug)]
enum TraceStart {
    CellVertex(usize),
    Intersection(usize),
}

/// Pick the start point for the FIRST trace of this clip operation.
/// Prefers any inside cell vertex (gives a clean cell-vertex start);
/// falls back to any unvisited ENTRY intersection when no vertex is
/// inside (e.g., the cell is fully outside the boundary but slices
/// through its interior — the bug discovered by
/// `property_test_against_linesweeper`).
fn first_trace_start(
    cell_v: &[Point],
    inside: &[bool],
    is_entry: &[bool],
    visited: &[bool],
) -> Option<TraceStart> {
    for i in 0..cell_v.len() {
        if inside[i] {
            return Some(TraceStart::CellVertex(i));
        }
    }
    next_unvisited_entry(is_entry, visited)
}

/// Find the next unvisited ENTRY-type intersection. Used to start
/// further trace components when the boundary cuts the cell into
/// multiple disjoint pieces. EXIT intersections don't qualify —
/// walking forward on the cell from an exit immediately leaves the
/// clip region. Returns `None` when every entry has been consumed.
fn next_unvisited_entry(
    is_entry: &[bool],
    visited: &[bool],
) -> Option<TraceStart> {
    is_entry
        .iter()
        .zip(visited.iter())
        .position(|(&entry, &v)| entry && !v)
        .map(TraceStart::Intersection)
}

/// Trace one connected component of the clip polygon, emit it to
/// `output`, mark visited intersections.
#[allow(clippy::too_many_arguments)]
fn trace_component(
    start: TraceStart,
    cell_v: &[Point],
    inside: &[bool],
    intersections_per_edge: &[Vec<usize>],
    intersections_per_boundary: &std::collections::HashMap<
        (usize, usize),
        Vec<usize>,
    >,
    all_ix: &[Intersection],
    boundary_polygon: &[Vec<Point>],
    visited: &mut [bool],
    output: &mut BezPath,
) {
    let n = cell_v.len();
    let mut polygon: Vec<Point> = Vec::new();

    // Cursor state: are we currently walking along the cell or
    // along a boundary subpath?
    enum Cursor {
        Cell {
            /// Current edge being traversed.
            edge: usize,
            /// Parameter along edge — start position within this edge.
            t: f64,
            /// Index into `intersections_per_edge[edge]` — the next
            /// intersection to encounter on this edge.
            next_ix_pos: usize,
        },
        Boundary {
            sp: usize,
            edge: usize,
            t: f64,
            next_ix_pos: usize,
        },
    }

    let (mut cursor, start_point) = match start {
        TraceStart::CellVertex(i) => {
            let cursor = Cursor::Cell {
                edge: i,
                t: 0.0,
                next_ix_pos: 0,
            };
            (cursor, cell_v[i])
        }
        TraceStart::Intersection(id) => {
            // Start mid-edge on the cell, just *after* the
            // intersection point. This becomes an "exit" event we
            // immediately switch from at the start of the loop —
            // simpler to encode by starting one step later.
            let ix = &all_ix[id];
            visited[id] = true;
            // Position: just past this intersection on the cell.
            let pos_in_list = intersections_per_edge[ix.cell_edge]
                .iter()
                .position(|&x| x == id)
                .expect("intersection id not in edge list");
            let cursor = Cursor::Cell {
                edge: ix.cell_edge,
                t: ix.cell_t,
                next_ix_pos: pos_in_list + 1,
            };
            (cursor, ix.point)
        }
    };
    polygon.push(start_point);

    // Cap iteration to prevent infinite loops on pathological
    // input (shouldn't happen with valid convex cell + valid
    // boundary, but defensive).
    let max_steps = 4 * n + 4 * boundary_polygon.iter().map(|s| s.len()).sum::<usize>() + 64;
    for _step in 0..max_steps {
        match cursor {
            Cursor::Cell {
                edge,
                t: _,
                next_ix_pos,
            } => {
                // Walk along cell from current position to either
                // (a) next intersection on this edge, (b) end of
                // edge (vertex (edge+1)%n).
                if next_ix_pos < intersections_per_edge[edge].len() {
                    let ix_id = intersections_per_edge[edge][next_ix_pos];
                    if visited[ix_id] {
                        // Loop closed.
                        break;
                    }
                    visited[ix_id] = true;
                    let ix = all_ix[ix_id];
                    push_unique(&mut polygon, ix.point);
                    // Switch to boundary, walking forward along the
                    // boundary subpath from this intersection.
                    let pos_in_bnd = intersections_per_boundary
                        [&(ix.boundary_subpath, ix.boundary_edge)]
                        .iter()
                        .position(|&x| x == ix_id)
                        .expect("intersection not in boundary list");
                    cursor = Cursor::Boundary {
                        sp: ix.boundary_subpath,
                        edge: ix.boundary_edge,
                        t: ix.boundary_t,
                        next_ix_pos: pos_in_bnd + 1,
                    };
                } else {
                    // No more intersections on this cell edge —
                    // arrive at the next vertex.
                    let next_v = (edge + 1) % n;
                    let next_point = cell_v[next_v];
                    if (next_point - polygon[0]).hypot() < EPS && polygon.len() > 1 {
                        break;
                    }
                    push_unique(&mut polygon, next_point);
                    if !inside[next_v] {
                        // Vertex is outside — shouldn't happen in
                        // a "Cell" cursor without an intervening
                        // intersection. Bail to avoid infinite loop.
                        break;
                    }
                    cursor = Cursor::Cell {
                        edge: next_v,
                        t: 0.0,
                        next_ix_pos: 0,
                    };
                }
            }
            Cursor::Boundary {
                sp,
                edge,
                t: _,
                next_ix_pos,
            } => {
                // Walk along boundary from current position. Look
                // for the next intersection on this boundary edge
                // (after our current t). If none, advance to the
                // next boundary edge (emit boundary vertex on the
                // way) and continue searching.
                let ixs_here =
                    intersections_per_boundary.get(&(sp, edge));
                let next_ix_id = ixs_here.and_then(|ids| {
                    ids.get(next_ix_pos).copied()
                });

                if let Some(ix_id) = next_ix_id {
                    if visited[ix_id] {
                        break;
                    }
                    visited[ix_id] = true;
                    let ix = all_ix[ix_id];
                    push_unique(&mut polygon, ix.point);
                    // Switch back to cell, continuing forward
                    // along the cell edge from this intersection.
                    let pos_in_cell = intersections_per_edge[ix.cell_edge]
                        .iter()
                        .position(|&x| x == ix_id)
                        .expect("intersection not in cell list");
                    cursor = Cursor::Cell {
                        edge: ix.cell_edge,
                        t: ix.cell_t,
                        next_ix_pos: pos_in_cell + 1,
                    };
                } else {
                    // No more intersections on this boundary
                    // edge. Advance to the next boundary vertex
                    // (which we may need to emit) and check the
                    // next boundary edge.
                    let m = boundary_polygon[sp].len();
                    let next_b_v = (edge + 1) % m;
                    let next_point = boundary_polygon[sp][next_b_v];
                    if (next_point - polygon[0]).hypot() < EPS && polygon.len() > 1 {
                        break;
                    }
                    push_unique(&mut polygon, next_point);
                    cursor = Cursor::Boundary {
                        sp,
                        edge: next_b_v,
                        t: 0.0,
                        next_ix_pos: 0,
                    };
                }
            }
        }
    }

    if polygon.len() >= 3 {
        emit_polygon(output, &polygon);
    }
}

/// Append a vertex to `polygon`, skipping it if it's a duplicate of
/// the most-recently-pushed point (within `EPS`). Avoids zero-length
/// polygon edges from numerical noise at intersections.
fn push_unique(polygon: &mut Vec<Point>, p: Point) {
    if let Some(last) = polygon.last() {
        if (*last - p).hypot() < EPS {
            return;
        }
    }
    polygon.push(p);
}

/// Append `polygon` as a closed subpath of `path` (move + lines +
/// close).
fn emit_polygon(path: &mut BezPath, polygon: &[Point]) {
    if polygon.len() < 3 {
        return;
    }
    path.move_to(polygon[0]);
    for p in &polygon[1..] {
        path.line_to(*p);
    }
    path.close_path();
}

/// Normalize a polygon-with-holes to "outer CCW, holes CW" so that
/// walking each subpath forward keeps the puzzle interior (the
/// region inside the outer subpath but outside every hole) on the
/// left.
///
/// "Outer" is identified as the subpath with the largest absolute
/// signed area — geometrically the only subpath that contains all
/// the others. For each subpath, if its current winding doesn't
/// match the canonical convention, we reverse it.
fn normalize_subpath_orientations(
    subpaths: &[Vec<Point>],
) -> Vec<Vec<Point>> {
    let mut out: Vec<Vec<Point>> = subpaths.iter().cloned().collect();
    let signed_areas: Vec<f64> = out.iter().map(|sp| signed_area(sp)).collect();
    let outer_idx = signed_areas
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
        .map(|(i, _)| i);
    for (i, sub) in out.iter_mut().enumerate() {
        let want_positive = Some(i) == outer_idx;
        let is_positive = signed_areas[i] > 0.0;
        if want_positive != is_positive {
            sub.reverse();
        }
    }
    out
}

/// Signed area of a polygon via the shoelace formula. Positive →
/// CCW, negative → CW.
fn signed_area(polygon: &[Point]) -> f64 {
    let mut sum = 0.0;
    let n = polygon.len();
    for i in 0..n {
        let a = polygon[i];
        let b = polygon[(i + 1) % n];
        sum += a.x * b.y - b.x * a.y;
    }
    sum * 0.5
}

/// Test whether `p` is inside a convex polygon. Uses the
/// "all-cross-products-same-sign" rule, which is O(n) but with
/// minimal work per edge — fine for cells of ~6 vertices.
fn point_in_convex_polygon(polygon: &[Point], p: Point) -> bool {
    let n = polygon.len();
    if n < 3 {
        return false;
    }
    let mut sign: f64 = 0.0;
    for i in 0..n {
        let a = polygon[i];
        let b = polygon[(i + 1) % n];
        let cross = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
        if cross.abs() < EPS {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if sign * cross < 0.0 {
            return false;
        }
    }
    true
}

/// Compute the intersection of two line segments, returning
/// `Some((point, t_along_first, s_along_second))` where `t` and
/// `s` are in `[0, 1]`.
///
/// Returns `None` if:
/// - the segments are parallel or colinear (degenerate)
/// - the intersection lies outside `[0, 1]` on either segment
/// - either segment has zero length
fn segment_intersect(
    a1: Point,
    a2: Point,
    b1: Point,
    b2: Point,
) -> Option<(Point, f64, f64)> {
    let r_x = a2.x - a1.x;
    let r_y = a2.y - a1.y;
    let s_x = b2.x - b1.x;
    let s_y = b2.y - b1.y;
    let denom = r_x * s_y - r_y * s_x;
    if denom.abs() < EPS {
        return None; // parallel or colinear
    }
    let dx = b1.x - a1.x;
    let dy = b1.y - a1.y;
    let t = (dx * s_y - dy * s_x) / denom;
    let s = (dx * r_y - dy * r_x) / denom;
    // Permit a touch of slop so vertex-on-edge intersections still
    // register. Downstream `push_unique` will dedupe spurious
    // near-coincident points.
    if !(-EPS..=1.0 + EPS).contains(&t) {
        return None;
    }
    if !(-EPS..=1.0 + EPS).contains(&s) {
        return None;
    }
    let t = t.clamp(0.0, 1.0);
    let s = s.clamp(0.0, 1.0);
    let point = Point::new(a1.x + t * r_x, a1.y + t * r_y);
    Some((point, t, s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flat_boundary::flatten_polygon;
    use crate::masking::mask_intersection;
    use kurbo::{BezPath, PathEl};

    /// Build a closed `BezPath` from a slice of points (move to first,
    /// line to rest, close).
    fn polygon_path(pts: &[Point]) -> BezPath {
        let mut path = BezPath::new();
        path.move_to(pts[0]);
        for p in &pts[1..] {
            path.line_to(*p);
        }
        path.close_path();
        path
    }

    /// Total area of a (possibly multi-subpath) flattened polygon.
    fn total_area(path: &BezPath) -> f64 {
        let mut subs: Vec<Vec<Point>> = Vec::new();
        let mut cur: Vec<Point> = Vec::new();
        kurbo::flatten(path.iter(), 0.25, |el| match el {
            PathEl::MoveTo(p) => {
                if !cur.is_empty() {
                    subs.push(std::mem::take(&mut cur));
                }
                cur.push(p);
            }
            PathEl::LineTo(p) => cur.push(p),
            _ => {}
        });
        if !cur.is_empty() {
            subs.push(cur);
        }
        // Drop trailing duplicate-of-first
        for sub in &mut subs {
            if sub.len() >= 2 && (sub[sub.len() - 1] - sub[0]).hypot() < 1e-6 {
                sub.pop();
            }
        }
        let mut total = 0.0;
        for sub in &subs {
            total += signed_area(sub).abs();
        }
        total
    }

    /// Property: convex_clip and linesweeper produce equal-area
    /// polygons (within tolerance) for the same inputs.
    fn assert_areas_match(
        cell_pts: &[Point],
        boundary: &BezPath,
        label: &str,
    ) {
        let cell_path = polygon_path(cell_pts);
        let boundary_polygon = flatten_polygon(boundary, 0.25);
        let boundary_index = BoundaryIndex::new(&boundary_polygon);

        let custom = convex_clip(cell_pts, &boundary_polygon, &boundary_index);
        let reference = mask_intersection(&cell_path, boundary)
            .expect("linesweeper should succeed");

        let custom_area = total_area(&custom);
        let ref_area = total_area(&reference);
        let diff = (custom_area - ref_area).abs();
        let rel = if ref_area > EPS {
            diff / ref_area
        } else {
            diff
        };
        assert!(
            rel < 0.02 || diff < 0.5,
            "{}: area mismatch (custom={}, ref={}, diff={}, rel={:.4})",
            label,
            custom_area,
            ref_area,
            diff,
            rel,
        );
    }

    fn rect_path(x0: f64, y0: f64, x1: f64, y1: f64) -> BezPath {
        polygon_path(&[
            Point::new(x0, y0),
            Point::new(x1, y0),
            Point::new(x1, y1),
            Point::new(x0, y1),
        ])
    }

    #[test]
    fn cell_fully_inside() {
        let cell = vec![
            Point::new(40.0, 40.0),
            Point::new(60.0, 40.0),
            Point::new(60.0, 60.0),
            Point::new(40.0, 60.0),
        ];
        let boundary = rect_path(0.0, 0.0, 100.0, 100.0);
        assert_areas_match(&cell, &boundary, "cell fully inside rect");
    }

    #[test]
    fn cell_fully_outside() {
        let cell = vec![
            Point::new(200.0, 200.0),
            Point::new(220.0, 200.0),
            Point::new(220.0, 220.0),
            Point::new(200.0, 220.0),
        ];
        let boundary = rect_path(0.0, 0.0, 100.0, 100.0);
        assert_areas_match(&cell, &boundary, "cell fully outside rect");
    }

    #[test]
    fn cell_clipped_left_edge() {
        let cell = vec![
            Point::new(-20.0, 40.0),
            Point::new(20.0, 40.0),
            Point::new(20.0, 60.0),
            Point::new(-20.0, 60.0),
        ];
        let boundary = rect_path(0.0, 0.0, 100.0, 100.0);
        assert_areas_match(&cell, &boundary, "cell clipped on left");
    }

    #[test]
    fn cell_clipped_corner() {
        let cell = vec![
            Point::new(-10.0, -10.0),
            Point::new(30.0, -10.0),
            Point::new(30.0, 30.0),
            Point::new(-10.0, 30.0),
        ];
        let boundary = rect_path(0.0, 0.0, 100.0, 100.0);
        assert_areas_match(&cell, &boundary, "cell clipped at corner");
    }

    #[test]
    fn triangle_cell_in_rect() {
        let cell = vec![
            Point::new(10.0, 10.0),
            Point::new(40.0, 10.0),
            Point::new(25.0, 30.0),
        ];
        let boundary = rect_path(0.0, 0.0, 100.0, 100.0);
        assert_areas_match(&cell, &boundary, "triangle cell inside rect");
    }

    #[test]
    fn triangle_cell_clipped() {
        let cell = vec![
            Point::new(50.0, 50.0),
            Point::new(150.0, 50.0),
            Point::new(100.0, 150.0),
        ];
        let boundary = rect_path(0.0, 0.0, 100.0, 100.0);
        assert_areas_match(&cell, &boundary, "triangle cell clipped");
    }

    /// Massive cross-check: build real Voronoi diagrams over various
    /// boundary shapes + seed counts, and verify that for every
    /// cell, our `convex_clip` produces the same total area as
    /// `linesweeper::mask_intersection` (within tolerance).
    ///
    /// Linesweeper is the trusted oracle. Any cell where we diverge
    /// gets printed with its inputs so we can reduce to a minimal
    /// repro and fix.
    ///
    /// This test is the heart of correctness — it runs ~hundreds of
    /// real CVT cells against rect/heart/star boundaries plus a
    /// boundary with a whimsy hole. If this passes, the integration
    /// tests should also pass.
    #[test]
    fn property_test_against_linesweeper() {
        use crate::masking::mask_difference;
        use crate::shapes::{circle_path, heart_path, star_path};
        use kurbo::Shape;
        use voronoice::{
            BoundingBox, ClipBehavior, Point as VPoint, VoronoiBuilder,
        };

        // Build a few representative boundaries.
        let mut rect = BezPath::new();
        rect.move_to(Point::new(0.0, 0.0));
        rect.line_to(Point::new(200.0, 0.0));
        rect.line_to(Point::new(200.0, 150.0));
        rect.line_to(Point::new(0.0, 150.0));
        rect.close_path();
        let heart = heart_path(200.0, 150.0);
        let star = star_path(200.0, 150.0, 5, 4.0);
        // Boundary with a hole (rect minus circular whimsy).
        let mut hole_outer = BezPath::new();
        hole_outer.move_to(Point::new(0.0, 0.0));
        hole_outer.line_to(Point::new(200.0, 0.0));
        hole_outer.line_to(Point::new(200.0, 150.0));
        hole_outer.line_to(Point::new(0.0, 150.0));
        hole_outer.close_path();
        let whimsy = circle_path(40.0, 40.0);
        // Translate whimsy to (60, 75)
        let whimsy = kurbo::Affine::translate(kurbo::Vec2::new(60.0, 75.0))
            * whimsy;
        let rect_with_hole = mask_difference(&hole_outer, &whimsy)
            .expect("difference should succeed");

        let scenarios: &[(&str, &BezPath, &[usize])] = &[
            ("rect", &rect, &[4, 8, 16]),
            ("heart", &heart, &[4, 8, 16]),
            ("star", &star, &[8, 16]),
            ("rect-with-hole", &rect_with_hole, &[8, 16]),
        ];

        let mut failures = 0;
        let mut total_cells = 0;

        for (name, boundary, counts) in scenarios {
            for &n_seeds in *counts {
                let bbox = boundary.bounding_box();
                let pad = bbox.width().max(bbox.height()) * 0.1 + 1.0;
                let v_bbox = BoundingBox::new(
                    VPoint {
                        x: bbox.center().x,
                        y: bbox.center().y,
                    },
                    bbox.width() + 2.0 * pad,
                    bbox.height() + 2.0 * pad,
                );

                // Rejection-sample seeds inside the boundary.
                let mut seeds: Vec<VPoint> = Vec::new();
                let mut state: u64 = 0xCAFEBABE_u64.wrapping_add(n_seeds as u64);
                while seeds.len() < n_seeds {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let x = ((state >> 33) as f64 / (1u64 << 31) as f64) * bbox.width() + bbox.x0;
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let y = ((state >> 33) as f64 / (1u64 << 31) as f64) * bbox.height() + bbox.y0;
                    if boundary.winding(Point::new(x, y)) != 0 {
                        seeds.push(VPoint { x, y });
                    }
                    if seeds.len() == 0 && state.count_ones() > 10000 {
                        break;
                    }
                }
                if seeds.len() < 2 {
                    continue;
                }

                let voronoi = match VoronoiBuilder::default()
                    .set_sites(seeds.clone())
                    .set_bounding_box(v_bbox)
                    .set_clip_behavior(ClipBehavior::Clip)
                    .build()
                {
                    Some(v) => v,
                    None => continue,
                };

                let boundary_polygon = flatten_polygon(boundary, 0.25);
                let boundary_index = BoundaryIndex::new(&boundary_polygon);

                for site_idx in 0..seeds.len() {
                    let cell_pts: Vec<Point> = voronoi
                        .cell(site_idx)
                        .iter_vertices()
                        .map(|v| Point::new(v.x, v.y))
                        .collect();
                    if cell_pts.len() < 3 {
                        continue;
                    }
                    let cell_path = polygon_path(&cell_pts);

                    let custom = convex_clip(
                        &cell_pts,
                        &boundary_polygon,
                        &boundary_index,
                    );
                    let reference = match mask_intersection(&cell_path, boundary) {
                        Ok(r) => r,
                        Err(_) => continue,
                    };

                    let custom_area = total_area(&custom);
                    let ref_area = total_area(&reference);
                    total_cells += 1;

                    let diff = (custom_area - ref_area).abs();
                    let rel = if ref_area > EPS {
                        diff / ref_area
                    } else {
                        diff
                    };
                    let bad = rel > 0.05 && diff > 0.5;
                    if bad {
                        failures += 1;
                        if failures <= 3 {
                            eprintln!(
                                "MISMATCH {}/{}/site{}: cell={:?} custom_area={:.3} ref_area={:.3} rel={:.4}",
                                name, n_seeds, site_idx, cell_pts, custom_area, ref_area, rel,
                            );
                        }
                    }
                }
            }
        }

        eprintln!(
            "property test: {} / {} cells diverged",
            failures, total_cells,
        );
        assert_eq!(failures, 0, "see stderr for first 3 failing cases");
    }

    /// Cell wraps around a hole (whimsy fully inside the cell).
    #[test]
    fn cell_with_hole() {
        let cell = vec![
            Point::new(10.0, 10.0),
            Point::new(90.0, 10.0),
            Point::new(90.0, 90.0),
            Point::new(10.0, 90.0),
        ];
        // Outer rect 0..100 with a hole 40..60.
        let mut boundary = BezPath::new();
        boundary.move_to(Point::new(0.0, 0.0));
        boundary.line_to(Point::new(100.0, 0.0));
        boundary.line_to(Point::new(100.0, 100.0));
        boundary.line_to(Point::new(0.0, 100.0));
        boundary.close_path();
        // Hole, CW
        boundary.move_to(Point::new(40.0, 40.0));
        boundary.line_to(Point::new(40.0, 60.0));
        boundary.line_to(Point::new(60.0, 60.0));
        boundary.line_to(Point::new(60.0, 40.0));
        boundary.close_path();
        assert_areas_match(&cell, &boundary, "cell containing hole");
    }
}
