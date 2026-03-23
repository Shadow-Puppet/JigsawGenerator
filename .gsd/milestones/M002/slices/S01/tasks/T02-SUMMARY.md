---
id: T02
parent: S01
milestone: M002
provides:
  - mask_intersection(base, shape) -> Result<BezPath, String> boolean intersection wrapper
  - mask_difference(base, shape) -> Result<BezPath, String> boolean difference wrapper
  - pub mod masking wired into puzzle-core lib.rs
key_files:
  - crates/puzzle-core/src/masking.rs
  - crates/puzzle-core/src/lib.rs
key_decisions:
  - Shared boolean_op() helper avoids code duplication between intersection and difference wrappers
  - Contour concatenation iterates all PathEl variants including QuadTo for completeness even though linesweeper currently only emits MoveTo/LineTo/CurveTo/ClosePath
patterns_established:
  - Boolean op wrappers return Result<BezPath, String> — linesweeper Error mapped via .to_string()
  - Multi-contour results concatenated into a single BezPath with multiple subpaths
  - Test helper rect_path(x, y, w, h) constructs closed rectangle BezPaths for masking tests
observability_surfaces:
  - cargo test -- masking runs 4 masking-specific unit tests
  - Call .to_svg() on any BezPath returned by mask_intersection/mask_difference to inspect geometric output
duration: 5m
verification_result: passed
completed_at: 2026-03-22
blocker_discovered: false
---

# T02: Build masking wrappers with boolean op integration tests

**Added mask_intersection and mask_difference wrappers around linesweeper binary_op with 4 integration tests proving intersection, difference, determinism, and empty-overlap behavior**

## What Happened

Created `crates/puzzle-core/src/masking.rs` with two public functions: `mask_intersection` and `mask_difference`, both thin wrappers around `linesweeper::binary_op` using `FillRule::EvenOdd`. A private `boolean_op` helper handles the shared logic — calling `binary_op`, mapping `linesweeper::Error` to `String` via `.to_string()`, and concatenating all resulting `Contour` paths into a single `BezPath` by iterating each `PathEl` variant. Wired the module into `lib.rs` with `pub mod masking;` and `pub use masking::*;`. Added 4 unit tests: intersection of a heart inside a rectangle (verifies non-empty result with smaller bounding box), difference of a rectangle minus a star (verifies non-empty result with bounding box matching the outer rectangle), determinism (identical inputs produce identical SVG output), and empty-overlap (non-overlapping rectangles produce an empty BezPath). All 114 tests pass and WASM compilation succeeds.

## Verification

- Ran `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — 114 tests passed (110 existing + 4 new masking tests), 0 failed.
- Ran `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — exited 0, no errors.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --manifest-path crates/puzzle-core/Cargo.toml` | 0 | ✅ pass | 5.0s |
| 2 | `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` | 0 | ✅ pass | 3.4s |

## Diagnostics

- Run `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- masking` to exercise only the 4 masking tests.
- Call `.to_svg()` on any `BezPath` returned by `mask_intersection`/`mask_difference` to visually inspect SVG path data.
- `linesweeper::Error` is mapped to human-readable strings: "one of the inputs was infinite", "one of the inputs had a NaN", "one of the inputs had a non-closed path".

## Deviations

None — implementation followed the task plan exactly.

## Known Issues

None.

## Files Created/Modified

- `crates/puzzle-core/src/masking.rs` — new file with mask_intersection, mask_difference, boolean_op helper, and 4 unit tests
- `crates/puzzle-core/src/lib.rs` — added `pub mod masking;` and `pub use masking::*;`
