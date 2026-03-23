# S01: Shape Library & Boolean Op Foundation — UAT

**Milestone:** M002
**Written:** 2026-03-23

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: S01 is a pure Rust library slice with no runtime, UI, or server components. All deliverables are verified through unit tests and compilation checks. No browser, Canvas, or user interaction is involved.

## Preconditions

- Rust toolchain installed with `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- Working directory is the project root (or worktree root)
- `cargo` is available and can resolve crate dependencies

## Smoke Test

Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml` and confirm 114 tests pass with 0 failures. This exercises all existing grid/connector/export tests plus the 9 new shape and masking tests.

## Test Cases

### 1. Heart path produces valid closed BezPath

1. Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_heart_path_is_closed`
2. **Expected:** Test passes — heart path ends with `ClosePath` element

### 2. Heart path fits within specified dimensions

1. Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_heart_path_bounding_box`
2. **Expected:** Test passes — bounding box of heart_path(100, 80) fits within (100, 80) ± 2px tolerance

### 3. Star path produces valid closed BezPath

1. Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_star_path_is_closed`
2. **Expected:** Test passes — star path ends with `ClosePath` element

### 4. Star path fits within specified dimensions

1. Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_star_path_bounding_box`
2. **Expected:** Test passes — bounding box of star_path(100, 100, 5) fits within (100, 100) ± 1px tolerance

### 5. Star has correct vertex count

1. Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_star_path_point_count`
2. **Expected:** Test passes — 5-pointed star has 9 `LineTo` segments (1 `MoveTo` + 9 `LineTo` + 1 `ClosePath` = 10 vertices for 5 outer + 5 inner)

### 6. Boolean intersection of overlapping shapes produces non-empty result

1. Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_intersection_heart_and_rect`
2. **Expected:** Test passes — intersection of a heart inside a 200×200 rectangle produces a non-empty path with a bounding box smaller than the rectangle

### 7. Boolean difference preserves outer rectangle bounding box

1. Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_difference_rect_minus_star`
2. **Expected:** Test passes — difference of 200×200 rectangle minus an interior star has bounding box matching the rectangle (within 1px tolerance)

### 8. Boolean ops are deterministic

1. Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_intersection_deterministic`
2. **Expected:** Test passes — calling `mask_intersection` twice with identical inputs produces identical SVG string output

### 9. Non-overlapping shapes produce empty intersection

1. Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- test_no_overlap_intersection_empty`
2. **Expected:** Test passes — intersection of two rectangles 500px apart produces an empty BezPath (0 elements)

### 10. WASM compilation succeeds for full dependency tree

1. Run `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown`
2. **Expected:** Exit code 0, no errors or warnings. This proves linesweeper and all transitive dependencies compile to wasm32-unknown-unknown.

### 11. All existing tests still pass (no regressions)

1. Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml`
2. **Expected:** 114 tests pass, 0 failed, 0 ignored. The 105 pre-existing tests for grid, connector, edge, config, seed, binary_export, and svg_export are unaffected by the new shapes and masking modules.

## Edge Cases

### Heart at extreme aspect ratios

1. In a Rust test, call `heart_path(10.0, 200.0)` (very narrow) and `heart_path(200.0, 10.0)` (very wide)
2. Verify paths are still closed (end with `ClosePath`) and bounding boxes don't exceed specified dimensions by more than 2px
3. **Expected:** Paths are geometrically valid but may look visually odd — this is acceptable for v1

### Star with minimum points (2)

1. In a Rust test, call `star_path(100.0, 100.0, 2)`
2. **Expected:** Produces a closed path with 4 vertices (2 outer + 2 inner) — essentially a diamond shape. No panic.

### Star with many points (20+)

1. In a Rust test, call `star_path(100.0, 100.0, 20)`
2. **Expected:** Produces a closed path with 40 vertices. Bounding box fits within (100, 100).

### Zero-size shapes in boolean ops

1. Call `mask_intersection` with `heart_path(0.0, 0.0)` as one input
2. **Expected:** Either returns an empty path or returns an error — should not panic

### Very small overlapping region

1. Create two rectangles that overlap by just 1px: `rect(0,0,100,100)` and `rect(99,0,100,100)`
2. Call `mask_intersection` on them
3. **Expected:** Returns a non-empty result (a 1×100 sliver) or an empty result — should not panic or produce an error

## Failure Signals

- Any test in `cargo test` reports FAILED — indicates a regression or incorrect implementation
- `cargo check --target wasm32-unknown-unknown` exits non-zero — indicates a dependency uses APIs not available in WASM (e.g. filesystem, threading, getrandom)
- Test count drops below 114 — indicates a test was accidentally removed or a module wasn't wired correctly
- `heart_path` or `star_path` produce paths where the last element is not `ClosePath` — linesweeper will produce incorrect boolean op results on open paths

## Not Proven By This UAT

- Visual quality of heart/star shapes (requires rendering to SVG/Canvas and human inspection)
- Boolean op correctness on complex curved intersections (only tested with simple heart-in-rectangle and star-in-rectangle cases)
- Performance of boolean ops at scale (no benchmarking — addressed in S04 when interactive performance matters)
- Runtime WASM execution (only `cargo check`, not `cargo build` + actual WASM instantiation — addressed in S02)
- Integration with grid clipping (shapes and masking are standalone; grid consumption is S02's scope)

## Notes for Tester

- All tests are deterministic — no random seeds or timing-dependent behavior. Re-running should always produce the same result.
- The `-- shapes` and `-- masking` test name filters are useful for running only the new tests quickly.
- If you want to visually inspect a shape, add a test that prints `heart_path(100.0, 100.0).to_svg()` — paste the output into an SVG `<path d="..."/>` element to view in a browser.
- linesweeper 0.3.0 is a beta crate. If `cargo update` bumps it, re-run all masking tests to verify compatibility.
