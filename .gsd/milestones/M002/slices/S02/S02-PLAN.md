# S02: Boundary-Aware Grid Generation

**Goal:** Generate puzzle grids clipped to non-rectangular boundary shapes, with cells outside removed, internal edges filtered, and the shape contour used as the border — all exported to SVG and exposed through WASM.
**Demo:** A WASM endpoint generates a heart-shaped puzzle with valid connectors; SVG export shows the heart contour as the border with only interior edges; same seed + shape produces identical output.

## Must-Haves

- `BoundaryPuzzle` wraps `PuzzleGrid` and classifies cells as inside/outside a closed BezPath boundary using kurbo `Shape::winding`
- Grid edges between two outside cells are excluded; edges between two inside cells are kept with their connectors
- Boundary-adjacent edges (one inside, one outside cell) are excluded from internal edge output — the shape contour replaces them
- Shape contour used as the border path instead of the rectangular border in SVG and binary export
- `mask_difference` support: given a whimsy shape, remove all grid edges inside it (R004 contract)
- Determinism preserved: same seed + same boundary config = identical output (R013)
- WASM endpoints accept optional border shape parameter and return boundary-aware grid data

## Proof Level

- This slice proves: contract + integration (Rust unit tests prove boundary logic; WASM compilation proves end-to-end wiring)
- Real runtime required: no (WASM compilation check is sufficient; browser rendering is S03)
- Human/UAT required: no

## Verification

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all existing 114 tests pass + new boundary tests pass
- `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` — boundary-specific tests: cell classification, edge filtering, determinism, whimsy hole, SVG output
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — WASM endpoint tests for border shape parameter
- `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — confirms WASM compilation
- `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary_no_cells` — failure-path test: tiny boundary produces zero included cells and zero included edges, verifying graceful empty-grid handling

## Observability / Diagnostics

- Runtime signals: none (pure computation, no async/IO)
- Inspection surfaces: `cargo test -- boundary` runs all boundary-specific tests; `BoundaryPuzzle::included_cells()` returns the set of included (row, col) pairs for debugging
- Failure visibility: test assertions show cell classification, edge counts, SVG path content
- Redaction constraints: none

## Integration Closure

- Upstream surfaces consumed: `crates/puzzle-core/src/shapes.rs` (heart_path, star_path), `crates/puzzle-core/src/masking.rs` (mask_intersection, mask_difference), `crates/puzzle-core/src/grid.rs` (PuzzleGrid), `crates/puzzle-core/src/svg_export.rs` (edge_transform, build_border_path), `crates/puzzle-core/src/binary_export.rs` (border_to_binary, edges_to_binary)
- New wiring introduced in this slice: `BoundaryPuzzle` struct in `boundary.rs`; extended WASM endpoints accepting border shape; boundary-aware SVG and binary export functions
- What remains before the milestone is truly usable end-to-end: S03 (UI border selection), S04 (whimsy drag-drop), S05 (sub-puzzle), S06 (final polish)

## Tasks

- [x] **T01: Implement BoundaryPuzzle core engine with cell classification and edge filtering** `est:1h`
  - Why: Core geometric engine for R002/R004/R013 — classifies cells as inside/outside a boundary shape, filters edges, provides the `included_cells` and `included_edges` API that SVG export and WASM integration consume
  - Files: `crates/puzzle-core/src/boundary.rs`, `crates/puzzle-core/src/lib.rs`
  - Do: Create `BoundaryPuzzle` struct wrapping `PuzzleGrid` + boundary `BezPath`. Use kurbo `Shape::winding()` to test cell center containment (nonzero = inside). Classify edges: between two inside cells → included (keep connector); between inside and outside → boundary-adjacent (excluded from internal edges, replaced by shape contour); between two outside cells → excluded. Support both mask mode (keep inside boundary) and difference mode (remove inside whimsy shape). Generate full rectangular grid first for RNG determinism, then filter. Include comprehensive unit tests: cell classification for heart/star shapes, edge filtering correctness, determinism, whimsy hole cutting, empty-boundary edge case.
  - Verify: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` passes all new tests
  - Done when: `BoundaryPuzzle::new()` correctly classifies cells and filters edges for heart and star boundaries; determinism test passes; whimsy difference mode works; all 114 existing tests still pass
  - **Skill:** None required — pure Rust data structures and kurbo geometry

- [x] **T02: Add boundary-aware SVG and binary export** `est:45m`
  - Why: Proves the visual output contract — shape contour replaces rectangular border in SVG; binary export sends shape contour commands and only included internal edges; completes the export pipeline for S03/S06 to consume
  - Files: `crates/puzzle-core/src/boundary.rs`, `crates/puzzle-core/src/svg_export.rs`, `crates/puzzle-core/src/binary_export.rs`
  - Do: Add `generate_boundary_svg()` function that uses the boundary shape BezPath as the border path instead of the rectangular border, then appends only included internal edge connectors. Add `boundary_edges_to_binary()` that serializes only included internal edges. Add `boundary_border_to_binary()` that serializes the shape contour path. Add tests: SVG contains shape path data (curves for heart, lines for star), SVG excludes outside edges, SVG is deterministic, binary export stride/count correctness for boundary puzzles.
  - Verify: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` passes all boundary export tests
  - Done when: Heart-shaped puzzle SVG contains cubic bezier border curves (not rectangular), only interior edges appear, binary export counts match included edge count

- [x] **T03: Wire boundary puzzle through WASM endpoints** `est:45m`
  - Why: Connects Rust boundary engine to the browser — S03 needs WASM endpoints that accept a border shape parameter; proves the full pipeline compiles to WASM
  - Files: `crates/puzzle-core/src/config.rs`, `crates/puzzle-wasm/src/lib.rs`
  - Do: Add optional `border_shape: Option<String>` field to `PuzzleConfig` (serde default None for backward compat). In puzzle-wasm, when `border_shape` is Some("heart"|"star"), create the corresponding BezPath scaled to puzzle dimensions, construct `BoundaryPuzzle`, and use boundary-aware export. Existing endpoints continue to work unchanged when `border_shape` is None. Add WASM-level tests: generate with border_shape="heart" returns SVG with curved border, binary export has correct shape contour, backward compat (no border_shape) unchanged. Verify WASM compilation.
  - Verify: `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` passes; `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` succeeds
  - Done when: `generate_edges_binary('{"border_shape":"heart",...}')` returns boundary-aware data; `generate_svg(...)` with border_shape returns heart-bordered SVG; all existing WASM tests pass unchanged; WASM target compiles

## Files Likely Touched

- `crates/puzzle-core/src/boundary.rs` (new)
- `crates/puzzle-core/src/lib.rs`
- `crates/puzzle-core/src/svg_export.rs`
- `crates/puzzle-core/src/binary_export.rs`
- `crates/puzzle-core/src/config.rs`
- `crates/puzzle-wasm/src/lib.rs`
