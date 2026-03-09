---
phase: quick-019
plan: 1
subsystem: puzzle-core, web-ui
tags: [tab-offset, slider, randomize, url-sync]
dependency_graph:
  requires: []
  provides: [tab-offset-parameter, per-edge-offset-randomization]
  affects: [config, edge-params, classic-connector, grid, ui, url-sharing]
tech_stack:
  added: []
  patterns: [offset-shifted-center, randomize-per-edge]
key_files:
  created: []
  modified:
    - crates/puzzle-core/src/config.rs
    - crates/puzzle-core/src/edge.rs
    - crates/puzzle-core/src/classic_connector.rs
    - crates/puzzle-core/src/grid.rs
    - crates/puzzle-core/src/connector.rs
    - web/index.html
    - web/src/main.ts
decisions:
  - "Offset range -0.15..=0.15 as fraction of edge length (shifts knob up to 15% from center)"
  - "Offset applied to center calculation: center = length * (0.5 + offset), preserving backward compat at 0"
  - "RNG consumption order: tab_size, neck_ratio, offset — offset last to preserve backward compat when None"
  - "URL params: off= (integer percentage), offr=1 for randomize, offmax= for max range"
metrics:
  duration: 5 min
  completed: 2026-03-09
  tasks_completed: 2
  tasks_total: 2
---

# Quick Task 19: Add Tab Offset Slider Summary

**One-liner:** Tab offset slider (-0.15 to 0.15) shifts knob position along each edge with per-edge randomization and URL sync.

## What Was Done

### Task 1: Rust config, edge params, and connector generator (b9ab053)
- Added `offset: f64` and `offset_max: Option<f64>` fields to `TabConfig` with `#[serde(default)]` for backward compatibility
- Added validation: offset must be in -0.15..=0.15, offset_max must be >= offset
- Added `randomize_offset()` method: returns fixed offset when `offset_max` is None (zero RNG consumption), random value in range when Some
- Added `offset: f64` field to `EdgeParams` struct
- Changed center calculation in `ClassicKnobConnector::generate()` from `length * 0.5` to `length * (0.5 + params.offset)`
- Added `randomize_offset()` call in `grid.rs generate_connectors()` after tab_size and neck_ratio (preserves RNG order)
- Updated all `EdgeParams` and `TabConfig` struct literals in tests (connector.rs, classic_connector.rs, config.rs)

### Task 2: UI slider with randomize toggle and URL sync (a1be0a9)
- Added Tab Offset slider group in HTML after Taper group with range -0.15 to 0.15, step 0.01, default 0
- Added randomize toggle checkbox with dice icon, matching existing taper/tab patterns
- Wired offset into `buildConfig()` — sends `tab.offset` and optionally `tab.offset_max` to WASM
- Added `loadFromURL()` restore for `off=`, `offr=`, `offmax=` URL params (integer percentages)
- Added `updateURL()` to persist offset state in URL params
- Added offset readout update in `updateReadouts()` showing fixed or range values
- Added range highlight for offset slider track
- Added offset slider to event wiring: input handlers, max slider clamping, randomize toggle

## Deviations from Plan

None - plan executed exactly as written.

## Verification

- `cargo test --workspace` — all 118 tests pass (105 puzzle-core + 13 puzzle-wasm)
- `wasm-pack build` — WASM compiles and optimizes successfully
- `npx tsc --noEmit` — TypeScript type checking passes cleanly
- Default offset=0 produces identical output to before (backward compatible)
- All existing test assertions unchanged

## Self-Check: PASSED

All 7 modified files verified present. Both task commits (b9ab053, a1be0a9) verified in git log.
