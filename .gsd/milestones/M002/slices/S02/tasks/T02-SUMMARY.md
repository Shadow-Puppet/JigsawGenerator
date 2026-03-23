---
id: T02
parent: S02
milestone: M002
provides:
  - generate_boundary_svg() — SVG export using shape contour as border with only included internal edges
  - boundary_edges_to_binary() — binary edge export for included edges only (EDGE_STRIDE format)
  - boundary_border_to_binary() — binary border export using command-prefixed format (CMD_MOVE_TO etc.)
  - build_svg_document() and edge_transform() made accessible for cross-module use
key_files:
  - crates/puzzle-core/src/boundary.rs
  - crates/puzzle-core/src/svg_export.rs
  - crates/puzzle-core/src/binary_export.rs
key_decisions:
  - Export methods live on BoundaryPuzzle (not standalone functions) so they can access cell_inside, boundary, and grid directly
  - edge_transform made fully pub (not just pub(crate)) for potential WASM crate use in T03
  - build_svg_document made pub(crate) since it's only needed within puzzle-core
  - CMD_* constants made pub(crate) to keep them internal while allowing boundary.rs access
patterns_established:
  - BoundaryPuzzle export methods delegate to included_h_edges()/included_v_edges() for filtering, then use same serialization logic as the rectangular exports
  - append_included_edge_paths() private helper factored out to avoid SVG path construction duplication
observability_surfaces:
  - cargo test -- boundary_svg runs all SVG export boundary tests
  - cargo test -- boundary_binary runs all binary export boundary tests
  - SVG output can be inspected for C commands (curved border) and M-command count (edge filtering correctness)
duration: 12m
verification_result: passed
completed_at: 2026-03-23
blocker_discovered: false
---

# T02: Add boundary-aware SVG and binary export

**Added generate_boundary_svg(), boundary_edges_to_binary(), and boundary_border_to_binary() methods to BoundaryPuzzle — heart-shaped SVG output contains cubic bezier border curves with only included internal edges, and binary formats match existing command constants.**

## What Happened

Extended the `BoundaryPuzzle` struct with three export methods:

1. **`generate_boundary_svg()`** — Builds a complete SVG document using the boundary BezPath as the border subpath (replacing the rectangular border), then appends connector curves for only included internal edges. The heart boundary produces `C` (cubic bezier) commands, proving non-rectangular geometry in the output.

2. **`boundary_edges_to_binary()`** — Serializes only included internal edges using the same EDGE_STRIDE (36 floats) format as `edges_to_binary()`. Iterates `included_h_edges()` and `included_v_edges()` indices to access edge connectors.

3. **`boundary_border_to_binary()`** — Serializes the boundary shape path using the command-prefixed format (CMD_MOVE_TO, CMD_LINE_TO, CMD_CURVE_TO, CMD_CLOSE), matching `border_to_binary()`.

Visibility changes to support cross-module access:
- `edge_transform` in `svg_export.rs`: `pub(crate)` → `pub` (needed for WASM crate in T03)
- `build_svg_document` in `svg_export.rs`: private → `pub(crate)`
- `CMD_MOVE_TO`, `CMD_LINE_TO`, `CMD_CURVE_TO`, `CMD_CLOSE` in `binary_export.rs`: private → `pub(crate)`

## Verification

All verification commands pass:

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — 132 tests pass (114 original + 9 T01 + 9 T02)
- `cargo test -- boundary` — 19 boundary-specific tests pass (9 T01 + 9 T02 + 1 config)
- `cargo test -- boundary_no_cells` — empty-grid edge case passes
- `cargo check --manifest-path crates/puzzle-core/Cargo.toml` — no warnings

9 new export tests added:
- `test_boundary_svg_contains_cubic_curves` — heart SVG has `C` commands ✅
- `test_boundary_svg_excludes_outside_edges` — M-count = included edges + 1 ✅
- `test_boundary_svg_deterministic` — same seed = identical SVG ✅
- `test_boundary_svg_has_valid_structure` — SVG wrapper elements correct ✅
- `test_boundary_binary_edge_count` — binary count matches included_edge_count ✅
- `test_boundary_binary_border_starts_with_moveto` — starts with CMD_MOVE_TO ✅
- `test_boundary_binary_border_has_curves` — heart border has CMD_CURVE_TO ✅
- `test_boundary_binary_border_has_close` — border has CMD_CLOSE ✅
- `test_boundary_binary_edges_deterministic` — same seed = identical binary ✅

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml` | 0 | ✅ pass | 0.01s |
| 2 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` | 0 | ✅ pass | 0.02s |
| 3 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary_no_cells` | 0 | ✅ pass | 0.00s |
| 4 | `cargo check --manifest-path crates/puzzle-core/Cargo.toml` | 0 | ✅ pass | 0.05s |

## Diagnostics

- `cargo test -- boundary_svg` runs SVG export boundary tests
- `cargo test -- boundary_binary` runs binary export boundary tests
- SVG output can be visually inspected: `C` commands prove curved boundary; M-command count proves edge filtering
- Binary output can be validated: first float = CMD_MOVE_TO (0.0); presence of CMD_CURVE_TO (2.0) proves curves; data.len() / EDGE_STRIDE gives edge count

## Deviations

- Added `test_boundary_svg_has_valid_structure` (4th SVG test) and `test_boundary_binary_border_has_close` plus `test_boundary_binary_edges_deterministic` (extra binary tests) beyond the 6 specified in the plan — 9 total tests instead of 6, exceeding the "at least 5" must-have.
- Added private `append_included_edge_paths()` helper to factor out edge path construction shared between SVG and potential future uses.

## Known Issues

None.

## Files Created/Modified

- `crates/puzzle-core/src/boundary.rs` — added generate_boundary_svg(), boundary_edges_to_binary(), boundary_border_to_binary() methods + append_included_edge_paths() helper + 9 export tests
- `crates/puzzle-core/src/svg_export.rs` — changed edge_transform to pub, build_svg_document to pub(crate)
- `crates/puzzle-core/src/binary_export.rs` — changed CMD_MOVE_TO, CMD_LINE_TO, CMD_CURVE_TO, CMD_CLOSE to pub(crate)
