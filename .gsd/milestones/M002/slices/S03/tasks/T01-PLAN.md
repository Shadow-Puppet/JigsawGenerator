---
estimated_steps: 3
estimated_files: 1
---

# T01: Add piece_count to WASM generate_edges_binary response

**Slice:** S03 — Custom Border UI
**Milestone:** M002

## Description

The JavaScript frontend currently computes piece count as `rows * cols`, which is wrong for boundary puzzles (heart, star shapes exclude cells outside the boundary). The WASM `generate_edges_binary()` function already creates a `BoundaryPuzzle` when `border_shape` is set, so it knows the real included cell count. This task adds a `piece_count` field to the returned JS object so the frontend can display the correct number.

The change is small: after building the result JS object (which already has `edges`, `border`, `width`, `height`), add one more `Reflect::set` call for `piece_count`. For boundary puzzles, use `BoundaryPuzzle::included_cell_count()`. For rectangular puzzles, use `rows * cols`.

**Relevant skill:** None needed — this is a minimal Rust/WASM change.

## Steps

1. In `crates/puzzle-wasm/src/lib.rs`, find the `generate_edges_binary()` function. In the boundary branch (where `BoundaryPuzzle::new(grid, boundary)` is called), capture `bp.included_cell_count()` before consuming `bp` for edge/border/SVG export. In the rectangular branch, compute `rows as usize * cols as usize` from the grid config. Store the count in a variable `piece_count: usize`.

2. After the result JS object is built (the block with `Reflect::set` for edges, border, width, height), add:
   ```rust
   let _ = js_sys::Reflect::set(
       &obj,
       &JsValue::from_str("piece_count"),
       &JsValue::from_f64(piece_count as f64),
   );
   ```

3. Add a test `test_generate_edges_binary_piece_count` that:
   - Calls `generate_edges_binary()` with a rectangular config (no border_shape) and verifies `piece_count == rows * cols`
   - Calls `generate_edges_binary()` with `"border_shape":"heart"` and verifies `piece_count < rows * cols` and `piece_count > 0`
   - Note: `generate_edges_binary` returns `JsValue`, so use `js_sys::Reflect::get()` to read the `piece_count` property. Since these are `#[cfg(test)]` tests (not `#[wasm_bindgen_test]`), the `JsValue` operations work in native test mode.

   **Important:** The existing WASM tests in this file are `#[test]` (not wasm_bindgen_test), so they run with `cargo test`. The `JsValue`/`Reflect` operations work in native test mode because `wasm-bindgen` provides mock implementations. Follow the same pattern as the existing tests.

## Must-Haves

- [ ] `generate_edges_binary()` returns `piece_count` in the JS object for rectangular puzzles (value = `rows * cols`)
- [ ] `generate_edges_binary()` returns `piece_count` in the JS object for boundary puzzles (value = actual included cell count, less than `rows * cols`)
- [ ] At least one test verifies `piece_count` is present and correct for both cases
- [ ] All existing WASM tests still pass
- [ ] WASM target compiles: `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown`

## Verification

- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — all tests pass (existing + new piece_count test)
- `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — WASM compiles
- `grep -q 'piece_count' crates/puzzle-wasm/src/lib.rs` — field name exists in source

## Inputs

- `crates/puzzle-wasm/src/lib.rs` — current WASM endpoints with `generate_edges_binary()` function
- `crates/puzzle-core/src/boundary.rs` — `BoundaryPuzzle` struct with `included_cell_count()` method

## Expected Output

- `crates/puzzle-wasm/src/lib.rs` — modified to include `piece_count` in `generate_edges_binary()` return object, plus new test
