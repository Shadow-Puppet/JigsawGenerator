use kurbo::Point;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;

use crate::config::PuzzleConfig;
use crate::connector::ConnectorGenerator;
use crate::edge::{Edge, TabDirection};
use crate::piece::{Piece, PieceEdges, PieceType};
use crate::seed::create_rng;

/// Assign tab direction: border edges always In, internal edges random.
fn assign_direction(is_border: bool, rng: &mut ChaCha8Rng) -> TabDirection {
    if is_border {
        TabDirection::In
    } else if rng.random_bool(0.5) {
        TabDirection::In
    } else {
        TabDirection::Out
    }
}

/// A puzzle grid with shared-edge data model.
///
/// Edges between adjacent pieces are stored exactly once in shared arrays.
/// `h_edges` stores horizontal edges (top/bottom of pieces),
/// `v_edges` stores vertical edges (left/right of pieces).
///
/// For an NxM grid (rows x cols):
/// - h_edges has (rows+1) * cols elements (row-major order)
/// - v_edges has rows * (cols+1) elements (row-major order)
pub struct PuzzleGrid {
    pub config: PuzzleConfig,
    pub h_edges: Vec<Edge>,
    pub v_edges: Vec<Edge>,
}

impl PuzzleGrid {
    /// Construct a new PuzzleGrid from a validated PuzzleConfig.
    ///
    /// RNG iteration order (CRITICAL for determinism):
    /// 1. Iterate h_edges: row 0..=rows, col 0..cols
    ///    - border rows (row == 0 || row == rows): direction = In
    ///    - internal rows: direction = random from seeded RNG
    /// 2. Then iterate v_edges: row 0..rows, col 0..=cols
    ///    - border cols (col == 0 || col == cols): direction = In
    ///    - internal cols: direction = random from seeded RNG
    pub fn new(config: PuzzleConfig) -> Result<Self, String> {
        config.validate()?;

        let rows = config.rows as usize;
        let cols = config.cols as usize;
        let cell_w = config.width / cols as f64;
        let cell_h = config.height / rows as f64;

        let mut rng = create_rng(&config.seed);

        // Build horizontal edges: (rows+1) rows, cols columns
        let mut h_edges = Vec::with_capacity((rows + 1) * cols);
        for row in 0..=rows {
            for col in 0..cols {
                let is_border = row == 0 || row == rows;
                h_edges.push(Edge {
                    start: Point::new(col as f64 * cell_w, row as f64 * cell_h),
                    end: Point::new((col + 1) as f64 * cell_w, row as f64 * cell_h),
                    is_border,
                    direction: assign_direction(is_border, &mut rng),
                    connector: None,
                });
            }
        }

        // Build vertical edges: rows rows, (cols+1) columns
        let mut v_edges = Vec::with_capacity(rows * (cols + 1));
        for row in 0..rows {
            for col in 0..=cols {
                let is_border = col == 0 || col == cols;
                v_edges.push(Edge {
                    start: Point::new(col as f64 * cell_w, row as f64 * cell_h),
                    end: Point::new(col as f64 * cell_w, (row + 1) as f64 * cell_h),
                    is_border,
                    direction: assign_direction(is_border, &mut rng),
                    connector: None,
                });
            }
        }

        Ok(PuzzleGrid {
            config,
            h_edges,
            v_edges,
        })
    }

    /// Get a reference to the horizontal edge at grid position (row, col).
    ///
    /// row ranges from 0..=rows, col ranges from 0..cols.
    pub fn h_edge(&self, row: usize, col: usize) -> &Edge {
        &self.h_edges[row * self.config.cols as usize + col]
    }

    /// Get a reference to the vertical edge at grid position (row, col).
    ///
    /// row ranges from 0..rows, col ranges from 0..=cols.
    pub fn v_edge(&self, row: usize, col: usize) -> &Edge {
        &self.v_edges[row * (self.config.cols as usize + 1) + col]
    }

    /// Get the edge indices for the piece at (row, col).
    pub fn piece_edges(&self, row: usize, col: usize) -> PieceEdges {
        let cols = self.config.cols as usize;
        PieceEdges {
            top: row * cols + col,
            bottom: (row + 1) * cols + col,
            left: row * (cols + 1) + col,
            right: row * (cols + 1) + (col + 1),
        }
    }

    /// Classify a piece based on its border edge count.
    pub fn piece_type(&self, row: usize, col: usize) -> PieceType {
        let rows = self.config.rows as usize;
        let cols = self.config.cols as usize;
        let border_count = [row == 0, row == rows - 1, col == 0, col == cols - 1]
            .iter()
            .filter(|&&b| b)
            .count();
        match border_count {
            2 => PieceType::Corner,
            1 => PieceType::Edge,
            _ => PieceType::Interior,
        }
    }

    /// Compute the safe maximum tab size based on grid dimensions.
    ///
    /// Prevents opposing tabs from overlapping by ensuring:
    /// - H-edge tabs don't exceed cell height (protrude vertically)
    /// - V-edge tabs don't exceed cell width (protrude horizontally)
    /// - Approach span doesn't exceed edge length
    ///
    /// Returns the clamped max with a 90% safety margin.
    pub fn safe_tab_max(&self) -> f64 {
        let rows = self.config.rows as f64;
        let cols = self.config.cols as f64;
        let cell_w = self.config.width / cols;
        let cell_h = self.config.height / rows;

        // Knob height ratio used in connector generation
        const KNOB_HEIGHT_RATIO: f64 = 1.2;
        const APPROACH_RATIO: f64 = 1.2;

        // Constraint 1: H-edge tabs protrude into cell_h
        // 2 * cell_w * pct * KNOB_HEIGHT_RATIO < cell_h
        let max_h = cell_h / (2.0 * cell_w * KNOB_HEIGHT_RATIO);

        // Constraint 2: V-edge tabs protrude into cell_w
        // 2 * cell_h * pct * KNOB_HEIGHT_RATIO < cell_w
        let max_v = cell_w / (2.0 * cell_h * KNOB_HEIGHT_RATIO);

        // Constraint 3: Approach span must fit within edge
        // 2 * pct * APPROACH_RATIO < 1
        let max_approach = 1.0 / (2.0 * APPROACH_RATIO);

        let theoretical_max = max_h.min(max_v).min(max_approach);

        // Apply 90% safety margin, cap at 0.25.
        // No floor: extreme aspect ratios may need very small tabs to avoid overlap.
        // The UI-side TabConfig validation handles the display range.
        (theoretical_max * 0.9).min(0.25)
    }

    /// Populate connector geometry on all internal edges.
    ///
    /// Uses the given `ConnectorGenerator` to produce bezier curves for each
    /// internal edge. Border edges remain `connector: None`.
    ///
    /// Tab size is dynamically clamped to `safe_tab_max()` to prevent overlap.
    ///
    /// RNG iteration order matches grid construction: h_edges row-major,
    /// then v_edges row-major. A fresh RNG is created from the config seed
    /// with a suffix to avoid overlapping with the construction RNG sequence.
    pub fn generate_connectors(&mut self, connector: &dyn ConnectorGenerator) {
        use crate::edge::EdgeParams;

        // Create a separate RNG for connector generation (deterministic, independent of grid RNG)
        let mut rng = create_rng(&format!("{}-connectors", self.config.seed));

        // Clamp tab size to safe maximum for this grid's dimensions
        let safe_max = self.safe_tab_max();

        // Compute cell dimensions for cross_length
        let rows = self.config.rows as f64;
        let cols = self.config.cols as f64;
        let cell_w = self.config.width / cols;
        let cell_h = self.config.height / rows;

        // Generate connectors for horizontal edges
        // H-edges have length=cell_w, cross_length=cell_h (knobs protrude into cell height)
        for edge in &mut self.h_edges {
            if edge.is_border {
                continue;
            }
            let tab_size = self.config.tab.randomize_tab_size(safe_max, &mut rng);
            let neck_ratio = self.config.tab.randomize_neck_ratio(&mut rng);
            let params = EdgeParams {
                length: edge.length(),
                cross_length: cell_h,
                direction: edge.direction,
                tab_size,
                neck_ratio,
            };
            let curves = connector.generate(&params, &mut rng);
            edge.connector = Some(curves);
        }

        // Generate connectors for vertical edges
        // V-edges have length=cell_h, cross_length=cell_w (knobs protrude into cell width)
        for edge in &mut self.v_edges {
            if edge.is_border {
                continue;
            }
            let tab_size = self.config.tab.randomize_tab_size(safe_max, &mut rng);
            let neck_ratio = self.config.tab.randomize_neck_ratio(&mut rng);
            let params = EdgeParams {
                length: edge.length(),
                cross_length: cell_w,
                direction: edge.direction,
                tab_size,
                neck_ratio,
            };
            let curves = connector.generate(&params, &mut rng);
            edge.connector = Some(curves);
        }
    }

    /// Generate all pieces in the grid.
    pub fn pieces(&self) -> Vec<Piece> {
        let rows = self.config.rows as usize;
        let cols = self.config.cols as usize;
        let mut pieces = Vec::with_capacity(rows * cols);
        for row in 0..rows {
            for col in 0..cols {
                pieces.push(Piece {
                    row,
                    col,
                    piece_type: self.piece_type(row, col),
                    edges: self.piece_edges(row, col),
                });
            }
        }
        pieces
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    /// Helper to create a valid PuzzleConfig for testing.
    fn test_config(rows: u32, cols: u32, seed: &str) -> PuzzleConfig {
        PuzzleConfig {
            rows,
            cols,
            width: 200.0,
            height: 150.0,
            unit: Unit::Millimeters,
            tab: TabConfig::default(),
            border: BorderConfig::default(),
            seed: seed.to_string(),
        }
    }

    // ─── Edge Count Tests ────────────────────────────────────────

    #[test]
    fn test_2x2_edge_counts() {
        let grid = PuzzleGrid::new(test_config(2, 2, "test")).unwrap();
        // h_edges = (2+1)*2 = 6
        assert_eq!(grid.h_edges.len(), 6);
        // v_edges = 2*(2+1) = 6
        assert_eq!(grid.v_edges.len(), 6);
    }

    #[test]
    fn test_3x4_edge_counts() {
        let grid = PuzzleGrid::new(test_config(3, 4, "test")).unwrap();
        // h_edges = (3+1)*4 = 16
        assert_eq!(grid.h_edges.len(), 16);
        // v_edges = 3*(4+1) = 15
        assert_eq!(grid.v_edges.len(), 15);
    }

    #[test]
    fn test_6x8_edge_counts() {
        let grid = PuzzleGrid::new(test_config(6, 8, "test")).unwrap();
        // h_edges = (6+1)*8 = 56
        assert_eq!(grid.h_edges.len(), 56);
        // v_edges = 6*(8+1) = 54
        assert_eq!(grid.v_edges.len(), 54);
    }

    #[test]
    fn test_2x3_edge_counts() {
        let grid = PuzzleGrid::new(test_config(2, 3, "test")).unwrap();
        // h_edges = (2+1)*3 = 9
        assert_eq!(grid.h_edges.len(), 9);
        // v_edges = 2*(3+1) = 8
        assert_eq!(grid.v_edges.len(), 8);
    }

    #[test]
    fn test_3x3_edge_counts() {
        let grid = PuzzleGrid::new(test_config(3, 3, "test")).unwrap();
        // h_edges = (3+1)*3 = 12
        assert_eq!(grid.h_edges.len(), 12);
        // v_edges = 3*(3+1) = 12
        assert_eq!(grid.v_edges.len(), 12);
    }

    // ─── Edge Coordinate Tests ───────────────────────────────────

    #[test]
    fn test_h_edge_coordinates() {
        let grid = PuzzleGrid::new(test_config(3, 4, "coord-test")).unwrap();
        let cell_w = 200.0 / 4.0; // 50.0
        let cell_h = 150.0 / 3.0; // 50.0

        // h_edge at (0, 0) = top-left horizontal
        let e = grid.h_edge(0, 0);
        assert!((e.start.x - 0.0).abs() < 1e-10);
        assert!((e.start.y - 0.0).abs() < 1e-10);
        assert!((e.end.x - cell_w).abs() < 1e-10);
        assert!((e.end.y - 0.0).abs() < 1e-10);

        // h_edge at (1, 2) = row 1, col 2
        let e = grid.h_edge(1, 2);
        assert!((e.start.x - 2.0 * cell_w).abs() < 1e-10);
        assert!((e.start.y - 1.0 * cell_h).abs() < 1e-10);
        assert!((e.end.x - 3.0 * cell_w).abs() < 1e-10);
        assert!((e.end.y - 1.0 * cell_h).abs() < 1e-10);

        // h_edge at (3, 3) = bottom-right horizontal
        let e = grid.h_edge(3, 3);
        assert!((e.start.x - 3.0 * cell_w).abs() < 1e-10);
        assert!((e.start.y - 3.0 * cell_h).abs() < 1e-10);
        assert!((e.end.x - 4.0 * cell_w).abs() < 1e-10);
        assert!((e.end.y - 3.0 * cell_h).abs() < 1e-10);
    }

    #[test]
    fn test_v_edge_coordinates() {
        let grid = PuzzleGrid::new(test_config(3, 4, "coord-test")).unwrap();
        let cell_w = 200.0 / 4.0; // 50.0
        let cell_h = 150.0 / 3.0; // 50.0

        // v_edge at (0, 0) = top-left vertical
        let e = grid.v_edge(0, 0);
        assert!((e.start.x - 0.0).abs() < 1e-10);
        assert!((e.start.y - 0.0).abs() < 1e-10);
        assert!((e.end.x - 0.0).abs() < 1e-10);
        assert!((e.end.y - cell_h).abs() < 1e-10);

        // v_edge at (2, 4) = bottom-right vertical (col=4 = right border)
        let e = grid.v_edge(2, 4);
        assert!((e.start.x - 4.0 * cell_w).abs() < 1e-10);
        assert!((e.start.y - 2.0 * cell_h).abs() < 1e-10);
        assert!((e.end.x - 4.0 * cell_w).abs() < 1e-10);
        assert!((e.end.y - 3.0 * cell_h).abs() < 1e-10);
    }

    // ─── Border Detection Tests ──────────────────────────────────

    #[test]
    fn test_border_detection_h_edges() {
        let grid = PuzzleGrid::new(test_config(3, 4, "border-test")).unwrap();
        // Top row (row=0): all border
        for col in 0..4 {
            assert!(
                grid.h_edge(0, col).is_border,
                "h_edge(0, {col}) should be border"
            );
        }
        // Bottom row (row=rows=3): all border
        for col in 0..4 {
            assert!(
                grid.h_edge(3, col).is_border,
                "h_edge(3, {col}) should be border"
            );
        }
        // Internal rows: not border
        for row in 1..3 {
            for col in 0..4 {
                assert!(
                    !grid.h_edge(row, col).is_border,
                    "h_edge({row}, {col}) should NOT be border"
                );
            }
        }
    }

    #[test]
    fn test_border_detection_v_edges() {
        let grid = PuzzleGrid::new(test_config(3, 4, "border-test")).unwrap();
        // Left col (col=0): all border
        for row in 0..3 {
            assert!(
                grid.v_edge(row, 0).is_border,
                "v_edge({row}, 0) should be border"
            );
        }
        // Right col (col=cols=4): all border
        for row in 0..3 {
            assert!(
                grid.v_edge(row, 4).is_border,
                "v_edge({row}, 4) should be border"
            );
        }
        // Internal cols: not border
        for row in 0..3 {
            for col in 1..4 {
                assert!(
                    !grid.v_edge(row, col).is_border,
                    "v_edge({row}, {col}) should NOT be border"
                );
            }
        }
    }

    // ─── Tab Direction Tests ─────────────────────────────────────

    #[test]
    fn test_border_edges_direction_in() {
        let grid = PuzzleGrid::new(test_config(3, 4, "tab-test")).unwrap();
        // All border h_edges have direction=In
        for col in 0..4 {
            assert_eq!(grid.h_edge(0, col).direction, TabDirection::In);
            assert_eq!(grid.h_edge(3, col).direction, TabDirection::In);
        }
        // All border v_edges have direction=In
        for row in 0..3 {
            assert_eq!(grid.v_edge(row, 0).direction, TabDirection::In);
            assert_eq!(grid.v_edge(row, 4).direction, TabDirection::In);
        }
    }

    #[test]
    fn test_seed_determinism() {
        let grid1 = PuzzleGrid::new(test_config(4, 5, "my-puzzle")).unwrap();
        let grid2 = PuzzleGrid::new(test_config(4, 5, "my-puzzle")).unwrap();

        // Same seed → same directions
        for (e1, e2) in grid1.h_edges.iter().zip(grid2.h_edges.iter()) {
            assert_eq!(e1.direction, e2.direction);
            assert_eq!(e1.is_border, e2.is_border);
        }
        for (e1, e2) in grid1.v_edges.iter().zip(grid2.v_edges.iter()) {
            assert_eq!(e1.direction, e2.direction);
            assert_eq!(e1.is_border, e2.is_border);
        }
    }

    #[test]
    fn test_different_seeds_differ() {
        let grid1 = PuzzleGrid::new(test_config(4, 5, "seed-alpha")).unwrap();
        let grid2 = PuzzleGrid::new(test_config(4, 5, "seed-beta")).unwrap();

        // At least one internal edge should have different tab direction
        let any_diff = grid1
            .h_edges
            .iter()
            .zip(grid2.h_edges.iter())
            .chain(grid1.v_edges.iter().zip(grid2.v_edges.iter()))
            .filter(|(e1, _)| !e1.is_border)
            .any(|(e1, e2)| e1.direction != e2.direction);
        assert!(
            any_diff,
            "Different seeds should produce different tab directions"
        );
    }

    // ─── Shared-Edge Proof Tests ─────────────────────────────────

    #[test]
    fn test_shared_edge_horizontal_adjacent() {
        // Piece (0,0) bottom == Piece (1,0) top
        let grid = PuzzleGrid::new(test_config(3, 3, "shared")).unwrap();
        let p00 = grid.piece_edges(0, 0);
        let p10 = grid.piece_edges(1, 0);
        assert_eq!(
            p00.bottom, p10.top,
            "adjacent pieces must share horizontal edge index"
        );
    }

    #[test]
    fn test_shared_edge_vertical_adjacent() {
        // Piece (0,0) right == Piece (0,1) left
        let grid = PuzzleGrid::new(test_config(3, 3, "shared")).unwrap();
        let p00 = grid.piece_edges(0, 0);
        let p01 = grid.piece_edges(0, 1);
        assert_eq!(
            p00.right, p01.left,
            "adjacent pieces must share vertical edge index"
        );
    }

    #[test]
    fn test_shared_edge_all_adjacencies() {
        let grid = PuzzleGrid::new(test_config(4, 5, "full-check")).unwrap();
        let rows = 4usize;
        let cols = 5usize;

        // Check all horizontal adjacencies: piece(r,c).bottom == piece(r+1,c).top
        for r in 0..rows - 1 {
            for c in 0..cols {
                let upper = grid.piece_edges(r, c);
                let lower = grid.piece_edges(r + 1, c);
                assert_eq!(
                    upper.bottom,
                    lower.top,
                    "h-shared edge failed at ({r},{c})/({},{})",
                    r + 1,
                    c
                );
            }
        }

        // Check all vertical adjacencies: piece(r,c).right == piece(r,c+1).left
        for r in 0..rows {
            for c in 0..cols - 1 {
                let left = grid.piece_edges(r, c);
                let right = grid.piece_edges(r, c + 1);
                assert_eq!(
                    left.right,
                    right.left,
                    "v-shared edge failed at ({r},{c})/({r},{})",
                    c + 1
                );
            }
        }
    }

    // ─── Piece Type Tests ────────────────────────────────────────

    #[test]
    fn test_piece_type_corners() {
        let grid = PuzzleGrid::new(test_config(3, 4, "type-test")).unwrap();
        assert_eq!(grid.piece_type(0, 0), PieceType::Corner);
        assert_eq!(grid.piece_type(0, 3), PieceType::Corner);
        assert_eq!(grid.piece_type(2, 0), PieceType::Corner);
        assert_eq!(grid.piece_type(2, 3), PieceType::Corner);
    }

    #[test]
    fn test_piece_type_edges() {
        let grid = PuzzleGrid::new(test_config(3, 4, "type-test")).unwrap();
        // Top edge (not corners)
        assert_eq!(grid.piece_type(0, 1), PieceType::Edge);
        assert_eq!(grid.piece_type(0, 2), PieceType::Edge);
        // Bottom edge
        assert_eq!(grid.piece_type(2, 1), PieceType::Edge);
        // Left edge
        assert_eq!(grid.piece_type(1, 0), PieceType::Edge);
        // Right edge
        assert_eq!(grid.piece_type(1, 3), PieceType::Edge);
    }

    #[test]
    fn test_piece_type_interior() {
        let grid = PuzzleGrid::new(test_config(3, 4, "type-test")).unwrap();
        assert_eq!(grid.piece_type(1, 1), PieceType::Interior);
        assert_eq!(grid.piece_type(1, 2), PieceType::Interior);
    }

    #[test]
    fn test_piece_type_counts_match_breakdown() {
        use crate::{compute_piece_breakdown, GridConfig};

        // Test several grid sizes
        for (rows, cols) in [(2, 2), (3, 4), (4, 5), (6, 8), (10, 10)] {
            let grid = PuzzleGrid::new(test_config(rows, cols, "count-test")).unwrap();
            let expected = compute_piece_breakdown(&GridConfig { rows, cols }).unwrap();
            let pieces = grid.pieces();

            let corners = pieces
                .iter()
                .filter(|p| p.piece_type == PieceType::Corner)
                .count() as u32;
            let edges = pieces
                .iter()
                .filter(|p| p.piece_type == PieceType::Edge)
                .count() as u32;
            let interior = pieces
                .iter()
                .filter(|p| p.piece_type == PieceType::Interior)
                .count() as u32;

            assert_eq!(
                corners, expected.corners,
                "corners mismatch for {rows}x{cols}"
            );
            assert_eq!(edges, expected.edges, "edges mismatch for {rows}x{cols}");
            assert_eq!(
                interior, expected.interior,
                "interior mismatch for {rows}x{cols}"
            );
            assert_eq!(
                pieces.len() as u32,
                expected.total,
                "total mismatch for {rows}x{cols}"
            );
        }
    }

    // ─── Pieces Method Tests ─────────────────────────────────────

    #[test]
    fn test_pieces_count() {
        let grid = PuzzleGrid::new(test_config(3, 4, "pieces-test")).unwrap();
        assert_eq!(grid.pieces().len(), 12);
    }

    #[test]
    fn test_pieces_row_col_ordering() {
        let grid = PuzzleGrid::new(test_config(3, 4, "order-test")).unwrap();
        let pieces = grid.pieces();
        // Should be row-major: (0,0), (0,1), (0,2), (0,3), (1,0), ...
        assert_eq!((pieces[0].row, pieces[0].col), (0, 0));
        assert_eq!((pieces[1].row, pieces[1].col), (0, 1));
        assert_eq!((pieces[4].row, pieces[4].col), (1, 0));
        assert_eq!((pieces[11].row, pieces[11].col), (2, 3));
    }

    // ─── Validation Propagation Test ─────────────────────────────

    #[test]
    fn test_invalid_config_rejected() {
        let mut config = test_config(3, 4, "test");
        config.rows = 1; // below minimum
        assert!(PuzzleGrid::new(config).is_err());
    }

    // ─── Connector Field Test ────────────────────────────────────

    #[test]
    fn test_edges_have_no_connector() {
        let grid = PuzzleGrid::new(test_config(2, 2, "conn-test")).unwrap();
        for e in &grid.h_edges {
            assert!(e.connector.is_none());
        }
        for e in &grid.v_edges {
            assert!(e.connector.is_none());
        }
    }

    // ─── Generate Connectors Tests ───────────────────────────────

    #[test]
    fn test_generate_connectors_populates_internal_edges() {
        use crate::classic_connector::ClassicKnobConnector;

        let mut grid = PuzzleGrid::new(test_config(3, 4, "conn-pop")).unwrap();
        let connector = ClassicKnobConnector;
        grid.generate_connectors(&connector);

        // All internal edges should have Some connector
        for edge in &grid.h_edges {
            if edge.is_border {
                assert!(
                    edge.connector.is_none(),
                    "border edge should have no connector"
                );
            } else {
                assert!(
                    edge.connector.is_some(),
                    "internal h_edge should have connector after generate_connectors()"
                );
                let curves = edge.connector.as_ref().unwrap();
                assert!(
                    !curves.is_empty(),
                    "internal edge connector should have curves"
                );
            }
        }
        for edge in &grid.v_edges {
            if edge.is_border {
                assert!(
                    edge.connector.is_none(),
                    "border edge should have no connector"
                );
            } else {
                assert!(
                    edge.connector.is_some(),
                    "internal v_edge should have connector after generate_connectors()"
                );
                let curves = edge.connector.as_ref().unwrap();
                assert!(
                    !curves.is_empty(),
                    "internal edge connector should have curves"
                );
            }
        }
    }

    #[test]
    fn test_generate_connectors_deterministic() {
        use crate::classic_connector::ClassicKnobConnector;

        let connector = ClassicKnobConnector;

        let mut grid1 = PuzzleGrid::new(test_config(3, 4, "determ")).unwrap();
        grid1.generate_connectors(&connector);

        let mut grid2 = PuzzleGrid::new(test_config(3, 4, "determ")).unwrap();
        grid2.generate_connectors(&connector);

        // Same seed → same connector curves
        for (e1, e2) in grid1.h_edges.iter().zip(grid2.h_edges.iter()) {
            match (&e1.connector, &e2.connector) {
                (None, None) => {}
                (Some(c1), Some(c2)) => {
                    assert_eq!(c1.len(), c2.len(), "curve count should match");
                    for (a, b) in c1.iter().zip(c2.iter()) {
                        assert!(
                            (a.p0.x - b.p0.x).abs() < 1e-10
                                && (a.p0.y - b.p0.y).abs() < 1e-10
                                && (a.p1.x - b.p1.x).abs() < 1e-10
                                && (a.p1.y - b.p1.y).abs() < 1e-10
                                && (a.p2.x - b.p2.x).abs() < 1e-10
                                && (a.p2.y - b.p2.y).abs() < 1e-10
                                && (a.p3.x - b.p3.x).abs() < 1e-10
                                && (a.p3.y - b.p3.y).abs() < 1e-10,
                            "curves should match for same seed"
                        );
                    }
                }
                _ => panic!("connector presence should match for same seed"),
            }
        }
    }

    #[test]
    fn test_generate_connectors_2x2_minimum() {
        use crate::classic_connector::ClassicKnobConnector;

        // 2x2 is the minimum valid grid; it has no internal h_edges in first/last row
        // but does have 1 internal h_edge row and 1 internal v_edge column
        let mut grid = PuzzleGrid::new(test_config(2, 2, "2x2-conn")).unwrap();
        let connector = ClassicKnobConnector;
        grid.generate_connectors(&connector);

        // h_edges: 3 rows * 2 cols = 6 total. Row 0 and row 2 are border.
        // Row 1 (internal): 2 edges should have connectors
        let internal_h = grid.h_edges.iter().filter(|e| !e.is_border).count();
        let with_conn_h = grid
            .h_edges
            .iter()
            .filter(|e| e.connector.is_some())
            .count();
        assert_eq!(internal_h, 2, "2x2 grid has 2 internal h_edges");
        assert_eq!(
            with_conn_h, 2,
            "all internal h_edges should have connectors"
        );

        // v_edges: 2 rows * 3 cols = 6 total. Col 0 and col 2 are border.
        // Col 1 (internal): 2 edges should have connectors
        let internal_v = grid.v_edges.iter().filter(|e| !e.is_border).count();
        let with_conn_v = grid
            .v_edges
            .iter()
            .filter(|e| e.connector.is_some())
            .count();
        assert_eq!(internal_v, 2, "2x2 grid has 2 internal v_edges");
        assert_eq!(
            with_conn_v, 2,
            "all internal v_edges should have connectors"
        );
    }
}
