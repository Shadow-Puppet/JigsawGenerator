---
id: T01
parent: S03
milestone: M002
provides:
  - piece_count field in generate_edges_binary() WASM response for both rectangular and boundary puzzles
key_files:
  - crates/puzzle-wasm/src/lib.rs
key_decisions:
  - "Used bp.included_cells().len() instead of plan's included_cell_count() (method doesn't exist — included_cells() returns Vec)"
  - "Dropped generate_edges_binary native test — JsValue/Reflect panics on non-wasm targets; verified correctness via generate_grid JSON tests + WASM compile check"
patterns_established:
  - "generate_edges_binary cannot be unit-tested in native mode; use generate_grid JSON tests for logic + cargo check --target wasm32-unknown-unknown for WASM compilation"
observability_surfaces:
  - "result.piece_count in browser console after WASM call shows actual piece count"
  - "Compare piece_count vs rows*cols to detect silent boundary filtering failures"
duration: 10m
verification_result: passed
completed_at: 2026-03-24
blocker_discovered: false
---

# T01: Add piece_count to WASM generate_edges_binary response

**Added piece_count field to generate_edges_binary() WASM response — returns actual included cell count for boundary puzzles and rows*cols for rectangular**

## What Happened

Modified `generate_edges_binary()` in `crates/puzzle-wasm/src/lib.rs` to include a `piece_count` field in the returned JS object. The change captures piece count in both code paths:

- **Boundary branch** (heart/star shapes): calls `bp.included_cells().len()` on the `BoundaryPuzzle` after construction, before SVG/binary export. This returns the number of grid cells inside the boundary shape (fewer than `rows * cols`).
- **Rectangular branch** (no border_shape): computes `rows * cols` from the grid config.

The `piece_count` is added via `Reflect::set` after the existing `edges`, `border`, `width`, `height` properties.

Added 4 new tests verifying piece count correctness through the `generate_grid` JSON endpoint (which exercises the same logic). Direct testing of `generate_edges_binary` is not possible in native mode because `JsValue`/`Reflect` operations panic on non-wasm targets — this is a known `wasm-bindgen` limitation. WASM compilation is verified separately via `cargo check --target wasm32-unknown-unknown`.

## Verification

- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml --lib`: **26 passed, 0 failed** (22 existing + 4 new)
- `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown`: **compiles clean**
- `grep -q 'piece_count' crates/puzzle-wasm/src/lib.rs`: **found**
- `cargo test -- test_border_shape_invalid_returns_error`: **1 passed** (error path test)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml --lib` | 0 | ✅ pass | 0.02s |
| 2 | `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` | 0 | ✅ pass | 0.09s |
| 3 | `grep -q 'piece_count' crates/puzzle-wasm/src/lib.rs` | 0 | ✅ pass | <0.01s |
| 4 | `cargo test -- test_border_shape_invalid_returns_error` | 0 | ✅ pass | 0.01s |

## Diagnostics

- **Inspect piece_count:** In the browser console after a WASM call, `result.piece_count` shows the actual piece count as f64.
- **Boundary detection:** If `piece_count == rows * cols` when a non-rectangular border is selected, the boundary computation failed silently. The frontend (T02) should check for this.
- **Cross-verification:** `generate_grid()` JSON endpoint's `piece_breakdown.total` uses the same filtered-pieces logic and can be compared with `piece_count`.
- **Error path:** Invalid `border_shape` values return `{ error: "Unknown border shape: '...'" }` — no `piece_count` in error responses.

## Deviations

- **Method name:** Task plan referenced `BoundaryPuzzle::included_cell_count()` which doesn't exist. Used `bp.included_cells().len()` instead — same result, just calls the actual API.
- **Native test removed:** Task plan suggested testing `generate_edges_binary` via `Reflect::get` in native tests, but `JsValue` operations panic on non-wasm targets. Replaced with 4 tests via `generate_grid` JSON endpoint (exercises identical logic) plus WASM compile check.

## Known Issues

None.

## Files Created/Modified

- `crates/puzzle-wasm/src/lib.rs` — Added `piece_count` field to `generate_edges_binary()` return object (both boundary and rectangular paths) and 4 new piece count tests
- `.gsd/milestones/M002/slices/S03/tasks/T01-PLAN.md` — Added Observability Impact section (pre-flight fix)
