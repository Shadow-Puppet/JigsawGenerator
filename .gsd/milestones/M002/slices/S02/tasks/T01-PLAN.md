---
estimated_steps: 5
estimated_files: 2
---

# T01: Implement BoundaryPuzzle core engine with cell classification and edge filtering

**Slice:** S02 — Boundary-Aware Grid Generation
**Milestone:** M002

## Description

Create the `BoundaryPuzzle` struct in a new `boundary.rs` module. This is the core geometric engine for generating non-rectangular puzzles. It wraps an existing `PuzzleGrid` (which is generated rectangularly first for RNG determinism) and applies boundary-awareness as a post-processing step.

The key algorithm:
1. Generate a full rectangular `PuzzleGrid` with connectors (preserves RNG sequence = determinism)
2. For each cell `(row, col)`, test whether the cell center falls inside the boundary shape using `kurbo::Shape::winding()` (nonzero winding = inside)
3. Classify edges based on the two cells they border:
   - Both cells inside → **included** (keep connector)
   - Both cells outside → **excluded** (discard entirely)
   - One inside, one outside → **boundary-adjacent** (excluded from internal edges — the shape contour replaces these)
4. Border edges of the rectangular grid are always excluded (the shape contour is the new border)

For whimsy support (R004 — `mask_difference`): the same engine works in "difference mode" — cells inside the whimsy shape are marked as *outside* (removed), creating a hole in the grid.

**Critical constraints from S01 (Forward Intelligence):**
- Shape constructors center paths within `(width, height)`. For boundary mode, create the shape at puzzle dimensions: `heart_path(config.width, config.height)`.
- `Shape::winding()` returns nonzero for points inside a closed path. Use `shape.winding(cell_center) != 0` for containment.
- Boolean op results can contain multiple disjoint subpaths — but for boundary mode we use the shape directly (not a boolean op result), so this is not a concern in T01.

## Steps

1. Create `crates/puzzle-core/src/boundary.rs` with `BoundaryPuzzle` struct containing: the inner `PuzzleGrid`, the boundary `BezPath`, and computed cell inclusion (a `Vec<Vec<bool>>` where `true` = inside)
2. Implement `BoundaryPuzzle::new(grid: PuzzleGrid, boundary: BezPath)` that classifies all cells using winding number on cell centers. Cell center for `(row, col)` is `((col as f64 + 0.5) * cell_w, (row as f64 + 0.5) * cell_h)`.
3. Add `included_cells(&self) -> Vec<(usize, usize)>` returning `(row, col)` pairs of inside cells, and edge accessor methods: `included_h_edges()` and `included_v_edges()` that return iterators/vecs of edges between two included cells (with their grid indices for lookup).
4. Add `BoundaryPuzzle::new_with_hole(grid: PuzzleGrid, boundary: BezPath, hole: BezPath)` that marks cells inside the hole as excluded (for R004 whimsy difference mode). A cell is included if it's inside the boundary AND outside the hole.
5. Write comprehensive unit tests in `#[cfg(test)] mod tests` within `boundary.rs`:
   - `test_all_cells_inside_large_boundary` — boundary larger than grid → all cells included
   - `test_heart_boundary_excludes_corner_cells` — heart shape on a 6×8 grid excludes some corner cells
   - `test_star_boundary_excludes_cells` — star shape excludes cells in concavities
   - `test_included_edges_between_inside_cells` — only edges between two inside cells are included
   - `test_boundary_edge_count_less_than_full` — boundary puzzle has fewer internal edges than full grid
   - `test_determinism` — same seed + same boundary = identical cell inclusion and edge lists
   - `test_hole_removes_center_cells` — `new_with_hole` with a small centered shape removes center cells
   - `test_no_cells_inside_tiny_boundary` — very small boundary → no cells included
   - Register `pub mod boundary` and `pub use boundary::*` in `lib.rs`

## Must-Haves

- [ ] `BoundaryPuzzle` classifies cells using kurbo `Shape::winding()` on cell centers
- [ ] Edges between two included cells are marked as included; all other edges excluded
- [ ] Full rectangular grid generated first for RNG determinism — boundary filtering is pure post-processing
- [ ] `new_with_hole()` supports whimsy difference mode (R004)
- [ ] All 114 existing tests continue to pass
- [ ] At least 7 new boundary-specific unit tests pass

## Verification

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all tests pass (114 existing + new boundary tests)
- `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` — all boundary-specific tests pass
- `cargo check --manifest-path crates/puzzle-core/Cargo.toml` — no compiler warnings/errors

## Inputs

- `crates/puzzle-core/src/grid.rs` — PuzzleGrid struct with h_edges, v_edges, cell accessors
- `crates/puzzle-core/src/shapes.rs` — heart_path, star_path constructors (S01 output)
- `crates/puzzle-core/src/masking.rs` — mask_intersection, mask_difference (S01 output, used for reference only in T01)
- `crates/puzzle-core/src/edge.rs` — Edge struct definition
- `crates/puzzle-core/src/config.rs` — PuzzleConfig, TabConfig
- `crates/puzzle-core/src/lib.rs` — module registration

## Expected Output

- `crates/puzzle-core/src/boundary.rs` — new file with BoundaryPuzzle struct, cell classification, edge filtering, and ≥7 unit tests
- `crates/puzzle-core/src/lib.rs` — updated with `pub mod boundary` and `pub use boundary::*`
