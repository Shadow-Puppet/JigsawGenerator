---
id: T01
parent: S01
milestone: M002
provides:
  - linesweeper 0.3.0 dependency in puzzle-core
  - heart_path(width, height) -> BezPath shape constructor
  - star_path(width, height, points) -> BezPath shape constructor
  - WASM compilation proof for linesweeper dependency tree
key_files:
  - crates/puzzle-core/Cargo.toml
  - crates/puzzle-core/src/shapes.rs
  - crates/puzzle-core/src/lib.rs
key_decisions:
  - Heart shape uses 4 cubic beziers with empirically tuned control points for natural look
  - Star inner radius at 40% of outer radius per plan specification
  - Star starts at -PI/2 so first outer vertex points upward
patterns_established:
  - Shape constructors take (width, height) and return closed BezPaths centered in those dimensions
  - All shapes close with close_path() — required for linesweeper boolean ops
observability_surfaces:
  - cargo test -- shapes runs 5 shape-specific unit tests
  - cargo check --target wasm32-unknown-unknown proves WASM compilation
duration: 8m
verification_result: passed
completed_at: 2026-03-22
blocker_discovered: false
---

# T01: Add linesweeper dependency, create shape library, and prove WASM compilation

**Added linesweeper 0.3.0 dependency, heart_path and star_path BezPath constructors in shapes.rs, and proved full WASM compilation of the dependency tree**

## What Happened

Added `linesweeper = "0.3"` to puzzle-core's Cargo.toml. Created `crates/puzzle-core/src/shapes.rs` with two public shape constructors: `heart_path(width, height)` builds a heart from 4 cubic Bézier curves (bottom tip → left bulge → center dip → right bulge → back to tip), and `star_path(width, height, points)` builds a star polygon with alternating outer/inner vertices connected by line segments. Both return closed `kurbo::BezPath` instances centered within the specified dimensions. Wired the module into lib.rs with `pub mod shapes` and `pub use shapes::*`. Added 5 unit tests verifying closedness, bounding box containment, and vertex count. Confirmed all 110 tests pass (105 existing + 5 new) and that `cargo check --target wasm32-unknown-unknown` succeeds for puzzle-wasm, proving linesweeper and all transitive deps compile to WASM.

## Verification

- Ran `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — 110 tests passed, 0 failed, 0 warnings.
- Ran `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — exited 0, no errors or warnings.
- Shape tests verified: paths end with `ClosePath`, bounding boxes fit within specified dimensions (±2px tolerance for heart control points, ±1px for star), 5-pointed star has exactly 9 `LineTo` segments (1 `MoveTo` + 9 `LineTo` + 1 `ClosePath` = 10 vertices).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml` | 0 | ✅ pass | 0.3s |
| 2 | `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` | 0 | ✅ pass | 0.6s |

## Diagnostics

- Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- shapes` to exercise only the 5 shape tests.
- Use `heart_path(w, h).to_svg()` or `star_path(w, h, n).to_svg()` in test code to visually inspect SVG path data.
- If linesweeper breaks WASM compatibility in a future version, `cargo check --target wasm32-unknown-unknown` on puzzle-wasm will fail with a clear compilation error.

## Deviations

- Task plan said `test_star_path_point_count` should assert "10 line segments" but a 5-pointed star has 10 vertices total — 1 `MoveTo` + 9 `LineTo` + 1 `ClosePath`. Asserted 9 `LineTo` elements instead, which correctly represents all 10 vertices (5 outer + 5 inner).

## Known Issues

None.

## Files Created/Modified

- `crates/puzzle-core/Cargo.toml` — added `linesweeper = "0.3"` dependency
- `crates/puzzle-core/src/shapes.rs` — new file with heart_path, star_path, and 5 unit tests
- `crates/puzzle-core/src/lib.rs` — added `pub mod shapes;` and `pub use shapes::*;`
