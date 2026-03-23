//! Boundary-aware puzzle grid engine.
//!
//! Wraps a rectangular [`PuzzleGrid`] and classifies cells as inside or outside
//! a closed boundary shape. Edges between two inside cells are kept (with their
//! connectors); all other edges are excluded. The boundary shape contour
//! replaces the rectangular border.
//!
//! The rectangular grid is generated first for RNG determinism — boundary
//! filtering is a pure post-processing step.

use kurbo::{BezPath, PathEl, Point, Shape};

use crate::binary_export::{CMD_CLOSE, CMD_CURVE_TO, CMD_LINE_TO, CMD_MOVE_TO, EDGE_STRIDE};
use crate::grid::PuzzleGrid;
use crate::svg_export::{build_svg_document, edge_transform};

/// A puzzle grid clipped to a non-rectangular boundary shape.
///
/// The inner `PuzzleGrid` is generated rectangularly (preserving the RNG
/// sequence for determinism). Cell inclusion is computed by testing each
/// cell's center against the boundary shape using the winding number rule.
pub struct BoundaryPuzzle {
    /// The underlying full rectangular grid.
    pub grid: PuzzleGrid,
    /// The boundary shape (closed BezPath).
    pub boundary: BezPath,
    /// Cell inclusion matrix: `cell_inside[row][col]` is true if the cell
    /// center falls inside the boundary shape.
    pub cell_inside: Vec<Vec<bool>>,
}

impl BoundaryPuzzle {
    /// Create a boundary puzzle by classifying cells against a boundary shape.
    ///
    /// A cell at `(row, col)` is included if its center point has a nonzero
    /// winding number with respect to the boundary path.
    ///
    /// Cell center for `(row, col)` is:
    /// ```text
    /// x = (col + 0.5) * cell_w
    /// y = (row + 0.5) * cell_h
    /// ```
    pub fn new(grid: PuzzleGrid, boundary: BezPath) -> Self {
        let rows = grid.config.rows as usize;
        let cols = grid.config.cols as usize;
        let cell_w = grid.config.width / cols as f64;
        let cell_h = grid.config.height / rows as f64;

        let cell_inside = (0..rows)
            .map(|row| {
                (0..cols)
                    .map(|col| {
                        let center = Point::new(
                            (col as f64 + 0.5) * cell_w,
                            (row as f64 + 0.5) * cell_h,
                        );
                        boundary.winding(center) != 0
                    })
                    .collect()
            })
            .collect();

        Self {
            grid,
            boundary,
            cell_inside,
        }
    }

    /// Create a boundary puzzle with a hole (whimsy difference mode, R004).
    ///
    /// A cell is included if it is inside the boundary AND outside the hole.
    /// This supports cutting whimsy shapes out of the puzzle interior.
    pub fn new_with_hole(grid: PuzzleGrid, boundary: BezPath, hole: BezPath) -> Self {
        let rows = grid.config.rows as usize;
        let cols = grid.config.cols as usize;
        let cell_w = grid.config.width / cols as f64;
        let cell_h = grid.config.height / rows as f64;

        let cell_inside = (0..rows)
            .map(|row| {
                (0..cols)
                    .map(|col| {
                        let center = Point::new(
                            (col as f64 + 0.5) * cell_w,
                            (row as f64 + 0.5) * cell_h,
                        );
                        let inside_boundary = boundary.winding(center) != 0;
                        let inside_hole = hole.winding(center) != 0;
                        inside_boundary && !inside_hole
                    })
                    .collect()
            })
            .collect();

        Self {
            grid,
            boundary,
            cell_inside,
        }
    }

    /// Return the `(row, col)` pairs of all cells that are inside the boundary.
    pub fn included_cells(&self) -> Vec<(usize, usize)> {
        let rows = self.grid.config.rows as usize;
        let cols = self.grid.config.cols as usize;
        let mut cells = Vec::new();
        for row in 0..rows {
            for col in 0..cols {
                if self.cell_inside[row][col] {
                    cells.push((row, col));
                }
            }
        }
        cells
    }

    /// Return indices into `grid.h_edges` for horizontal edges between two
    /// included cells.
    ///
    /// A horizontal edge at grid position `(row, col)` separates cell
    /// `(row-1, col)` (above) and cell `(row, col)` (below). It is included
    /// only if both cells exist and are inside the boundary.
    ///
    /// Border h-edges (row == 0 or row == rows) are always excluded — the
    /// shape contour replaces the rectangular border.
    pub fn included_h_edges(&self) -> Vec<usize> {
        let rows = self.grid.config.rows as usize;
        let cols = self.grid.config.cols as usize;
        let mut indices = Vec::new();

        // Internal h-edges: rows 1..rows (exclusive of border rows 0 and rows)
        for row in 1..rows {
            for col in 0..cols {
                let above = self.cell_inside[row - 1][col];
                let below = self.cell_inside[row][col];
                if above && below {
                    let idx = row * cols + col;
                    indices.push(idx);
                }
            }
        }

        indices
    }

    /// Return indices into `grid.v_edges` for vertical edges between two
    /// included cells.
    ///
    /// A vertical edge at grid position `(row, col)` separates cell
    /// `(row, col-1)` (left) and cell `(row, col)` (right). It is included
    /// only if both cells exist and are inside the boundary.
    ///
    /// Border v-edges (col == 0 or col == cols) are always excluded — the
    /// shape contour replaces the rectangular border.
    pub fn included_v_edges(&self) -> Vec<usize> {
        let rows = self.grid.config.rows as usize;
        let cols = self.grid.config.cols as usize;
        let mut indices = Vec::new();

        // Internal v-edges: cols 1..cols (exclusive of border cols 0 and cols)
        for row in 0..rows {
            for col in 1..cols {
                let left = self.cell_inside[row][col - 1];
                let right = self.cell_inside[row][col];
                if left && right {
                    let idx = row * (cols + 1) + col;
                    indices.push(idx);
                }
            }
        }

        indices
    }

    /// Total number of included internal edges (horizontal + vertical).
    pub fn included_edge_count(&self) -> usize {
        self.included_h_edges().len() + self.included_v_edges().len()
    }

    // ─── Export Methods ──────────────────────────────────────────

    /// Generate a complete SVG document using the boundary shape as the border.
    ///
    /// The SVG contains:
    /// 1. The boundary BezPath as the border subpath (may include cubic curves)
    /// 2. Open subpaths for each included internal edge's connector curves
    ///
    /// The boundary shape's PathEl elements are iterated directly, supporting
    /// multi-subpath BezPaths (though typically there's one closed subpath).
    pub fn generate_boundary_svg(&self) -> String {
        let mut combined = BezPath::new();

        // 1. Boundary shape as border
        for el in self.boundary.iter() {
            match el {
                PathEl::MoveTo(p) => combined.move_to(p),
                PathEl::LineTo(p) => combined.line_to(p),
                PathEl::CurveTo(p1, p2, p3) => combined.curve_to(p1, p2, p3),
                PathEl::ClosePath => combined.close_path(),
                _ => {}
            }
        }

        // 2. Included internal edge connectors
        self.append_included_edge_paths(&mut combined);

        let path_data = combined.to_svg();
        build_svg_document(&path_data, self.grid.config.width, self.grid.config.height)
    }

    /// Serialize only included internal edge connectors as a flat f64 array.
    ///
    /// Format is identical to `edges_to_binary()` — each edge is encoded as
    /// `EDGE_STRIDE` (36) f64 values with header, moveTo, and 5 curve segments.
    /// Only edges between two included cells are serialized.
    pub fn boundary_edges_to_binary(&self) -> Vec<f64> {
        let h_indices = self.included_h_edges();
        let v_indices = self.included_v_edges();

        let mut data = Vec::with_capacity((h_indices.len() + v_indices.len()) * EDGE_STRIDE);

        // Included horizontal edges
        for &idx in &h_indices {
            let edge = &self.grid.h_edges[idx];
            if let Some(ref curves) = edge.connector {
                let transform = edge_transform(edge.start, edge.end);

                // Header: start/end points for AABB culling
                data.push(edge.start.x);
                data.push(edge.start.y);
                data.push(edge.end.x);
                data.push(edge.end.y);

                // MoveTo: first curve's p0 transformed
                let p0 = transform * curves[0].p0;
                data.push(p0.x);
                data.push(p0.y);

                // 5 curves × 6 floats (p1, p2, p3)
                for curve in curves {
                    let p1 = transform * curve.p1;
                    let p2 = transform * curve.p2;
                    let p3 = transform * curve.p3;
                    data.push(p1.x);
                    data.push(p1.y);
                    data.push(p2.x);
                    data.push(p2.y);
                    data.push(p3.x);
                    data.push(p3.y);
                }
            }
        }

        // Included vertical edges
        for &idx in &v_indices {
            let edge = &self.grid.v_edges[idx];
            if let Some(ref curves) = edge.connector {
                let transform = edge_transform(edge.start, edge.end);

                data.push(edge.start.x);
                data.push(edge.start.y);
                data.push(edge.end.x);
                data.push(edge.end.y);

                let p0 = transform * curves[0].p0;
                data.push(p0.x);
                data.push(p0.y);

                for curve in curves {
                    let p1 = transform * curve.p1;
                    let p2 = transform * curve.p2;
                    let p3 = transform * curve.p3;
                    data.push(p1.x);
                    data.push(p1.y);
                    data.push(p2.x);
                    data.push(p2.y);
                    data.push(p3.x);
                    data.push(p3.y);
                }
            }
        }

        data
    }

    /// Serialize the boundary shape path as a command-prefixed f64 array.
    ///
    /// Commands match `border_to_binary()`:
    /// - `0.0` (moveTo) + x, y (3 floats)
    /// - `1.0` (lineTo) + x, y (3 floats)
    /// - `2.0` (curveTo) + p1.x, p1.y, p2.x, p2.y, p3.x, p3.y (7 floats)
    /// - `3.0` (closePath) (1 float)
    ///
    /// Supports multi-subpath BezPaths (iterates all PathEl variants).
    pub fn boundary_border_to_binary(&self) -> Vec<f64> {
        let mut data = Vec::with_capacity(128);

        for el in self.boundary.iter() {
            match el {
                PathEl::MoveTo(p) => {
                    data.push(CMD_MOVE_TO);
                    data.push(p.x);
                    data.push(p.y);
                }
                PathEl::LineTo(p) => {
                    data.push(CMD_LINE_TO);
                    data.push(p.x);
                    data.push(p.y);
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    data.push(CMD_CURVE_TO);
                    data.push(p1.x);
                    data.push(p1.y);
                    data.push(p2.x);
                    data.push(p2.y);
                    data.push(p3.x);
                    data.push(p3.y);
                }
                PathEl::ClosePath => {
                    data.push(CMD_CLOSE);
                }
                _ => {}
            }
        }

        data
    }

    // ─── Private Helpers ─────────────────────────────────────────

    /// Append included internal edge connector curves to an existing BezPath.
    fn append_included_edge_paths(&self, path: &mut BezPath) {
        let h_indices = self.included_h_edges();
        let v_indices = self.included_v_edges();

        for &idx in &h_indices {
            let edge = &self.grid.h_edges[idx];
            if let Some(ref curves) = edge.connector {
                let transform = edge_transform(edge.start, edge.end);
                let first_p0 = transform * curves[0].p0;
                path.move_to(first_p0);
                for curve in curves {
                    let p1 = transform * curve.p1;
                    let p2 = transform * curve.p2;
                    let p3 = transform * curve.p3;
                    path.curve_to(p1, p2, p3);
                }
            }
        }

        for &idx in &v_indices {
            let edge = &self.grid.v_edges[idx];
            if let Some(ref curves) = edge.connector {
                let transform = edge_transform(edge.start, edge.end);
                let first_p0 = transform * curves[0].p0;
                path.move_to(first_p0);
                for curve in curves {
                    let p1 = transform * curve.p1;
                    let p2 = transform * curve.p2;
                    let p3 = transform * curve.p3;
                    path.curve_to(p1, p2, p3);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use crate::shapes::{heart_path, star_path};

    /// Helper to create a valid PuzzleConfig for testing.
    fn test_config(rows: u32, cols: u32, seed: &str) -> PuzzleConfig {
        PuzzleConfig {
            rows,
            cols,
            width: 200.0,
            height: 150.0,
            unit: Unit::Millimeters,
            tab: TabConfig::default(),
            seed: seed.to_string(),
        }
    }

    /// Build a closed rectangle BezPath.
    fn rect_path(x: f64, y: f64, w: f64, h: f64) -> BezPath {
        let mut path = BezPath::new();
        path.move_to(Point::new(x, y));
        path.line_to(Point::new(x + w, y));
        path.line_to(Point::new(x + w, y + h));
        path.line_to(Point::new(x, y + h));
        path.close_path();
        path
    }

    // ─── Cell Classification Tests ───────────────────────────────

    #[test]
    fn test_boundary_all_cells_inside_large_boundary() {
        // A boundary rectangle larger than the grid should include all cells.
        let grid = PuzzleGrid::new(test_config(4, 5, "all-inside")).unwrap();
        let boundary = rect_path(-10.0, -10.0, 220.0, 170.0);
        let bp = BoundaryPuzzle::new(grid, boundary);

        let included = bp.included_cells();
        assert_eq!(
            included.len(),
            4 * 5,
            "all cells should be inside a boundary larger than the grid"
        );
    }

    #[test]
    fn test_boundary_heart_excludes_corner_cells() {
        // Heart shape on a 6×8 grid should exclude some corner cells
        // because the heart curves inward at the corners.
        let config = test_config(6, 8, "heart-corners");
        let grid = PuzzleGrid::new(config.clone()).unwrap();
        let boundary = heart_path(config.width, config.height);
        let bp = BoundaryPuzzle::new(grid, boundary);

        let included = bp.included_cells();
        let total_cells = 6 * 8;

        assert!(
            included.len() < total_cells,
            "heart boundary should exclude some cells: included {} of {}",
            included.len(),
            total_cells
        );
        assert!(
            included.len() > 0,
            "heart boundary should include at least some cells"
        );

        // The top-left corner (0,0) should typically be excluded by the heart shape
        // because the heart dips inward at the top center, and top corners are outside.
        let top_left_included = bp.cell_inside[0][0];
        let top_right_included = bp.cell_inside[0][7];
        assert!(
            !top_left_included || !top_right_included,
            "at least one top corner should be outside the heart shape"
        );
    }

    #[test]
    fn test_boundary_star_excludes_cells() {
        // Star shape on a 6×8 grid should exclude cells in the concavities.
        let config = test_config(6, 8, "star-concavities");
        let grid = PuzzleGrid::new(config.clone()).unwrap();
        let boundary = star_path(config.width, config.height, 5);
        let bp = BoundaryPuzzle::new(grid, boundary);

        let included = bp.included_cells();
        let total_cells = 6 * 8;

        assert!(
            included.len() < total_cells,
            "star boundary should exclude cells in concavities: included {} of {}",
            included.len(),
            total_cells
        );
        assert!(
            included.len() > 0,
            "star boundary should include some cells"
        );
    }

    // ─── Edge Filtering Tests ────────────────────────────────────

    #[test]
    fn test_boundary_included_edges_between_inside_cells() {
        // With a large boundary (all cells inside), included internal edges
        // should match the full grid's internal edge count.
        let config = test_config(4, 5, "edges-all");
        let grid = PuzzleGrid::new(config.clone()).unwrap();
        let boundary = rect_path(-10.0, -10.0, 220.0, 170.0);
        let bp = BoundaryPuzzle::new(grid, boundary);

        let rows = 4usize;
        let cols = 5usize;
        // Internal h-edges in full grid: (rows-1) * cols
        let expected_h = (rows - 1) * cols;
        // Internal v-edges in full grid: rows * (cols-1)
        let expected_v = rows * (cols - 1);

        assert_eq!(
            bp.included_h_edges().len(),
            expected_h,
            "all internal h-edges should be included when boundary covers all cells"
        );
        assert_eq!(
            bp.included_v_edges().len(),
            expected_v,
            "all internal v-edges should be included when boundary covers all cells"
        );
    }

    #[test]
    fn test_boundary_edge_count_less_than_full() {
        // Heart boundary should produce fewer internal edges than the full grid.
        let config = test_config(6, 8, "edges-heart");
        let grid = PuzzleGrid::new(config.clone()).unwrap();

        let rows = 6usize;
        let cols = 8usize;
        let full_internal_h = (rows - 1) * cols;
        let full_internal_v = rows * (cols - 1);
        let full_internal = full_internal_h + full_internal_v;

        let boundary = heart_path(config.width, config.height);
        let bp = BoundaryPuzzle::new(grid, boundary);

        let boundary_internal = bp.included_edge_count();
        assert!(
            boundary_internal < full_internal,
            "heart boundary edges ({}) should be fewer than full grid edges ({})",
            boundary_internal,
            full_internal
        );
        assert!(
            boundary_internal > 0,
            "heart boundary should still have some internal edges"
        );
    }

    #[test]
    fn test_boundary_edge_indices_valid() {
        // All returned edge indices should be valid indices into the grid's
        // h_edges and v_edges arrays.
        let config = test_config(6, 8, "edge-valid");
        let grid = PuzzleGrid::new(config.clone()).unwrap();
        let h_len = grid.h_edges.len();
        let v_len = grid.v_edges.len();

        let boundary = heart_path(config.width, config.height);
        let bp = BoundaryPuzzle::new(grid, boundary);

        for &idx in &bp.included_h_edges() {
            assert!(
                idx < h_len,
                "h_edge index {} out of bounds (max {})",
                idx,
                h_len
            );
        }
        for &idx in &bp.included_v_edges() {
            assert!(
                idx < v_len,
                "v_edge index {} out of bounds (max {})",
                idx,
                v_len
            );
        }
    }

    // ─── Determinism Tests ───────────────────────────────────────

    #[test]
    fn test_boundary_determinism() {
        // Same seed + same boundary = identical cell inclusion and edge lists.
        let config1 = test_config(6, 8, "determinism");
        let config2 = test_config(6, 8, "determinism");

        let grid1 = PuzzleGrid::new(config1.clone()).unwrap();
        let grid2 = PuzzleGrid::new(config2.clone()).unwrap();

        let boundary1 = heart_path(config1.width, config1.height);
        let boundary2 = heart_path(config2.width, config2.height);

        let bp1 = BoundaryPuzzle::new(grid1, boundary1);
        let bp2 = BoundaryPuzzle::new(grid2, boundary2);

        assert_eq!(
            bp1.included_cells(),
            bp2.included_cells(),
            "same seed + same boundary must produce identical cell inclusion"
        );
        assert_eq!(
            bp1.included_h_edges(),
            bp2.included_h_edges(),
            "same seed + same boundary must produce identical h-edge indices"
        );
        assert_eq!(
            bp1.included_v_edges(),
            bp2.included_v_edges(),
            "same seed + same boundary must produce identical v-edge indices"
        );
    }

    // ─── Whimsy Hole Tests ───────────────────────────────────────

    #[test]
    fn test_boundary_hole_removes_center_cells() {
        // Large boundary (all cells inside) + small centered hole should
        // remove center cells.
        let config = test_config(6, 8, "hole-center");
        let grid = PuzzleGrid::new(config.clone()).unwrap();

        let boundary = rect_path(-10.0, -10.0, 220.0, 170.0);
        // Small centered hole
        let hole = rect_path(80.0, 55.0, 40.0, 40.0);

        let bp_no_hole = BoundaryPuzzle::new(
            PuzzleGrid::new(test_config(6, 8, "hole-center")).unwrap(),
            rect_path(-10.0, -10.0, 220.0, 170.0),
        );
        let bp_with_hole = BoundaryPuzzle::new_with_hole(grid, boundary, hole);

        let cells_without_hole = bp_no_hole.included_cells().len();
        let cells_with_hole = bp_with_hole.included_cells().len();

        assert!(
            cells_with_hole < cells_without_hole,
            "hole should remove cells: {} with hole vs {} without",
            cells_with_hole,
            cells_without_hole
        );

        // Verify that at least one center cell was removed
        let rows = 6usize;
        let cols = 8usize;
        let center_row = rows / 2;
        let center_col = cols / 2;
        // Check a few cells near center
        let center_removed = !bp_with_hole.cell_inside[center_row][center_col]
            || !bp_with_hole.cell_inside[center_row - 1][center_col]
            || !bp_with_hole.cell_inside[center_row][center_col - 1];
        assert!(
            center_removed,
            "at least one center cell should be removed by the hole"
        );
    }

    // ─── Empty Boundary Edge Case ────────────────────────────────

    #[test]
    fn test_boundary_no_cells_inside_tiny_boundary() {
        // A tiny boundary that doesn't contain any cell centers.
        let config = test_config(6, 8, "tiny-boundary");
        let grid = PuzzleGrid::new(config).unwrap();

        // 1×1 pixel boundary in the corner — no cell center falls inside it.
        let boundary = rect_path(0.0, 0.0, 0.5, 0.5);
        let bp = BoundaryPuzzle::new(grid, boundary);

        assert_eq!(
            bp.included_cells().len(),
            0,
            "tiny boundary should include no cells"
        );
        assert_eq!(
            bp.included_h_edges().len(),
            0,
            "no included cells means no included h-edges"
        );
        assert_eq!(
            bp.included_v_edges().len(),
            0,
            "no included cells means no included v-edges"
        );
        assert_eq!(
            bp.included_edge_count(),
            0,
            "total included edges should be zero"
        );
    }

    // ─── SVG Export Tests ────────────────────────────────────────

    /// Helper to create a BoundaryPuzzle with connectors generated.
    fn make_boundary_puzzle(rows: u32, cols: u32, seed: &str, boundary: BezPath) -> BoundaryPuzzle {
        let config = test_config(rows, cols, seed);
        let mut grid = PuzzleGrid::new(config).unwrap();
        grid.generate_connectors(&crate::classic_connector::ClassicKnobConnector);
        BoundaryPuzzle::new(grid, boundary)
    }

    #[test]
    fn test_boundary_svg_contains_cubic_curves() {
        // Heart boundary SVG should have C commands from the curved border.
        let config = test_config(6, 8, "svg-cubic");
        let boundary = heart_path(config.width, config.height);
        let bp = make_boundary_puzzle(6, 8, "svg-cubic", boundary);

        let svg = bp.generate_boundary_svg();
        let d_start = svg.find("d='").expect("should have d attribute") + 3;
        let d_end = svg[d_start..].find('\'').unwrap() + d_start;
        let path_data = &svg[d_start..d_end];

        assert!(
            path_data.contains('C'),
            "heart boundary SVG should contain C (cubic bezier) commands in border section"
        );
    }

    #[test]
    fn test_boundary_svg_excludes_outside_edges() {
        // SVG M-command count should match included edge count + 1 (border).
        let config = test_config(6, 8, "svg-exclude");
        let boundary = heart_path(config.width, config.height);
        let bp = make_boundary_puzzle(6, 8, "svg-exclude", boundary);

        let svg = bp.generate_boundary_svg();
        let d_start = svg.find("d='").unwrap() + 3;
        let d_end = svg[d_start..].find('\'').unwrap() + d_start;
        let path_data = &svg[d_start..d_end];

        let m_count = path_data.matches('M').count();
        let expected_m = bp.included_edge_count() + 1; // +1 for border MoveTo

        assert_eq!(
            m_count, expected_m,
            "M-command count ({}) should equal included edges ({}) + 1 (border)",
            m_count,
            bp.included_edge_count()
        );
    }

    #[test]
    fn test_boundary_svg_deterministic() {
        // Same seed + boundary = identical SVG.
        let config1 = test_config(6, 8, "svg-determ");
        let boundary1 = heart_path(config1.width, config1.height);
        let bp1 = make_boundary_puzzle(6, 8, "svg-determ", boundary1);

        let config2 = test_config(6, 8, "svg-determ");
        let boundary2 = heart_path(config2.width, config2.height);
        let bp2 = make_boundary_puzzle(6, 8, "svg-determ", boundary2);

        let svg1 = bp1.generate_boundary_svg();
        let svg2 = bp2.generate_boundary_svg();
        assert_eq!(svg1, svg2, "same seed + boundary must produce identical SVG");
    }

    #[test]
    fn test_boundary_svg_has_valid_structure() {
        // SVG should have proper wrapper elements.
        let config = test_config(6, 8, "svg-struct");
        let boundary = heart_path(config.width, config.height);
        let bp = make_boundary_puzzle(6, 8, "svg-struct", boundary);

        let svg = bp.generate_boundary_svg();
        assert!(svg.contains("<svg"), "should contain <svg element");
        assert!(svg.contains("<path"), "should contain <path element");
        assert!(svg.contains("width='200mm'"), "should have mm width");
        assert!(svg.contains("height='150mm'"), "should have mm height");
        assert!(svg.contains("stroke='#000000'"), "should have black stroke");
        assert!(svg.contains("fill='none'"), "should have no fill");
    }

    // ─── Binary Export Tests ─────────────────────────────────────

    #[test]
    fn test_boundary_binary_edge_count() {
        // Binary edge count should match included_h_edges + included_v_edges count.
        let config = test_config(6, 8, "bin-count");
        let boundary = heart_path(config.width, config.height);
        let bp = make_boundary_puzzle(6, 8, "bin-count", boundary);

        let data = bp.boundary_edges_to_binary();
        let expected_count = bp.included_edge_count();

        assert_eq!(
            data.len() % EDGE_STRIDE,
            0,
            "binary data length {} not divisible by stride {}",
            data.len(),
            EDGE_STRIDE
        );

        let actual_count = data.len() / EDGE_STRIDE;
        assert_eq!(
            actual_count, expected_count,
            "binary edge count ({}) should match included_edge_count ({})",
            actual_count, expected_count
        );
    }

    #[test]
    fn test_boundary_binary_border_starts_with_moveto() {
        // Border binary should start with CMD_MOVE_TO.
        let config = test_config(6, 8, "bin-moveto");
        let boundary = heart_path(config.width, config.height);
        let bp = make_boundary_puzzle(6, 8, "bin-moveto", boundary);

        let data = bp.boundary_border_to_binary();
        assert!(!data.is_empty(), "border binary should not be empty");
        assert_eq!(
            data[0], 0.0, // CMD_MOVE_TO
            "border binary should start with CMD_MOVE_TO (0.0), got {}",
            data[0]
        );
    }

    #[test]
    fn test_boundary_binary_border_has_curves() {
        // Heart border binary should contain CMD_CURVE_TO commands.
        let config = test_config(6, 8, "bin-curves");
        let boundary = heart_path(config.width, config.height);
        let bp = make_boundary_puzzle(6, 8, "bin-curves", boundary);

        let data = bp.boundary_border_to_binary();
        assert!(
            data.contains(&2.0), // CMD_CURVE_TO
            "heart border binary should contain CMD_CURVE_TO (2.0) commands"
        );
    }

    #[test]
    fn test_boundary_binary_border_has_close() {
        // Border binary should contain CMD_CLOSE.
        let config = test_config(6, 8, "bin-close");
        let boundary = heart_path(config.width, config.height);
        let bp = make_boundary_puzzle(6, 8, "bin-close", boundary);

        let data = bp.boundary_border_to_binary();
        assert!(
            data.contains(&3.0), // CMD_CLOSE
            "border binary should contain CMD_CLOSE (3.0) command"
        );
    }

    #[test]
    fn test_boundary_binary_edges_deterministic() {
        // Same seed + boundary = identical binary edge output.
        let config1 = test_config(6, 8, "bin-determ");
        let boundary1 = heart_path(config1.width, config1.height);
        let bp1 = make_boundary_puzzle(6, 8, "bin-determ", boundary1);

        let config2 = test_config(6, 8, "bin-determ");
        let boundary2 = heart_path(config2.width, config2.height);
        let bp2 = make_boundary_puzzle(6, 8, "bin-determ", boundary2);

        let data1 = bp1.boundary_edges_to_binary();
        let data2 = bp2.boundary_edges_to_binary();
        assert_eq!(data1.len(), data2.len(), "binary lengths should match");
        for (i, (a, b)) in data1.iter().zip(data2.iter()).enumerate() {
            assert!(
                (a - b).abs() < 1e-10,
                "binary data mismatch at index {}: {} vs {}",
                i,
                a,
                b
            );
        }
    }
}
