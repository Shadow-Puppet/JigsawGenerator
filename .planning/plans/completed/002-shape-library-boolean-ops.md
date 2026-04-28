# 002 — Shape library & boolean op foundation

**What:** Add a reusable shape library (`heart`, `star`) and boolean-op wrappers (`mask_intersection`, `mask_difference`) over kurbo `BezPath` using the `linesweeper` 0.3.0 crate.
**Why:** Everything downstream in M002 — custom borders, whimsy holes, sub-puzzles — depends on reliable path-vs-path boolean operations. This slice retires the biggest risk (WASM compatibility) before the dependent work starts.

## What shipped

- `crates/puzzle-core/src/shapes.rs` — `heart_path() -> BezPath`, `star_path() -> BezPath`. Both produce closed outlines. Star uses `N` outer + `N` inner vertices (2N total) as `MoveTo + (2N-1) LineTo + ClosePath`.
- `crates/puzzle-core/src/masking.rs` — `mask_intersection(a, b)` and `mask_difference(a, b)` wrappers around `linesweeper::binary_op`. Both handle the `Contours` → `BezPath` conversion: iterate contours, concat their `PathEl` streams into a single `BezPath`.
- linesweeper 0.3.0 added as a `puzzle-core` dependency. Verified to compile cleanly to `wasm32-unknown-unknown` with no feature-flag changes.
- 114 tests passing at slice close (out of which ~16 were new here).

## Patterns established

- **All paths to linesweeper must be closed.** Open `BezPath`s cause silent geometric garbage — no error, just wrong output. Shape constructors always emit `close_path()`; any manually built paths need the same treatment.
- Masking module absorbs the `Contours → BezPath` ergonomic mismatch. Callers get a single `BezPath` back; only reach for `binary_op` directly if you genuinely need per-contour access.
