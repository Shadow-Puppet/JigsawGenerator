---
id: T03
parent: S02
milestone: M002
provides:
  - PuzzleConfig.border_shape — optional field for boundary shape selection (serde-default None, backward compatible)
  - generate_svg() with border_shape returns boundary-aware SVG (heart/star contour as border)
  - generate_edges_binary() with border_shape returns boundary-aware binary edge and border data
  - generate_grid() with border_shape filters pieces to only included cells, adjusts piece counts
  - resolve_border_shape() helper centralizes shape name → BezPath resolution with error handling
key_files:
  - crates/puzzle-core/src/config.rs
  - crates/puzzle-wasm/src/lib.rs
  - crates/puzzle-wasm/Cargo.toml
key_decisions:
  - Added kurbo as direct dependency to puzzle-wasm (already transitive via puzzle-core) to name BezPath type in resolve_border_shape helper
  - generate_grid() uses enum GridAccess pattern to unify PuzzleGrid/BoundaryPuzzle ownership without cloning
  - Binary endpoint tests replaced with shape-resolution + SVG-cache tests since js_sys calls panic on native targets
patterns_established:
  - resolve_border_shape() as single authoritative mapping from shape name strings to BezPaths — new shapes only need one match arm
  - border_shape clone-before-move pattern for functions that need shape name after grid ownership transfers
observability_surfaces:
  - Unknown border_shape values produce structured error JSON from all three WASM endpoints
  - cargo test --manifest-path crates/puzzle-wasm/Cargo.toml runs all 22 WASM tests including 9 boundary-related
  - generate_edges_binary with border_shape caches boundary SVG, retrievable via get_cached_svg()
duration: 18m
verification_result: passed
completed_at: 2026-03-23
blocker_discovered: false
---

# T03: Wire boundary puzzle through WASM endpoints

**Extended all three WASM endpoints (generate_svg, generate_edges_binary, generate_grid) to accept optional border_shape parameter — heart/star shapes produce boundary-aware SVG and binary output while existing behavior is fully preserved when border_shape is absent.**

## What Happened

1. **Config extension:** Added `border_shape: Option<String>` to `PuzzleConfig` with `#[serde(default, skip_serializing_if = "Option::is_none")]`. Updated all struct literals across puzzle-core (config.rs, boundary.rs, grid.rs, svg_export.rs, binary_export.rs) and the Default impl. Backward compatible — existing JSON without `border_shape` deserializes cleanly with None.

2. **generate_svg():** When `border_shape` is set to "heart" or "star", creates the BezPath at puzzle dimensions, constructs `BoundaryPuzzle`, and calls `generate_boundary_svg()`. When None, delegates to existing `puzzle_core::generate_svg()`. Unknown shape names return error JSON.

3. **generate_edges_binary():** Same pattern — clones `border_shape` before grid creation (ownership transfer), then branches: boundary-aware path uses `BoundaryPuzzle` for edges/border binary data and SVG cache; rectangular path uses existing `edges_to_binary()`/`border_to_binary()`. Result object shape unchanged: `{edges, border, width, height}`.

4. **generate_grid():** Uses `GridAccess` enum to unify ownership (grid moves into BoundaryPuzzle when active, stays owned otherwise). Filters pieces to only boundary-included cells, adjusts piece breakdown counts, and reports boundary-aware internal_count via `included_edge_count()`.

5. **Helper:** `resolve_border_shape(name, width, height)` is the single mapping from shape name strings to BezPaths. Adding a new shape requires one match arm here.

6. **Tests:** Added 9 new tests (22 total) covering heart SVG, star SVG, backward compatibility, fewer pieces with border, invalid shape error, shape resolution (heart/star/unknown), and heart SVG determinism.

## Verification

All five slice-level verification commands pass:

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — 132 tests pass (all existing + boundary tests from T01/T02)
- `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` — 19 boundary-specific tests pass
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — 22 tests pass (13 existing + 9 new boundary WASM tests)
- `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — WASM compilation succeeds
- `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary_no_cells` — empty boundary edge case passes

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml` | 0 | ✅ pass | 0.06s |
| 2 | `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` | 0 | ✅ pass | 0.01s |
| 3 | `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` | 0 | ✅ pass | 1.77s |
| 4 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` | 0 | ✅ pass | 0.01s |
| 5 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary_no_cells` | 0 | ✅ pass | 0.00s |

## Diagnostics

- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` runs all 22 WASM tests including 9 border-shape tests
- `resolve_border_shape()` returns structured error for unknown shapes — visible in endpoint error JSON
- `generate_edges_binary()` with `border_shape` caches boundary SVG retrievable via `get_cached_svg()`
- `generate_grid()` with `border_shape` reports filtered piece counts and boundary-aware internal edge count

## Deviations

- Replaced planned `generate_edges_binary` JsValue-based tests with `resolve_border_shape()` unit tests and SVG-level tests, because `js_sys` APIs (Float64Array, Reflect::get) panic on native test targets. The binary export logic is already thoroughly tested in puzzle-core's T02 boundary binary tests (9 tests). The WASM wiring is validated by the SVG tests (which do work natively) since both endpoints follow the same branching pattern.
- Added `kurbo = "0.13"` to puzzle-wasm/Cargo.toml to name the `BezPath` type in `resolve_border_shape()`. This adds no binary bloat since kurbo is already transitively included via puzzle-core.
- Added 9 tests instead of the planned 5 — extra coverage for shape resolution, star border, determinism, and backward compatibility.

## Known Issues

None.

## Files Created/Modified

- `crates/puzzle-core/src/config.rs` — added `border_shape: Option<String>` field to PuzzleConfig with serde defaults
- `crates/puzzle-wasm/src/lib.rs` — extended generate_svg(), generate_edges_binary(), generate_grid() with border shape support; added resolve_border_shape() helper; added 9 new tests
- `crates/puzzle-wasm/Cargo.toml` — added kurbo dependency for BezPath type
- `crates/puzzle-core/src/boundary.rs` — updated test_config helper with border_shape field
- `crates/puzzle-core/src/grid.rs` — updated test_config helper with border_shape field
- `crates/puzzle-core/src/svg_export.rs` — updated test_config helper with border_shape field
- `crates/puzzle-core/src/binary_export.rs` — updated test_config helper with border_shape field
