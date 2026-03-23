# S01 — Shape Library & Boolean Op Foundation — Research

**Date:** 2026-03-21
**Depth:** Deep (new dependency, WASM compilation risk, unfamiliar API)

## Summary

This slice adds linesweeper v0.3.0 as a dependency to puzzle-core and proves that boolean path operations (intersection, difference) work on kurbo BezPaths — including compilation to wasm32-unknown-unknown. It also defines the starter shape library (heart, star) as reusable BezPath constructors, and provides thin masking wrapper functions that downstream slices consume.

**The highest risk — WASM compilation — is already retired.** I verified that linesweeper 0.3.0 and its entire dependency tree (kurbo 0.13, polycool 0.4, arrayvec, rustc-hash, smallvec) compile cleanly for `wasm32-unknown-unknown` with no errors. The kurbo version (0.13) matches the existing project dependency exactly, so there are no version conflicts.

**The API is straightforward.** `linesweeper::binary_op(&path_a, &path_b, FillRule::EvenOdd, BinaryOp::Intersection)` returns `Result<Contours, Error>`. `Contours` yields `Contour` items via `.contours()` iterator, where each `Contour` has a `.path: BezPath` field, an `.outer: bool` field, and an optional `.parent` index. Converting results back to BezPath for downstream use is zero-cost — it's already a BezPath.

## Recommendation

Add linesweeper 0.3.0 to puzzle-core's `Cargo.toml`, create two new modules (`shapes.rs` and `masking.rs`), and verify with unit tests that cover:
1. Shape construction (heart and star produce valid closed BezPaths)
2. Boolean operations (intersection and difference on shape × rectangle)
3. WASM target compilation (`cargo check --target wasm32-unknown-unknown`)

Use `FillRule::EvenOdd` throughout — it handles both simple and complex paths correctly. Define shapes as functions that take `(width, height)` and return `BezPath`, centered within the given dimensions. The masking wrappers should convert `Contours` results back to a single `BezPath` by concatenating all contour paths (contours may be multiple disjoint regions after boolean ops).

## Implementation Landscape

### Key Files

- `crates/puzzle-core/Cargo.toml` — Add `linesweeper = "0.3"` to `[dependencies]`. No features needed (the `svg` feature is optional and not required). kurbo 0.13 already matches.
- `crates/puzzle-core/src/shapes.rs` — **New file.** Shape library with `heart_path(width, height) -> BezPath` and `star_path(width, height, points) -> BezPath`. The heart uses 4 cubic bezier curves; the star uses line segments between alternating outer/inner radii. Both produce closed paths centered in the given dimensions.
- `crates/puzzle-core/src/masking.rs` — **New file.** Thin wrappers: `mask_intersection(base: &BezPath, shape: &BezPath) -> Result<BezPath, String>` and `mask_difference(base: &BezPath, shape: &BezPath) -> Result<BezPath, String>`. These call `linesweeper::binary_op` and convert `Contours` → single `BezPath` by concatenating all contour `.path` fields.
- `crates/puzzle-core/src/lib.rs` — Add `pub mod shapes;` and `pub mod masking;` declarations plus re-exports.
- `crates/puzzle-wasm/Cargo.toml` — No changes needed. puzzle-wasm depends on puzzle-core; linesweeper comes transitively.

### Existing Patterns to Follow

- **Module structure:** Follow the pattern of `connector.rs` / `classic_connector.rs` — trait + implementation in separate modules. Here it's simpler: just two standalone modules with public functions.
- **BezPath usage:** The codebase already uses kurbo extensively (`svg_export.rs` builds BezPaths with `move_to`, `line_to`, `curve_to`, `close_path`). Shapes module follows the same pattern.
- **Test patterns:** Follow existing test structure — helper functions for configs, determinism checks, geometry validation. Tests in `shapes.rs` verify path closure and bounding box; tests in `masking.rs` verify boolean ops produce non-empty valid output.
- **Error handling:** Use `Result<_, String>` matching the existing codebase pattern (e.g., `PuzzleConfig::validate()`).

### Build Order

**Task 1: Add linesweeper dependency + WASM proof** (risk retirement)
- Edit `crates/puzzle-core/Cargo.toml` to add `linesweeper = "0.3"`
- Write a minimal integration test that calls `binary_op` on two shapes
- Run `cargo check --target wasm32-unknown-unknown` on puzzle-wasm to prove transitive WASM compilation
- Run `cargo test` on puzzle-core to prove boolean ops produce valid output
- This is the riskiest step — if linesweeper has WASM issues, we learn immediately. (Already pre-verified: it compiles clean.)

**Task 2: Shape library (`shapes.rs`)**
- Create `crates/puzzle-core/src/shapes.rs` with `heart_path` and `star_path`
- Heart: 4 cubic bezier segments forming a symmetric heart, closed path, centered in (width, height)
- Star: 5-pointed star using alternating outer/inner radii with line segments, closed path, centered
- Both functions take `(width: f64, height: f64)` — shapes scale to fit within these dimensions
- Add `pub mod shapes;` and `pub use shapes::*;` to `lib.rs`
- Unit tests: verify paths are closed (ends with ClosePath), non-empty, bounding box within specified dimensions

**Task 3: Masking wrappers (`masking.rs`)**
- Create `crates/puzzle-core/src/masking.rs` with intersection/difference wrappers
- Convert `Contours` result to single `BezPath` by iterating `.contours()` and appending each `.path`
- Map `linesweeper::Error` to `String` for the existing error pattern
- Add `pub mod masking;` and `pub use masking::*;` to `lib.rs`
- Unit tests: verify intersection of rect+heart produces non-empty path, difference of rect-star produces non-empty path, results are deterministic (same inputs → same output)

### Verification Approach

1. **Unit tests:** `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all existing 105 tests must still pass, plus new tests for shapes and masking
2. **WASM compilation:** `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — must succeed with no errors
3. **Determinism:** Same shape + same rect → identical BezPath output across repeated calls (critical for seed determinism in downstream slices)

The WASM full build (`wasm-pack build`) is NOT required for this slice — `cargo check --target wasm32-unknown-unknown` is sufficient to prove compilation. Full WASM integration happens in S02.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Boolean path operations | linesweeper 0.3.0 | Pure Rust, kurbo-native (same BezPath type), WASM-compatible, handles bezier intersection/difference robustly |
| 2D path primitives | kurbo 0.13.0 | Already in the project, provides BezPath, Point, Affine, Shape trait |

## Constraints

- **kurbo version must stay at 0.13** — linesweeper 0.3.0 depends on `kurbo ^0.13.0`, and the project already uses `kurbo 0.13`. These are compatible (both resolve to 0.13.0). Do NOT upgrade kurbo to 0.14+.
- **No optional features needed** — linesweeper's `svg` feature is for SVG file I/O (not needed; we handle SVG ourselves). No `default-features = false` needed.
- **Edition 2024** — both crates use `edition = "2024"`. linesweeper uses 2021, but this doesn't matter for dependency compatibility.
- **FillRule::EvenOdd** — use this consistently. NonZero fill rule behaves differently for self-intersecting paths and would produce unexpected results for complex shapes.

## Common Pitfalls

- **`Contours` is an iterator, not a collection** — `result.contours()` returns `impl Iterator<Item = &Contour>`. Must `.collect::<Vec<_>>()` if you need to index or check length. For the masking wrapper, just iterate and append.
- **Multiple contours from a single boolean op** — intersection of a concave shape with a rectangle can produce multiple disjoint contour paths. The masking wrapper must handle this by concatenating all contour paths into a single BezPath (multiple subpaths).
- **Heart shape must be a valid closed path** — linesweeper requires closed paths for boolean ops. The heart shape must end with `close_path()`. Forgetting this will cause incorrect results silently (the library doesn't panic on open paths, it just produces wrong output).
- **Star shape uses line segments, not curves** — the star polygon is straight-edged. This is fine for linesweeper (it handles mixed line/curve paths). Don't over-engineer with bezier-smoothed star points — simple polygon is cleaner for a starter shape.

## Open Risks

- **linesweeper edge cases with tangent intersections** — the library is in "early beta state." If a heart curve is exactly tangent to a grid line, the sweep-line algorithm might produce degenerate output. This is unlikely for the shapes we're defining (they're well within the puzzle rectangle), but S02 should include edge-case tests when intersecting with actual grid edges.
- **Performance under WASM** — boolean ops run in ~1ms on native for simple shapes (verified). WASM overhead is typically 2-3x, so ~3ms per op. This is well within the <50ms target for interactive use. However, S04 should benchmark with realistic grid complexity.
