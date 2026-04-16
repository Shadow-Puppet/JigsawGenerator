use kurbo::PathEl;

use crate::grid::PuzzleGrid;
use crate::svg_export::{build_border_path, edge_transform};

/// Number of f64 values per internal edge in the binary format.
///
/// Layout per edge (36 floats):
/// - `[0..4]`: start.x, start.y, end.x, end.y (bounding hint for viewport culling)
/// - `[4..6]`: moveTo x, y (first curve's p0, transformed to global coords)
/// - `[6..36]`: 5 curves × 6 floats (p1.x, p1.y, p2.x, p2.y, p3.x, p3.y)
pub const EDGE_STRIDE: usize = 36;

/// Serialize all internal edge connector curves as a flat f64 array.
///
/// Each internal edge is encoded as a fixed-stride chunk of 36 f64 values.
/// All coordinates are in mm (puzzle coordinate space), transformed from
/// edge-local to global coordinates.
///
/// Edges are iterated in the same order as SVG export: internal horizontal
/// edges (rows 1..rows, all cols) then internal vertical edges (all rows,
/// cols 1..cols).
pub fn edges_to_binary(grid: &PuzzleGrid) -> Vec<f64> {
    let rows = grid.config.rows as usize;
    let cols = grid.config.cols as usize;

    // Estimate capacity: internal h_edges + internal v_edges
    let est_internal = (rows - 1) * cols + rows * (cols - 1);
    let mut data = Vec::with_capacity(est_internal * EDGE_STRIDE);

    // Internal horizontal edges
    for row in 1..rows {
        for col in 0..cols {
            let edge = grid.h_edge(row, col);
            if edge.is_border {
                continue;
            }
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

                // 5 curves × 6 floats (p1, p2, p3 — p0 is implicit from previous curve)
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
    }

    // Internal vertical edges
    for row in 0..rows {
        for col in 1..cols {
            let edge = grid.v_edge(row, col);
            if edge.is_border {
                continue;
            }
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

                // 5 curves × 6 floats
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
    }

    data
}

/// Command type constants for border binary encoding.
pub(crate) const CMD_MOVE_TO: f64 = 0.0;
pub(crate) const CMD_LINE_TO: f64 = 1.0;
pub(crate) const CMD_CURVE_TO: f64 = 2.0;
pub(crate) const CMD_CLOSE: f64 = 3.0;

/// Serialize the border path as a command-prefixed f64 array.
///
/// Commands:
/// - `0.0` (moveTo) + x, y (3 floats)
/// - `1.0` (lineTo) + x, y (3 floats)
/// - `2.0` (curveTo) + p1.x, p1.y, p2.x, p2.y, p3.x, p3.y (7 floats)
/// - `3.0` (closePath) (1 float)
pub fn border_to_binary(grid: &PuzzleGrid) -> Vec<f64> {
    let border = build_border_path(grid);
    let mut data = Vec::with_capacity(128);

    for el in border.iter() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classic_connector::ClassicKnobConnector;
    use crate::config::*;

    fn test_config(rows: u32, cols: u32, seed: &str) -> PuzzleConfig {
        PuzzleConfig {
            rows,
            cols,
            width: 200.0,
            height: 150.0,
            unit: Unit::Millimeters,
            tab: TabConfig::default(),
            seed: seed.to_string(),
            border_shape: None,
        }
    }

    fn make_grid(rows: u32, cols: u32, seed: &str) -> PuzzleGrid {
        let config = test_config(rows, cols, seed);
        let mut grid = PuzzleGrid::new(config).unwrap();
        grid.generate_connectors(&ClassicKnobConnector);
        grid
    }

    #[test]
    fn test_edges_to_binary_stride() {
        let grid = make_grid(3, 4, "binary-stride");
        let data = edges_to_binary(&grid);
        // Must be a multiple of EDGE_STRIDE
        assert_eq!(
            data.len() % EDGE_STRIDE,
            0,
            "data length {} not divisible by stride {}",
            data.len(),
            EDGE_STRIDE
        );
    }

    #[test]
    fn test_edges_to_binary_count() {
        let grid = make_grid(3, 4, "binary-count");
        let data = edges_to_binary(&grid);
        let edge_count = data.len() / EDGE_STRIDE;
        // 3x4 grid: internal h_edges = 2 rows * 4 cols = 8
        //           internal v_edges = 3 rows * 3 cols = 9
        //           total = 17
        assert_eq!(
            edge_count, 17,
            "3x4 grid should have 17 internal edges, got {}",
            edge_count
        );
    }

    #[test]
    fn test_edges_to_binary_deterministic() {
        let grid1 = make_grid(3, 4, "determ");
        let grid2 = make_grid(3, 4, "determ");
        let data1 = edges_to_binary(&grid1);
        let data2 = edges_to_binary(&grid2);
        assert_eq!(data1.len(), data2.len());
        for (i, (a, b)) in data1.iter().zip(data2.iter()).enumerate() {
            assert!(
                (a - b).abs() < 1e-10,
                "data mismatch at index {}: {} vs {}",
                i,
                a,
                b
            );
        }
    }

    #[test]
    fn test_border_to_binary_nonempty() {
        let grid = make_grid(3, 4, "border-bin");
        let data = border_to_binary(&grid);
        assert!(!data.is_empty(), "border binary data should not be empty");
    }

    #[test]
    fn test_border_to_binary_starts_with_moveto() {
        let grid = make_grid(3, 4, "border-start");
        let data = border_to_binary(&grid);
        assert_eq!(
            data[0], CMD_MOVE_TO,
            "border should start with moveTo command"
        );
    }

    #[test]
    fn test_border_to_binary_has_close() {
        let grid = make_grid(3, 4, "border-close");
        let data = border_to_binary(&grid);
        assert!(
            data.contains(&CMD_CLOSE),
            "border should contain a closePath command"
        );
    }

    #[test]
    fn test_border_to_binary_deterministic() {
        let grid1 = make_grid(3, 4, "border-det");
        let grid2 = make_grid(3, 4, "border-det");
        let data1 = border_to_binary(&grid1);
        let data2 = border_to_binary(&grid2);
        assert_eq!(data1.len(), data2.len());
        for (i, (a, b)) in data1.iter().zip(data2.iter()).enumerate() {
            assert!(
                (a - b).abs() < 1e-10,
                "border data mismatch at index {}: {} vs {}",
                i,
                a,
                b
            );
        }
    }

    #[test]
    fn test_edges_to_binary_2x2_minimum() {
        let grid = make_grid(2, 2, "2x2-bin");
        let data = edges_to_binary(&grid);
        let edge_count = data.len() / EDGE_STRIDE;
        // 2x2: internal h_edges = 1 row * 2 cols = 2
        //       internal v_edges = 2 rows * 1 col = 2
        //       total = 4
        assert_eq!(
            edge_count, 4,
            "2x2 grid should have 4 internal edges, got {}",
            edge_count
        );
    }
}
