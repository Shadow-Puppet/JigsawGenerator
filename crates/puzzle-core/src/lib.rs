pub mod binary_export;
pub mod boundary;
pub mod classic_connector;
pub mod config;
pub mod connector;
pub mod edge;
pub mod grid;
pub mod piece;
pub mod seed;
pub mod masking;
pub mod shapes;
pub mod svg_export;

pub use binary_export::*;
pub use boundary::*;
pub use classic_connector::*;
pub use config::*;
pub use connector::*;
pub use edge::*;
pub use grid::*;
pub use piece::*;
pub use seed::*;
pub use masking::*;
pub use shapes::*;
pub use svg_export::*;

use serde::{Deserialize, Serialize};

/// Configuration for a puzzle grid.
#[derive(Debug, Serialize, Deserialize)]
pub struct GridConfig {
    pub rows: u32,
    pub cols: u32,
}

/// Breakdown of piece types in a puzzle grid.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PieceBreakdown {
    pub total: u32,
    pub corners: u32,
    pub edges: u32,
    pub interior: u32,
}

/// Compute the piece breakdown for a given grid configuration.
///
/// Validates that rows and cols are > 0, then computes:
/// - Total pieces = rows * cols
/// - Corner pieces (always 4, unless grid is 1-dimensional or 1x1)
/// - Edge pieces (on boundary but not corners)
/// - Interior pieces (not on boundary)
///
/// Edge cases:
/// - 1x1: 1 corner, 0 edges, 0 interior
/// - 1xN / Nx1: 2 corners, N-2 edges, 0 interior
/// - 2x2: 4 corners, 0 edges, 0 interior
pub fn compute_piece_breakdown(config: &GridConfig) -> Result<PieceBreakdown, String> {
    if config.rows == 0 {
        return Err("rows must be greater than 0".to_string());
    }
    if config.cols == 0 {
        return Err("cols must be greater than 0".to_string());
    }

    let rows = config.rows;
    let cols = config.cols;
    let total = rows * cols;

    let (corners, edges, interior) = match (rows, cols) {
        (1, 1) => (1, 0, 0),
        (1, c) => (2, c - 2, 0),
        (r, 1) => (2, r - 2, 0),
        (r, c) => {
            let corners = 4;
            let edges = 2 * (r - 2) + 2 * (c - 2);
            let interior = (r - 2) * (c - 2);
            (corners, edges, interior)
        }
    };

    Ok(PieceBreakdown {
        total,
        corners,
        edges,
        interior,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_3x4_grid() {
        let config = GridConfig { rows: 3, cols: 4 };
        let result = compute_piece_breakdown(&config).unwrap();
        assert_eq!(
            result,
            PieceBreakdown {
                total: 12,
                corners: 4,
                edges: 6,
                interior: 2,
            }
        );
    }

    #[test]
    fn test_1x1_grid() {
        let config = GridConfig { rows: 1, cols: 1 };
        let result = compute_piece_breakdown(&config).unwrap();
        assert_eq!(
            result,
            PieceBreakdown {
                total: 1,
                corners: 1,
                edges: 0,
                interior: 0,
            }
        );
    }

    #[test]
    fn test_1x5_grid() {
        let config = GridConfig { rows: 1, cols: 5 };
        let result = compute_piece_breakdown(&config).unwrap();
        assert_eq!(
            result,
            PieceBreakdown {
                total: 5,
                corners: 2,
                edges: 3,
                interior: 0,
            }
        );
    }

    #[test]
    fn test_2x2_grid() {
        let config = GridConfig { rows: 2, cols: 2 };
        let result = compute_piece_breakdown(&config).unwrap();
        assert_eq!(
            result,
            PieceBreakdown {
                total: 4,
                corners: 4,
                edges: 0,
                interior: 0,
            }
        );
    }

    #[test]
    fn test_invalid_zero_rows() {
        let config = GridConfig { rows: 0, cols: 4 };
        let result = compute_piece_breakdown(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "rows must be greater than 0");
    }

    #[test]
    fn test_invalid_zero_cols() {
        let config = GridConfig { rows: 3, cols: 0 };
        let result = compute_piece_breakdown(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "cols must be greater than 0");
    }

    #[test]
    fn test_5x1_grid() {
        let config = GridConfig { rows: 5, cols: 1 };
        let result = compute_piece_breakdown(&config).unwrap();
        assert_eq!(
            result,
            PieceBreakdown {
                total: 5,
                corners: 2,
                edges: 3,
                interior: 0,
            }
        );
    }
}
