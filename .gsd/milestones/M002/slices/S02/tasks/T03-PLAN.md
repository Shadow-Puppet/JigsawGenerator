---
estimated_steps: 5
estimated_files: 2
---

# T03: Wire boundary puzzle through WASM endpoints

**Slice:** S02 — Boundary-Aware Grid Generation
**Milestone:** M002

## Description

Connect the Rust `BoundaryPuzzle` engine to the browser by extending the WASM endpoints to accept an optional border shape parameter. When the user selects a border shape (S03 UI work, later), the JavaScript will pass `"border_shape": "heart"` in the config JSON, and the WASM layer will produce boundary-aware output.

**Backward compatibility is critical:** all existing WASM endpoints must continue to work identically when `border_shape` is absent or null. The new field uses `#[serde(default)]` so existing JSON payloads without it deserialize cleanly.

**Config extension:** Add `border_shape: Option<String>` to `PuzzleConfig` in `config.rs`. Valid values: `"heart"`, `"star"` (matching the shape library from S01). None/missing = rectangular puzzle (existing behavior).

**WASM changes:** In `generate_edges_binary()` and `generate_svg()`, check `config.border_shape`. When Some, create the shape BezPath at puzzle dimensions, construct `BoundaryPuzzle`, and use the boundary-aware export methods from T02. When None, use existing code path unchanged.

## Steps

1. Add `border_shape: Option<String>` field to `PuzzleConfig` in `config.rs` with `#[serde(default, skip_serializing_if = "Option::is_none")]`. No validation needed — unknown shape names are rejected at WASM layer. Ensure existing config deserialization works unchanged (the field is optional).
2. Update `generate_svg()` in `puzzle-wasm/src/lib.rs`: after creating the grid and generating connectors, check `config.border_shape`. If Some("heart"|"star"), create the shape BezPath at `(config.width, config.height)` dimensions, construct `BoundaryPuzzle::new(grid, boundary)`, and call `generate_boundary_svg()`. Otherwise use existing `puzzle_core::generate_svg()`. Return error JSON for unknown shape names.
3. Update `generate_edges_binary()` in `puzzle-wasm/src/lib.rs`: same pattern — when border shape is specified, use `BoundaryPuzzle` and its boundary export methods. The JS result object gets the same shape: `edges` (Float64Array), `border` (Float64Array), `width`, `height`. Also cache the boundary SVG.
4. Add `generate_grid()` support: when border shape is specified, include only included pieces in the response and adjust piece counts.
5. Write/extend tests in `puzzle-wasm/src/lib.rs`:
   - `test_generate_svg_with_heart_border` — SVG output contains cubic curves in border, not rectangular lines
   - `test_generate_svg_with_star_border` — star border SVG works
   - `test_generate_svg_no_border_shape_unchanged` — None/missing border_shape produces identical output to existing behavior
   - `test_generate_grid_with_border_shape_fewer_pieces` — piece count with heart border < full rectangular count
   - `test_border_shape_invalid_returns_error` — unknown shape name returns error JSON
   - Verify WASM compilation: `cargo check --target wasm32-unknown-unknown`

## Must-Haves

- [ ] `PuzzleConfig` accepts optional `border_shape` field; backward compatible
- [ ] `generate_svg()` with `border_shape: "heart"` returns heart-bordered SVG
- [ ] `generate_edges_binary()` with border shape returns boundary-aware binary data
- [ ] All existing WASM tests pass unchanged (no border_shape = no change)
- [ ] WASM target compiles: `cargo check --target wasm32-unknown-unknown -p puzzle-wasm`

## Verification

- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — all WASM tests pass
- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all core tests still pass
- `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — WASM compilation succeeds

## Inputs

- `crates/puzzle-core/src/boundary.rs` — BoundaryPuzzle with generate_boundary_svg(), boundary_edges_to_binary(), boundary_border_to_binary() from T01+T02
- `crates/puzzle-core/src/shapes.rs` — heart_path(), star_path() constructors
- `crates/puzzle-core/src/config.rs` — PuzzleConfig struct to extend
- `crates/puzzle-wasm/src/lib.rs` — existing WASM endpoints to extend

## Expected Output

- `crates/puzzle-core/src/config.rs` — updated with optional border_shape field
- `crates/puzzle-wasm/src/lib.rs` — updated WASM endpoints with border shape support and ≥5 new tests

## Observability Impact

- **New diagnostic surface:** `resolve_border_shape()` helper centralizes shape name validation — unknown names produce structured error JSON `{"error":"Unknown border shape: <name>"}` visible in both `generate_svg()` and `generate_grid()` responses.
- **Inspection:** WASM tests verify boundary-aware SVG generation, piece filtering, and error handling for invalid shapes. `generate_edges_binary()` with `border_shape` caches the boundary SVG (retrievable via `get_cached_svg()`).
- **Failure visibility:** Invalid `border_shape` values return error JSON from all three endpoints (`generate_svg`, `generate_grid`, `generate_edges_binary`) rather than panicking.
- **No runtime signals:** Pure computation, no async/IO.
