# 003 — Boundary-aware grid generation

**What:** Generate a puzzle grid clipped to an arbitrary closed `BezPath` boundary, with correct cell classification, edge filtering, SVG export, binary export, and whimsy-hole support.
**Why:** This is the shared engine for both custom borders (mask mode) and whimsy placement (reverse-mask mode). Landing it centrally means S03/S04/S05 just pick the right configuration.

## What shipped

- `crates/puzzle-core/src/boundary.rs` — `BoundaryPuzzle` struct:
  - `new(grid, boundary)` — cell classification via `kurbo::Shape::winding()` on cell centres; nonzero = inside. Stores `Vec<Vec<bool>>` for O(1) lookup.
  - `new_with_hole(grid, boundary, hole)` — adds a second containment test; cell must be inside boundary **and** outside hole. This is the R004 reverse-mask primitive for whimsy.
  - `included_h_edges()` / `included_v_edges()` — return indices (not copies) of edges where both adjacent cells are inside.
  - `generate_boundary_svg()` — uses boundary BezPath as the border subpath, then appends only included-interior connectors.
  - `boundary_edges_to_binary()`, `boundary_border_to_binary()` — binary exports for Canvas. Border uses CMD-prefixed encoding so shape contours ride in the same `Float64Array`.
- `crates/puzzle-core/src/config.rs` — `PuzzleConfig.border_shape: Option<String>` with `#[serde(default)]` so old configs still deserialize.
- `crates/puzzle-wasm/src/lib.rs` — all three endpoints (`generate_svg`, `generate_edges_binary`, `generate_grid`) extended with `border_shape` handling. New `resolve_border_shape(name, width, height)` helper centralises the name → `BezPath` mapping.
- `crates/puzzle-core/src/svg_export.rs` — `edge_transform` made `pub`, `build_svg_document` made `pub(crate)` so boundary export can reuse them.
- 132 puzzle-core + 22 puzzle-wasm tests at slice close, +27 new. Zero regressions.

## Patterns established

- **Boundary filtering as post-processing.** Always build the full rectangular grid first (preserves the canonical RNG consumption order), then apply containment filtering on top. Seed determinism survives any border/whimsy change. Sub-puzzles in S05 will reuse this pattern inside the whimsy contour.
- **Winding number, not boolean ops, for point containment.** `kurbo::Shape::winding()` on a cell centre is far simpler and faster than a `linesweeper` intersection. Boolean ops stay reserved for path-vs-path work (masking, difference).
- **Edge indices, not copies.** Downstream consumers read connectors straight from the grid's edge arrays — no serialize/deserialize round-trip.
- **Centralised shape-name resolver.** `resolve_border_shape()` is the single place to add a new preset shape; everything else branches on `Option<String>` in the config.
