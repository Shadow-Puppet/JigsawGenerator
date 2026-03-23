---
id: T01
parent: S02
milestone: M002
provides:
  - BoundaryPuzzle struct with cell classification via kurbo winding number
  - Edge filtering: included_h_edges / included_v_edges returning indices for edges between two inside cells
  - new_with_hole() for whimsy difference mode (R004)
  - included_cells() diagnostic surface
key_files:
  - crates/puzzle-core/src/boundary.rs
  - crates/puzzle-core/src/lib.rs
key_decisions:
  - Cell inclusion stored as Vec<Vec<bool>> for O(1) lookup during edge filtering
  - Edge accessors return indices into grid.h_edges / grid.v_edges (not copies) so downstream can access connectors
  - Border h/v edges always excluded regardless of cell inclusion — shape contour replaces them
patterns_established:
  - Boundary filtering as pure post-processing on a full rectangular PuzzleGrid (preserves RNG determinism)
  - Winding number containment test via kurbo Shape::winding() on cell centers
observability_surfaces:
  - BoundaryPuzzle::included_cells() returns (row, col) pairs for debugging
  - cargo test -- boundary runs all boundary-specific tests
duration: 15m
verification_result: passed
completed_at: 2026-03-23
blocker_discovered: false
---

# T01: Implement BoundaryPuzzle core engine with cell classification and edge filtering

**Added BoundaryPuzzle engine with winding-number cell classification, edge filtering, and whimsy hole support — 9 new tests pass alongside all 114 existing tests.**

## What Happened

Created `crates/puzzle-core/src/boundary.rs` with the `BoundaryPuzzle` struct that wraps a full rectangular `PuzzleGrid` and applies boundary-aware post-processing. The core algorithm:

1. Takes a pre-generated `PuzzleGrid` (preserving the RNG sequence for determinism)
2. For each cell `(row, col)`, computes the center point `((col+0.5)*cell_w, (row+0.5)*cell_h)` and tests containment using `kurbo::Shape::winding()` (nonzero = inside)
3. Stores inclusion as `Vec<Vec<bool>>` for O(1) edge filtering
4. `included_h_edges()` and `included_v_edges()` return indices into the grid's edge arrays for edges where both adjacent cells are inside the boundary
5. Border edges (row 0/rows for h, col 0/cols for v) are always excluded — the shape contour replaces them

Also implemented `new_with_hole()` for R004 whimsy difference mode: a cell must be inside the boundary AND outside the hole to be included.

Registered the module in `lib.rs` as `pub mod boundary` + `pub use boundary::*`.

## Verification

All three verification commands from the task plan pass:

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — 123 tests pass (114 existing + 9 new)
- `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` — 10 boundary-related tests pass (9 new + 1 pre-existing config test)
- `cargo check --manifest-path crates/puzzle-core/Cargo.toml` — no warnings or errors

Slice-level verifications applicable at T01 stage:
- `cargo test -- boundary` ✅ (cell classification, edge filtering, determinism, whimsy hole)
- `cargo test -- boundary_no_cells` ✅ (empty-grid graceful handling)
- Full test suite 123/123 ✅ (no regressions)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml` | 0 | ✅ pass | 0.02s |
| 2 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` | 0 | ✅ pass | 0.01s |
| 3 | `cargo check --manifest-path crates/puzzle-core/Cargo.toml` | 0 | ✅ pass | 1.34s |
| 4 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary_no_cells` | 0 | ✅ pass | 0.00s |

## Diagnostics

- `BoundaryPuzzle::included_cells()` returns the set of (row, col) pairs for inspecting which cells are inside the boundary
- `cargo test -- boundary` runs all boundary-specific tests
- Test assertions display cell counts, edge counts, and index bounds for debugging

## Deviations

- Added `test_boundary_edge_indices_valid` (9th test) beyond the 8 specified in the task plan — validates that returned edge indices are within bounds of the grid's edge arrays. This provides an extra safety net for T02's export code.

## Known Issues

None.

## Files Created/Modified

- `crates/puzzle-core/src/boundary.rs` — new: BoundaryPuzzle struct with cell classification, edge filtering, hole support, and 9 unit tests
- `crates/puzzle-core/src/lib.rs` — modified: added `pub mod boundary` and `pub use boundary::*`
