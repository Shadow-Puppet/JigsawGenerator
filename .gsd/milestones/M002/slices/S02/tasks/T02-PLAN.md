---
estimated_steps: 5
estimated_files: 3
---

# T02: Add boundary-aware SVG and binary export

**Slice:** S02 — Boundary-Aware Grid Generation
**Milestone:** M002

## Description

Extend the export pipeline so `BoundaryPuzzle` can produce both SVG and binary output with the shape contour as the border and only included internal edges. This is the visual proof that boundary-aware grid generation works correctly.

**SVG export:** Replace the rectangular border subpath with the boundary shape's BezPath. Then iterate only the included internal edges (from `BoundaryPuzzle`) and append their connector curves. The shape contour for a heart will contain cubic bezier curves (`C` commands), proving non-rectangular geometry in the output.

**Binary export:** Same principle — `boundary_border_to_binary()` serializes the shape contour path using the existing command-prefixed format (CMD_MOVE_TO, CMD_LINE_TO, CMD_CURVE_TO, CMD_CLOSE). `boundary_edges_to_binary()` serializes only included internal edges using the existing EDGE_STRIDE format.

Both functions live in `boundary.rs` as methods on `BoundaryPuzzle`, consuming the existing `edge_transform()` helper from `svg_export.rs` (already `pub(crate)`) and the command constants from `binary_export.rs`.

**Key constraint from S01 Forward Intelligence:** Boolean op results can have multiple disjoint subpaths. While we use the shape directly (not a boolean op result) for border mode, the SVG/binary serialization must handle multi-subpath BezPaths correctly by iterating all PathEl variants.

## Steps

1. Add `generate_boundary_svg(&self) -> String` method to `BoundaryPuzzle` in `boundary.rs`. It builds the complete SVG: uses `self.boundary` BezPath as the border subpath (iterating all PathEl elements), then appends connector curves for only included internal edges (using `edge_transform` from `svg_export`). Wraps in SVG document with mm dimensions and viewBox.
2. Add `boundary_edges_to_binary(&self) -> Vec<f64>` method that serializes only included internal edge connectors in the same EDGE_STRIDE format as `edges_to_binary()`. Make the command constants in `binary_export.rs` `pub(crate)` so `boundary.rs` can use them.
3. Add `boundary_border_to_binary(&self) -> Vec<f64>` method that serializes the boundary shape path using the command-prefixed format (same as `border_to_binary()`).
4. Make `edge_transform` in `svg_export.rs` public (change `pub(crate)` to `pub`) so it's accessible from `boundary.rs`. Also make `build_svg_document` or its logic accessible (either make it public or inline the format string).
5. Write tests in `boundary.rs`:
   - `test_boundary_svg_contains_cubic_curves` — heart boundary SVG has `C` commands in border section
   - `test_boundary_svg_excludes_outside_edges` — SVG M-command count matches included edge count + 1 (border)
   - `test_boundary_svg_deterministic` — same seed + boundary = identical SVG
   - `test_boundary_binary_edge_count` — binary edge count matches included_h_edges + included_v_edges count
   - `test_boundary_binary_border_starts_with_moveto` — border binary starts with CMD_MOVE_TO
   - `test_boundary_binary_border_has_curves` — heart border binary contains CMD_CURVE_TO

## Must-Haves

- [ ] `generate_boundary_svg()` uses shape contour as border, not rectangular path
- [ ] Only included internal edges appear in SVG and binary output
- [ ] Binary export format compatible with existing command constants (CMD_MOVE_TO etc.) and EDGE_STRIDE
- [ ] SVG output is deterministic (same seed + boundary = identical)
- [ ] At least 5 new export-specific tests pass

## Verification

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` — all boundary tests pass (T01 + T02)
- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all 114+ tests pass
- `cargo check --manifest-path crates/puzzle-core/Cargo.toml` — no warnings

## Inputs

- `crates/puzzle-core/src/boundary.rs` — BoundaryPuzzle from T01 with cell classification and edge filtering
- `crates/puzzle-core/src/svg_export.rs` — edge_transform helper, build_svg_document pattern
- `crates/puzzle-core/src/binary_export.rs` — EDGE_STRIDE, CMD_* constants, serialization pattern
- `crates/puzzle-core/src/grid.rs` — PuzzleGrid with h_edge/v_edge accessors
- `crates/puzzle-core/src/edge.rs` — Edge struct with connector field

## Expected Output

- `crates/puzzle-core/src/boundary.rs` — updated with generate_boundary_svg(), boundary_edges_to_binary(), boundary_border_to_binary() methods and ≥5 new tests
- `crates/puzzle-core/src/svg_export.rs` — edge_transform visibility changed to pub; build_svg_document or equivalent made accessible
- `crates/puzzle-core/src/binary_export.rs` — CMD_* constants visibility changed to pub(crate)

## Observability Impact

- **Inspection surfaces:** `cargo test -- boundary_svg` and `cargo test -- boundary_binary` run all export-specific boundary tests. SVG output can be inspected for `C` commands (proving curved boundary), M-command count (proving correct edge filtering), and deterministic output.
- **Failure visibility:** Test assertions show SVG path data content, binary command constants, edge stride counts, and M-command counts with descriptive messages.
- **No runtime signals:** Pure computation, no async/IO.
