# S02: Boundary-Aware Grid Generation — UAT Script

**Proof level:** Contract + Integration (Rust unit tests + WASM compilation)  
**No browser/UI required** — all verification is via `cargo test` and `cargo check`.

---

## Preconditions

- Working directory: project root (or worktree root)
- Rust toolchain with `wasm32-unknown-unknown` target installed
- All S01 artifacts present (`shapes.rs`, `masking.rs`)

---

## Test Case 1: Cell Classification — Heart Shape

**What it tests:** Cells outside the heart boundary are excluded; cells inside are included.

**Steps:**
1. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_heart_excludes_corner_cells`
2. Expected: Test passes. A heart shape on a grid excludes corner cells that fall outside the curved boundary.
3. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_all_cells_inside_large_boundary`
4. Expected: Test passes. A boundary larger than the grid includes all cells.

**Pass criteria:** Both tests exit 0.

---

## Test Case 2: Cell Classification — Star Shape

**What it tests:** Star shape excludes cells in the concave indentations between points.

**Steps:**
1. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_star_excludes_cells`
2. Expected: Test passes. Star boundary excludes some cells, and the excluded count is less than the full grid.

**Pass criteria:** Exit 0, fewer included cells than total grid cells.

---

## Test Case 3: Edge Filtering

**What it tests:** Only edges between two inside cells are included; boundary-adjacent and outside edges are excluded.

**Steps:**
1. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_included_edges_between_inside_cells`
2. Expected: Included edge count > 0 and < total internal edge count.
3. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_edge_count_less_than_full`
4. Expected: Heart boundary produces fewer included edges than the full rectangular grid.
5. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_edge_indices_valid`
6. Expected: All returned indices are within bounds of `grid.h_edges` / `grid.v_edges`.

**Pass criteria:** All three tests exit 0.

---

## Test Case 4: Whimsy Hole (Difference Mode)

**What it tests:** R004 — a whimsy shape placed inside the boundary removes cells, creating a "hole" in the grid.

**Steps:**
1. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_hole_removes_center_cells`
2. Expected: Test passes. With a hole shape, fewer cells are included than without the hole. Center cells within the hole shape are excluded.

**Pass criteria:** Exit 0, included cell count with hole < included cell count without hole.

---

## Test Case 5: Determinism (R013)

**What it tests:** Same seed + same boundary shape = identical output across all surfaces.

**Steps:**
1. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_determinism`
2. Expected: Two BoundaryPuzzles with same seed/config produce identical cell inclusion and edge indices.
3. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_svg_deterministic`
4. Expected: Two SVG generations with same seed produce byte-identical output.
5. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_binary_edges_deterministic`
6. Expected: Two binary exports with same seed produce identical byte sequences.
7. Run: `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- test_generate_svg_heart_border_deterministic`
8. Expected: Two WASM-level SVG generations with same seed produce identical output.

**Pass criteria:** All four tests exit 0.

---

## Test Case 6: SVG Export — Heart Boundary

**What it tests:** SVG output uses heart shape contour (cubic bezier curves) as border, not a rectangle.

**Steps:**
1. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_svg_contains_cubic_curves`
2. Expected: Heart-shaped SVG contains `C` commands (cubic bezier curves) in the path data.
3. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_svg_excludes_outside_edges`
4. Expected: The SVG `M` command count equals included_edge_count + 1 (one M for the border, one per internal edge).
5. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_svg_has_valid_structure`
6. Expected: SVG has correct `<svg>` wrapper, `viewBox`, `xmlns`, and `<path>` element.

**Pass criteria:** All three tests exit 0.

---

## Test Case 7: Binary Export — Boundary Data

**What it tests:** Binary edge and border export formats are correct for boundary puzzles.

**Steps:**
1. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_binary_edge_count`
2. Expected: Binary edge data length / EDGE_STRIDE == included_edge_count.
3. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_binary_border_starts_with_moveto`
4. Expected: First float in border binary data = CMD_MOVE_TO (0.0).
5. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_binary_border_has_curves`
6. Expected: Heart border binary contains CMD_CURVE_TO (2.0) values.
7. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_binary_border_has_close`
8. Expected: Border binary ends with CMD_CLOSE (3.0).

**Pass criteria:** All four tests exit 0.

---

## Test Case 8: Empty Boundary Edge Case

**What it tests:** A boundary too small to contain any cell centers produces zero included cells and zero edges gracefully — no panic, no invalid output.

**Steps:**
1. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_boundary_no_cells_inside_tiny_boundary`
2. Expected: Test passes. Zero included cells, zero included edges, no crash.

**Pass criteria:** Exit 0.

---

## Test Case 9: WASM Endpoint Integration

**What it tests:** WASM endpoints accept border_shape parameter and return boundary-aware output.

**Steps:**
1. Run: `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- test_generate_svg_with_heart_border`
2. Expected: SVG output from WASM endpoint contains cubic bezier curves.
3. Run: `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- test_generate_svg_with_star_border`
4. Expected: SVG output from WASM endpoint contains star shape data.
5. Run: `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- test_generate_svg_no_border_shape_unchanged`
6. Expected: Without border_shape, SVG output is identical to pre-S02 behavior.
7. Run: `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- test_border_shape_invalid_returns_error`
8. Expected: Unknown border_shape value returns error JSON, not panic.
9. Run: `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- test_generate_grid_with_border_shape_fewer_pieces`
10. Expected: Grid with border_shape reports fewer pieces than full rectangular grid.

**Pass criteria:** All five tests exit 0.

---

## Test Case 10: WASM Compilation

**What it tests:** The entire dependency tree (including boundary.rs, linesweeper, kurbo) compiles to wasm32-unknown-unknown.

**Steps:**
1. Run: `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown`
2. Expected: Exit 0, no errors, no warnings.

**Pass criteria:** Exit 0.

---

## Test Case 11: No Regressions

**What it tests:** All pre-existing tests still pass after S02 changes.

**Steps:**
1. Run: `cargo test --manifest-path crates/puzzle-core/Cargo.toml`
2. Expected: 132 tests pass (114 pre-S02 + 18 new boundary tests).
3. Run: `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml`
4. Expected: 22 tests pass (13 pre-S02 + 9 new boundary WASM tests).

**Pass criteria:** Both exit 0 with zero failures.

---

## Aggregate Verification (Run All)

```bash
cargo test --manifest-path crates/puzzle-core/Cargo.toml && \
cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary && \
cargo test --manifest-path crates/puzzle-wasm/Cargo.toml && \
cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown && \
cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary_no_cells
```

**Expected:** All 5 commands exit 0. Total: 154 tests pass (132 puzzle-core + 22 puzzle-wasm).
