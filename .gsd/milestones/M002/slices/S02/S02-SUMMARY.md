# S02: Boundary-Aware Grid Generation — Summary

**Status:** Complete  
**Completed:** 2026-03-24  
**Tasks:** 3/3 (T01 ✅, T02 ✅, T03 ✅)

## What This Slice Delivered

A complete boundary-aware puzzle generation pipeline: given an arbitrary closed BezPath shape, the system generates a puzzle grid clipped to that boundary with correct cell classification, edge filtering, SVG export (shape contour as border with only interior edges), binary export, and WASM endpoints — all deterministic and backward compatible.

**End-to-end capability:** `generate_svg('{"border_shape":"heart", ...}')` returns an SVG with a cubic bezier heart contour as the border, only interior edges with connectors, and no rectangular border remnants. `generate_edges_binary(...)` with `border_shape` returns boundary-aware binary data for Canvas rendering. `generate_grid(...)` reports filtered piece counts.

## Key Artifacts

| File | What it provides |
|------|-----------------|
| `crates/puzzle-core/src/boundary.rs` | `BoundaryPuzzle` struct — cell classification via kurbo winding number, `included_h_edges()`/`included_v_edges()` returning indices, `new_with_hole()` for whimsy difference mode, `generate_boundary_svg()`, `boundary_edges_to_binary()`, `boundary_border_to_binary()`, 18 unit tests |
| `crates/puzzle-core/src/lib.rs` | `pub mod boundary` + `pub use boundary::*` |
| `crates/puzzle-core/src/config.rs` | `PuzzleConfig.border_shape: Option<String>` (serde default None) |
| `crates/puzzle-core/src/svg_export.rs` | `edge_transform` made pub; `build_svg_document` made pub(crate) |
| `crates/puzzle-core/src/binary_export.rs` | `CMD_*` constants made pub(crate) |
| `crates/puzzle-wasm/src/lib.rs` | All 3 endpoints extended with border_shape support; `resolve_border_shape()` helper; 9 new tests |
| `crates/puzzle-wasm/Cargo.toml` | Added kurbo direct dependency |

## How It Works

1. **Cell classification:** `BoundaryPuzzle::new()` takes a pre-generated `PuzzleGrid` + boundary `BezPath`. For each cell, computes center point and tests containment via `kurbo::Shape::winding()` (nonzero = inside). Stores as `Vec<Vec<bool>>` for O(1) lookup.

2. **Edge filtering:** `included_h_edges()`/`included_v_edges()` return indices for edges where both adjacent cells are inside the boundary. Border-row/column edges are always excluded (shape contour replaces them).

3. **Whimsy hole:** `new_with_hole()` applies a second containment test — cell must be inside boundary AND outside hole shape. This is the R004 "reverse mask" for whimsy placement.

4. **SVG export:** `generate_boundary_svg()` uses the boundary BezPath as the border subpath, then appends connector paths for only included internal edges.

5. **Binary export:** `boundary_edges_to_binary()` serializes included edges in EDGE_STRIDE format; `boundary_border_to_binary()` serializes shape contour with CMD_* prefixed format.

6. **WASM wiring:** `resolve_border_shape(name, width, height)` maps "heart"/"star" to scaled BezPaths. All 3 endpoints branch on `border_shape`: when Some, use BoundaryPuzzle; when None, use existing rectangular path.

7. **Determinism:** Full rectangular grid is always generated first (preserving RNG sequence), then boundary filtering is applied as post-processing. Same seed + same shape = identical output (D024).

## Patterns Established

- **Boundary filtering as post-processing** — generate the full grid for RNG determinism, then classify and filter. This pattern will be reused in S05 for sub-puzzle generation inside whimsy contours.
- **Winding-number containment** — use `kurbo::Shape::winding()` for point-in-path tests, not boolean ops. Reserve linesweeper for path-vs-path operations.
- **Edge indices, not copies** — `included_h_edges()`/`included_v_edges()` return indices into the grid's edge arrays so downstream code can access connectors directly.
- **resolve_border_shape() centralization** — single function maps shape name strings to BezPaths. Adding a new shape requires one match arm.
- **border_shape as optional config field** — `Option<String>` with serde defaults ensures full backward compatibility.

## Verification

All 5 slice-level checks pass:

| # | Command | Result | Count |
|---|---------|--------|-------|
| 1 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml` | ✅ | 132 tests (114 existing + 18 new) |
| 2 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` | ✅ | 19 boundary tests |
| 3 | `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` | ✅ | 22 tests (13 existing + 9 new) |
| 4 | `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` | ✅ | compiles |
| 5 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary_no_cells` | ✅ | 1 test (empty boundary edge case) |

**Total new tests:** 27 (18 puzzle-core + 9 puzzle-wasm)  
**Zero regressions** on existing 127 tests (114 puzzle-core + 13 puzzle-wasm)

## Requirements Validated

- **R002** (boundary-aware grid generation) — validated: cell classification, edge filtering, SVG and binary export all proven with heart/star shapes
- **R004** (whimsy hole in grid) — validated: `new_with_hole()` removes cells inside whimsy shape, no connectors at boundary
- **R013** (determinism) — validated: 4 determinism tests prove same seed + same boundary = identical output across SVG, binary, and WASM

## What Downstream Slices Should Know

- **S03 (Custom Border UI):** WASM endpoints are ready. Pass `border_shape: "heart"` or `"star"` in the config JSON. `generate_svg()` returns complete boundary SVG. `generate_edges_binary()` returns boundary-aware binary data for Canvas. `generate_grid()` returns filtered piece counts.
- **S04 (Whimsy Drag-Drop):** Use `BoundaryPuzzle::new_with_hole()` for the reverse-mask. The WASM layer needs a new parameter for whimsy shape/position/size — not yet wired (S04 task). The `resolve_border_shape()` pattern can be extended for whimsy shapes.
- **S05 (Sub-Puzzle):** `BoundaryPuzzle::new()` with a whimsy contour as boundary generates the sub-puzzle grid inside the whimsy shape. The same engine, different boundary.
- **S06 (Export):** `generate_boundary_svg()` and binary export functions are complete. SVG structure is single `<path>` with shape contour as first subpath.

## Decisions Made

- D023: BoundaryPuzzle cell data as Vec<Vec<bool>> with edge indices (not copies) — O(1) lookup, zero-copy downstream access
- D024: Boundary filtering as post-processing on full rectangular grid — preserves RNG determinism
- D025: resolve_border_shape() as centralized shape name → BezPath mapping in WASM layer

## Knowledge Captured

- K006: js_sys APIs panic on native test targets — test WASM binary endpoints indirectly
- K007: Use kurbo winding number for cell containment, not linesweeper boolean ops
