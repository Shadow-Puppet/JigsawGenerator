---
estimated_steps: 5
estimated_files: 3
---

# T01: Add linesweeper dependency, create shape library, and prove WASM compilation

**Slice:** S01 — Shape Library & Boolean Op Foundation
**Milestone:** M002

## Description

Add linesweeper 0.3.0 to puzzle-core's dependencies, create the shape library module with heart and star BezPath constructors, and prove the full dependency tree compiles to wasm32-unknown-unknown. This retires the highest-risk item in the milestone (WASM compilation of a new dependency) and delivers the shape primitives that every downstream slice consumes. Satisfies requirement R001 (heart and star shapes as kurbo BezPaths).

**Key skill:** This is pure Rust — new module creation, kurbo BezPath construction, unit testing. No frontend/UI work.

## Steps

1. **Add linesweeper dependency** — Edit `crates/puzzle-core/Cargo.toml` to add `linesweeper = "0.3"` under `[dependencies]`. Do NOT add any features. The existing `kurbo = { version = "0.13", features = ["serde"] }` is compatible with linesweeper's `kurbo ^0.13.0` requirement — do not change the kurbo version. Run `cargo check --manifest-path crates/puzzle-core/Cargo.toml` to confirm the dependency resolves.

2. **Create shapes.rs with heart_path** — Create `crates/puzzle-core/src/shapes.rs`. Implement `pub fn heart_path(width: f64, height: f64) -> BezPath` that constructs a heart shape using 4 cubic bezier curves (`move_to`, `curve_to` × 4, `close_path`). The heart must be centered within the given (width, height) dimensions. The top of the heart has two bumps, the bottom comes to a point. Use `kurbo::BezPath`, `kurbo::Point`. The path must end with `close_path()` — linesweeper requires closed paths for boolean ops (open paths produce silently wrong results).

3. **Add star_path to shapes.rs** — Implement `pub fn star_path(width: f64, height: f64, points: usize) -> BezPath` that constructs a star polygon. Use alternating outer radius (fills the bounding box) and inner radius (typically 40% of outer) with `line_to` segments between vertices placed at angular intervals of `2π / (2 * points)`. Close with `close_path()`. Center within (width, height).

4. **Wire shapes module into lib.rs** — Add `pub mod shapes;` and `pub use shapes::*;` to `crates/puzzle-core/src/lib.rs`. Place the new module declarations alongside the existing ones (after `seed` or at the end of the module list).

5. **Add unit tests and verify WASM compilation** — Add `#[cfg(test)] mod tests` in shapes.rs with tests:
   - `test_heart_path_is_closed`: heart_path(100.0, 100.0) produces a path that ends with `PathEl::ClosePath` and has > 0 elements.
   - `test_heart_path_bounding_box`: `use kurbo::Shape;` then `heart_path(100.0, 80.0).bounding_box()` fits within Rect(0, 0, 100, 80) (with small tolerance for control points).
   - `test_star_path_is_closed`: star_path(100.0, 100.0, 5) produces a closed, non-empty path.
   - `test_star_path_bounding_box`: bounding box fits within specified dimensions.
   - `test_star_path_point_count`: a 5-pointed star has 10 line segments (5 outer + 5 inner vertices).
   - Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all 105 existing tests plus new tests pass.
   - Run `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — must succeed with no errors. This proves linesweeper + all transitive deps compile to WASM.

## Must-Haves

- [ ] `linesweeper = "0.3"` added to `crates/puzzle-core/Cargo.toml` dependencies
- [ ] `heart_path(width, height)` returns a valid closed BezPath centered in dimensions
- [ ] `star_path(width, height, points)` returns a valid closed BezPath centered in dimensions
- [ ] `pub mod shapes;` and `pub use shapes::*;` in lib.rs
- [ ] All 105 existing tests still pass
- [ ] New shape unit tests pass
- [ ] `cargo check --target wasm32-unknown-unknown` succeeds for puzzle-wasm

## Verification

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all tests pass (105 existing + new shape tests)
- `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — exits 0

## Observability Impact

- **Signals changed:** New unit tests (`shapes::tests::*`) appear in `cargo test` output — 5 new test cases added to the existing 105.
- **Inspection surface:** Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- shapes` to exercise only shape tests. Use `heart_path(...).to_svg()` or `star_path(...).to_svg()` in test code to visually inspect SVG path data.
- **Failure visibility:** If linesweeper breaks WASM compatibility in a future version, `cargo check --target wasm32-unknown-unknown` on puzzle-wasm will fail with a clear compilation error. Shape invariant violations (unclosed paths, out-of-bounds geometry) are caught by the unit tests.

## Inputs

- `crates/puzzle-core/Cargo.toml` — existing dependency file to add linesweeper to
- `crates/puzzle-core/src/lib.rs` — existing module root to add shapes module declaration
- `crates/puzzle-wasm/Cargo.toml` — needed for WASM compilation check (depends on puzzle-core transitively)

## Expected Output

- `crates/puzzle-core/Cargo.toml` — modified with linesweeper dependency
- `crates/puzzle-core/src/shapes.rs` — new file with heart_path, star_path, and unit tests
- `crates/puzzle-core/src/lib.rs` — modified with shapes module declaration and re-export
