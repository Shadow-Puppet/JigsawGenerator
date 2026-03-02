use serde::{Deserialize, Serialize};

/// Edge indices for a single puzzle piece, referencing shared edges
/// in the `PuzzleGrid`'s `h_edges` and `v_edges` arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PieceEdges {
    /// Index into h_edges for the top edge.
    pub top: usize,
    /// Index into h_edges for the bottom edge.
    pub bottom: usize,
    /// Index into v_edges for the left edge.
    pub left: usize,
    /// Index into v_edges for the right edge.
    pub right: usize,
}

/// Classification of a puzzle piece by its position in the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PieceType {
    /// Corner piece: 2 border edges (exactly 4 per grid).
    Corner,
    /// Edge piece: 1 border edge (on boundary but not corner).
    Edge,
    /// Interior piece: 0 border edges.
    Interior,
}

/// A single puzzle piece with its position, type, and edge references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Piece {
    /// Row position in the grid (0-indexed).
    pub row: usize,
    /// Column position in the grid (0-indexed).
    pub col: usize,
    /// Classification based on border edge count.
    pub piece_type: PieceType,
    /// Indices into the grid's shared edge arrays.
    pub edges: PieceEdges,
}
