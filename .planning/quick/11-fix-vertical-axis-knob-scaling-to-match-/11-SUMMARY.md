---
phase: quick-011
plan: 01
subsystem: puzzle-core/connectors
tags: [knob-scaling, cross-axis, aspect-ratio, overlap-fix]
dependency_graph:
  requires: []
  provides: [uniform-knob-sizing, cross-length-aware-connectors]
  affects: [classic_connector, edge, grid]
tech_stack:
  added: []
  patterns: [min-dimension-scaling]
key_files:
  created: []
  modified:
    - crates/puzzle-core/src/edge.rs
    - crates/puzzle-core/src/grid.rs
    - crates/puzzle-core/src/classic_connector.rs
    - crates/puzzle-core/src/connector.rs
decisions:
  - "Use min(length, cross_length) as knob base dimension for uniform sizing across axes"
  - "Remove safe_tab_max 0.15 floor to prevent forced overlap at extreme aspect ratios"
  - "Use cross_length for Y-bound validation instead of length"
metrics:
  duration: 3 min
  completed: "2026-03-05T01:57:00Z"
---

# Quick Task 11: Fix Vertical Axis Knob Scaling Summary

Uniform knob sizing via min(length, cross_length) base dimension, preventing overlap at any aspect ratio.

## What Changed

### EdgeParams: cross_length field (edge.rs)
Added `cross_length: f64` to `EdgeParams` — carries the perpendicular cell dimension so connector generators can size knobs relative to both axes, not just the edge's own length.

### Grid connector generation (grid.rs)
- `generate_connectors()` now computes `cell_w` and `cell_h` and passes `cross_length: cell_h` for h-edges and `cross_length: cell_w` for v-edges
- `safe_tab_max()` changed from `(theoretical_max * 0.9).max(0.15).min(0.25)` to `(theoretical_max * 0.9).min(0.25)` — removing the 0.15 floor that forced overlapping knobs at extreme aspect ratios

### ClassicKnobConnector (classic_connector.rs)
- `generate()`: knob base dimension changed from `length` to `length.min(params.cross_length)`, ensuring both h-edge and v-edge knobs are sized by the smaller cell dimension
- `validate()`: Y-axis bounding box check now uses `cross_length` instead of `length`, correctly reflecting that knobs protrude into the perpendicular dimension

### Test updates (connector.rs, classic_connector.rs)
- All `EdgeParams` construction sites updated with `cross_length` field
- New `test_uniform_knob_size_across_axes`: verifies identical knob height on both axes for non-square grids
- New `test_extreme_aspect_ratio_no_overlap`: verifies knobs stay within `cross_length/2` at 5:1 ratio

## Commits

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Add cross_length to EdgeParams and update grid | fed8c15 | edge.rs, grid.rs, connector.rs, classic_connector.rs |
| 2 | Use cross_length for uniform knob scaling | d69e29e | classic_connector.rs |

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- **99 tests pass** (97 existing + 2 new) — `cargo test --manifest-path crates/puzzle-core/Cargo.toml`
- **WASM builds** — `wasm-pack build crates/puzzle-wasm --target web --release`
- Seed determinism preserved (test_generate_connectors_deterministic passes)
- Connector continuity preserved (test_curves_continuous passes)
- Edge coordinate invariants preserved (test_h_edge_coordinates, test_v_edge_coordinates pass)

## Self-Check: PASSED

All 4 modified files exist. Both commits (fed8c15, d69e29e) verified in git log.
