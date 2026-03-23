# S01: Shape Library & Boolean Op Foundation

**Goal:** Heart and star shapes defined as reusable kurbo BezPaths in puzzle-core, with linesweeper-backed boolean intersection/difference wrappers proven to compile to WASM.
**Demo:** Unit tests prove linesweeper compiles to WASM and boolean intersection/difference works on heart and star BezPaths; shapes defined as reusable kurbo paths in puzzle-core.

## Must-Haves

- linesweeper 0.3.0 added to puzzle-core dependencies with kurbo 0.13 compatibility preserved
- `heart_path(width, height) -> BezPath` produces a valid closed path centered in the given dimensions
- `star_path(width, height, points) -> BezPath` produces a valid closed path centered in the given dimensions
- `mask_intersection(base, shape) -> Result<BezPath, String>` wraps `linesweeper::binary_op` with `BinaryOp::Intersection`
- `mask_difference(base, shape) -> Result<BezPath, String>` wraps `linesweeper::binary_op` with `BinaryOp::Difference`
- All 105 existing tests still pass
- `cargo check --target wasm32-unknown-unknown` succeeds for puzzle-wasm (proving linesweeper compiles to WASM transitively)
- Boolean ops are deterministic (same inputs → same output)

## Proof Level

- This slice proves: contract
- Real runtime required: no
- Human/UAT required: no

## Verification

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all existing 105 tests pass plus new tests for shapes and masking
- `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — succeeds with no errors

## Integration Closure

- Upstream surfaces consumed: none (first slice)
- New wiring introduced in this slice: `pub mod shapes` and `pub mod masking` in `crates/puzzle-core/src/lib.rs`; `linesweeper = "0.3"` in `crates/puzzle-core/Cargo.toml`
- What remains before the milestone is truly usable end-to-end: S02 (boundary-aware grid generation), S03 (border UI), S04 (whimsy drag-drop), S05 (sub-puzzle splitting), S06 (export polish)

## Tasks

- [x] **T01: Add linesweeper dependency, create shape library, and prove WASM compilation** `est:45m`
  - Why: Retires the highest risk (WASM compilation of linesweeper) and provides the shape BezPath constructors that masking and all downstream slices consume. Satisfies R001.
  - Files: `crates/puzzle-core/Cargo.toml`, `crates/puzzle-core/src/shapes.rs`, `crates/puzzle-core/src/lib.rs`
  - Do: Add `linesweeper = "0.3"` to puzzle-core Cargo.toml dependencies. Create `shapes.rs` with `heart_path(width: f64, height: f64) -> BezPath` (4 cubic beziers, closed, centered) and `star_path(width: f64, height: f64, points: usize) -> BezPath` (alternating outer/inner radii line segments, closed, centered). Add `pub mod shapes;` and `pub use shapes::*;` to lib.rs. Include unit tests in shapes.rs verifying: paths are closed (end with ClosePath), non-empty, bounding box within specified dimensions. Run `cargo check --target wasm32-unknown-unknown` on puzzle-wasm to prove transitive WASM compilation. Use `FillRule::EvenOdd` in any test that exercises binary_op. Use `kurbo::Shape::bounding_box()` for dimension assertions.
  - Verify: `cargo test --manifest-path crates/puzzle-core/Cargo.toml` passes (all 105 existing + new shape tests), `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` succeeds
  - Done when: shapes.rs exists with heart_path and star_path, both produce valid closed BezPaths, all tests pass, WASM compilation succeeds

- [x] **T02: Build masking wrappers with boolean op integration tests** `est:30m`
  - Why: Provides the thin masking API (intersection and difference) that S02-S05 consume for grid clipping, border masking, and whimsy placement. Proves linesweeper boolean ops produce correct, deterministic output on the shapes from T01.
  - Files: `crates/puzzle-core/src/masking.rs`, `crates/puzzle-core/src/lib.rs`
  - Do: Create `masking.rs` with `mask_intersection(base: &BezPath, shape: &BezPath) -> Result<BezPath, String>` and `mask_difference(base: &BezPath, shape: &BezPath) -> Result<BezPath, String>`. Both call `linesweeper::binary_op` with `FillRule::EvenOdd` and the appropriate `BinaryOp` variant. Convert `linesweeper::topology::Contours` result to a single `BezPath` by iterating `.contours()` and appending each contour's `.path` elements. Map `linesweeper::Error` to `String` via `.to_string()`. Add `pub mod masking;` and `pub use masking::*;` to lib.rs. Include unit tests: intersection of rectangle + heart produces non-empty path, difference of rectangle - star produces non-empty path, results are deterministic (call twice with same inputs, compare BezPath SVG output), empty intersection (non-overlapping shapes) produces empty path.
  - Verify: `cargo test --manifest-path crates/puzzle-core/Cargo.toml` passes (all existing + shapes + masking tests), `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` still succeeds
  - Done when: masking.rs exists with both wrapper functions, unit tests exercise intersection/difference/determinism, all tests pass, WASM compilation still succeeds

## Observability / Diagnostics

- **Runtime signals:** None — this slice is a pure compile-time library; no runtime processes, servers, or background tasks.
- **Inspection surfaces:** `cargo test --manifest-path crates/puzzle-core/Cargo.toml` runs shape and masking unit tests. `cargo check --target wasm32-unknown-unknown` proves WASM compilation. Individual shape functions can be inspected via `kurbo::BezPath::to_svg()` string output in tests.
- **Failure visibility:** Compilation errors in `cargo check` immediately surface missing or incompatible dependencies. Test failures in `cargo test` report which shape invariant (closed path, bounding box, vertex count) broke. WASM check failure indicates a dependency uses APIs not available in wasm32.
- **Redaction constraints:** None — no secrets, credentials, or user data involved.

## Files Likely Touched

- `crates/puzzle-core/Cargo.toml`
- `crates/puzzle-core/src/lib.rs`
- `crates/puzzle-core/src/shapes.rs`
- `crates/puzzle-core/src/masking.rs`
