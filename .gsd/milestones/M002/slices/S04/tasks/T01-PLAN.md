---
estimated_steps: 6
estimated_files: 3
---

# T01: Wire whimsy config into WASM endpoints with hole contour export

**Slice:** S04 — Whimsy Drag-Drop & Grid Adaptation
**Milestone:** M002

## Description

Add whimsy placement support to the Rust/WASM backend. This involves three changes:

1. **PuzzleConfig** gets four new optional fields for whimsy placement (`whimsy_shape`, `whimsy_x`, `whimsy_y`, `whimsy_scale`), all with `#[serde(default)]` for backward compatibility.

2. **BoundaryPuzzle** gets an `hole: Option<BezPath>` field. The `new_with_hole()` constructor sets it to `Some(hole)` so that export methods can include the hole contour in SVG and binary output. The `new()` constructor sets it to `None`. Both `generate_boundary_svg()` and `boundary_border_to_binary()` are updated to append the hole contour when present — the hole shape is a separate cut line for laser cutting.

3. **WASM layer** gets a `resolve_whimsy_shape()` helper that resolves a shape name to a BezPath, then applies scale (relative to shape's default dimensions) and translation (x/y offset in puzzle mm coordinates). All three WASM endpoints handle four config combinations: (a) neither border nor whimsy → rectangular, (b) border only → existing BoundaryPuzzle::new(), (c) whimsy only → BoundaryPuzzle::new_with_hole() with full rectangle as boundary, (d) both → BoundaryPuzzle::new_with_hole() with custom border as boundary.

**Key constraints:**
- PuzzleGrid doesn't implement Clone (K006) — extract whimsy config fields before consuming config in PuzzleGrid::new()
- Shape resolution stays in WASM layer (K008) — `resolve_whimsy_shape()` lives in lib.rs, not puzzle-core
- Follow the existing `resolve_border_shape()` pattern for the new helper
- The hole contour in SVG/binary is the whimsy shape itself (translated/scaled), not a clipping operation — it's a physical cut line (D017)

## Steps

1. **Add whimsy fields to PuzzleConfig** in `crates/puzzle-core/src/config.rs`:
   - Add `whimsy_shape: Option<String>`, `whimsy_x: Option<f64>`, `whimsy_y: Option<f64>`, `whimsy_scale: Option<f64>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
   - No validation needed — whimsy placement is free-form (D016)

2. **Add hole field to BoundaryPuzzle** in `crates/puzzle-core/src/boundary.rs`:
   - Add `pub hole: Option<BezPath>` to the struct
   - In `new()`: set `hole: None`
   - In `new_with_hole()`: set `hole: Some(hole)` (pass the hole parameter through)

3. **Update boundary export methods** in `crates/puzzle-core/src/boundary.rs`:
   - In `generate_boundary_svg()`: after appending the boundary shape elements, check `self.hole` — if `Some`, append the hole contour elements to `combined` BezPath (same pattern as boundary: iterate PathEl and append)
   - In `boundary_border_to_binary()`: after serializing the boundary shape, check `self.hole` — if `Some`, serialize the hole contour elements (same CMD_* pattern) appended to the data vec

4. **Add boundary hole export tests** in `crates/puzzle-core/src/boundary.rs`:
   - `test_hole_contour_in_svg` — construct BoundaryPuzzle with `new_with_hole()`, verify SVG contains elements from both the boundary and hole shapes
   - `test_hole_contour_in_binary` — verify binary border data is longer when hole is present vs when it's not
   - `test_no_hole_unchanged` — verify `new()` still produces the same SVG as before (no regression)

5. **Add `resolve_whimsy_shape()` helper** in `crates/puzzle-wasm/src/lib.rs`:
   - Takes `name: &str`, `x: f64`, `y: f64`, `scale: f64`, `base_width: f64`, `base_height: f64` (puzzle dimensions for default shape sizing)
   - Resolves shape name to BezPath via `heart_path()` / `star_path()` with `base_width * 0.3` and `base_height * 0.3` as default size (whimsy should be smaller than the puzzle)
   - Applies scale: multiply all coordinates by `scale`
   - Applies translate: offset all coordinates by `(x, y)` using kurbo `Affine::translate()`
   - Returns `Result<BezPath, String>` — error for unknown shapes

6. **Update all three WASM endpoints** in `crates/puzzle-wasm/src/lib.rs`:
   - Extract `whimsy_shape`, `whimsy_x`, `whimsy_y`, `whimsy_scale` from config before `PuzzleGrid::new()` consumes it
   - After grid creation and connector generation, resolve the whimsy shape if params are present
   - Handle four combinations:
     - Neither: existing rectangular path
     - Border only: existing `BoundaryPuzzle::new(grid, boundary)`
     - Whimsy only: create a full-rectangle boundary (`rect_boundary(width, height)`), then `BoundaryPuzzle::new_with_hole(grid, rect_boundary, whimsy_path)`
     - Both: `BoundaryPuzzle::new_with_hole(grid, border_boundary, whimsy_path)`
   - For each boundary case, use `bp.generate_boundary_svg()`, `bp.boundary_edges_to_binary()`, `bp.boundary_border_to_binary()`, `bp.included_cell_count()`
   - Add WASM tests: `test_generate_svg_with_whimsy`, `test_generate_grid_with_whimsy_fewer_pieces`, `test_whimsy_config_backward_compat`, `test_whimsy_plus_border`

## Must-Haves

- [ ] `PuzzleConfig` has `whimsy_shape`, `whimsy_x`, `whimsy_y`, `whimsy_scale` fields with `#[serde(default)]`
- [ ] `BoundaryPuzzle` has `hole: Option<BezPath>` field, set in `new_with_hole()`, `None` in `new()`
- [ ] `generate_boundary_svg()` includes hole contour when present
- [ ] `boundary_border_to_binary()` includes hole contour when present
- [ ] `resolve_whimsy_shape()` helper in WASM layer resolves name + applies scale/translate
- [ ] All three WASM endpoints handle all four config combinations correctly
- [ ] Config without whimsy fields deserializes cleanly (backward compat)
- [ ] All existing boundary tests still pass (no regression)

## Verification

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` — all boundary tests pass including new hole-export tests
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — all WASM tests pass including new whimsy tests
- `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — compiles to WASM
- `grep -q 'whimsy_shape' crates/puzzle-core/src/config.rs` — config fields exist
- `grep -q 'resolve_whimsy_shape' crates/puzzle-wasm/src/lib.rs` — WASM helper exists

## Inputs

- `crates/puzzle-core/src/config.rs` — existing PuzzleConfig to extend with whimsy fields
- `crates/puzzle-core/src/boundary.rs` — existing BoundaryPuzzle struct and export methods to extend with hole support
- `crates/puzzle-wasm/src/lib.rs` — existing WASM endpoints and `resolve_border_shape()` pattern to follow

## Expected Output

- `crates/puzzle-core/src/config.rs` — PuzzleConfig with whimsy_shape/whimsy_x/whimsy_y/whimsy_scale fields
- `crates/puzzle-core/src/boundary.rs` — BoundaryPuzzle with hole field, updated SVG/binary exports including hole contour, new hole-export tests
- `crates/puzzle-wasm/src/lib.rs` — resolve_whimsy_shape() helper, four-combination endpoint logic, new whimsy WASM tests

## Observability Impact

- **New structured error**: WASM endpoints return `{"error":"Unknown whimsy shape: '...'. Valid shapes: heart, star"}` for invalid `whimsy_shape` — inspectable via console or network tab
- **Piece count reflects hole**: `generate_grid()` response `piece_breakdown.total` decreases when whimsy is present, directly reflecting the hole's impact on the grid
- **Binary border data includes hole**: `boundary_border_to_binary()` appends hole contour commands after boundary commands — downstream Canvas drawing and SVG export include the whimsy cut line without special handling
- **Backward-compatible**: all three signals are absent when whimsy fields are omitted, preserving existing behavior
