---
estimated_steps: 4
estimated_files: 2
---

# T02: Build masking wrappers with boolean op integration tests

**Slice:** S01 — Shape Library & Boolean Op Foundation
**Milestone:** M002

## Description

Create the masking module that provides thin wrappers around linesweeper's `binary_op` function for intersection (mask) and difference (reverse-mask) operations. These are the core geometric primitives that S02-S05 consume for grid clipping, border masking, and whimsy placement. Tests prove boolean ops produce correct, deterministic output when applied to the shapes from T01.

**Key skill:** Pure Rust — linesweeper API usage, BezPath manipulation, unit testing. No frontend/UI work.

**linesweeper API reference (executor needs this):**
- `linesweeper::binary_op(set_a: &BezPath, set_b: &BezPath, fill_rule: FillRule, op: BinaryOp) -> Result<topology::Contours, Error>`
- `FillRule::EvenOdd` — use this consistently
- `BinaryOp::Intersection` and `BinaryOp::Difference`
- `Contours` has `.contours() -> impl Iterator<Item = &Contour>` (iterates all contours ignoring hierarchy)
- Each `Contour` has `path: BezPath`, `parent: Option<ContourIdx>`, `outer: bool`
- `Error` implements `Display`, so map to String via `.to_string()`
- Both input paths **must be closed** (end with ClosePath) or results are silently wrong
- Boolean ops can produce multiple disjoint contour paths — must concatenate all into a single BezPath

## Steps

1. **Create masking.rs** — Create `crates/puzzle-core/src/masking.rs`. Implement two public functions:

   ```rust
   pub fn mask_intersection(base: &BezPath, shape: &BezPath) -> Result<BezPath, String>
   pub fn mask_difference(base: &BezPath, shape: &BezPath) -> Result<BezPath, String>
   ```

   Both call `linesweeper::binary_op(base, shape, FillRule::EvenOdd, op)` where `op` is `BinaryOp::Intersection` or `BinaryOp::Difference`. Convert the `Result<Contours, Error>` to `Result<BezPath, String>`:
   - On success: iterate `contours.contours()`, for each `Contour` iterate its `path` elements via `contour.path.iter()` and append each `PathEl` to a new `BezPath` (use `move_to`, `line_to`, `curve_to`, `close_path` matching the `PathEl` variant). This concatenates multiple disjoint contour paths into a single BezPath with multiple subpaths.
   - On error: map via `.to_string()`

   Import types: `use kurbo::{BezPath, PathEl};` and `use linesweeper::{binary_op, BinaryOp, FillRule};`

2. **Wire masking module into lib.rs** — Add `pub mod masking;` and `pub use masking::*;` to `crates/puzzle-core/src/lib.rs`, alongside the shapes module added in T01.

3. **Add unit tests in masking.rs** — Add `#[cfg(test)] mod tests` with these tests:
   - `test_intersection_heart_and_rect`: Create a rectangle BezPath (200×200 at origin using `move_to`/`line_to`/`close_path`) and `heart_path(100.0, 100.0)` offset to be centered inside the rect. Call `mask_intersection`. Assert result is `Ok`, the returned BezPath is non-empty (has elements). Use `kurbo::Shape::bounding_box()` to verify the result is smaller than the rectangle.
   - `test_difference_rect_minus_star`: Create same rectangle, and `star_path(80.0, 80.0, 5)` centered inside it. Call `mask_difference`. Assert result is `Ok` and non-empty. The bounding box of the result should still be close to the rectangle's bounding box (since we're cutting a hole inside).
   - `test_intersection_deterministic`: Call `mask_intersection` twice with identical inputs. Convert both results to SVG string via `.to_svg()`. Assert strings are equal.
   - `test_no_overlap_intersection_empty`: Create two non-overlapping rectangles (one at 0,0 and one at 500,500, both 100×100). Call `mask_intersection`. Assert result is `Ok` and the BezPath has no elements (they don't overlap).

   Import `crate::shapes::{heart_path, star_path}` in the test module.

4. **Run full verification** — Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml` to verify all tests pass (existing 105 + shape tests from T01 + new masking tests). Run `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` to verify WASM compilation still succeeds with the new module.

## Must-Haves

- [ ] `mask_intersection` wraps `binary_op` with `BinaryOp::Intersection` and `FillRule::EvenOdd`
- [ ] `mask_difference` wraps `binary_op` with `BinaryOp::Difference` and `FillRule::EvenOdd`
- [ ] Contours result correctly concatenated into a single BezPath (handles multiple disjoint contours)
- [ ] `linesweeper::Error` mapped to `String` via `.to_string()`
- [ ] `pub mod masking;` and `pub use masking::*;` in lib.rs
- [ ] Unit tests pass for intersection, difference, determinism, and empty-overlap case
- [ ] All existing tests still pass
- [ ] WASM compilation still succeeds

## Verification

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all tests pass (105 existing + shape + masking)
- `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — exits 0

## Inputs

- `crates/puzzle-core/src/shapes.rs` — provides `heart_path` and `star_path` used in masking tests
- `crates/puzzle-core/src/lib.rs` — module root to add masking declaration (already has shapes from T01)
- `crates/puzzle-core/Cargo.toml` — already has linesweeper dependency from T01

## Expected Output

- `crates/puzzle-core/src/masking.rs` — new file with mask_intersection, mask_difference, and unit tests
- `crates/puzzle-core/src/lib.rs` — modified with masking module declaration and re-export

## Observability Impact

- **New signals:** No runtime signals — this is a pure library module with no servers, processes, or logs.
- **Inspection:** Run `cargo test -- masking` to exercise masking tests. Call `.to_svg()` on any `BezPath` returned by `mask_intersection`/`mask_difference` to inspect geometric output in SVG form.
- **Failure visibility:** Test failures report which masking invariant broke (non-empty result, bounding box constraint, determinism, empty overlap). Compilation errors surface immediately via `cargo check`. `linesweeper::Error` is mapped to human-readable strings ("one of the inputs was infinite", etc.).
