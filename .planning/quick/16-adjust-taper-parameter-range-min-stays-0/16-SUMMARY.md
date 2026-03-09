---
phase: quick-16
plan: 16
subsystem: config/taper
tags: [taper, range-adjustment, config]
dependency_graph:
  requires: []
  provides: [taper-range-0.57-1.32]
  affects: [puzzle-core-config, wasm-clamping, js-mapping]
tech_stack:
  added: []
  patterns: [linear-interpolation-mapping]
key_files:
  created: []
  modified:
    - crates/puzzle-core/src/config.rs
    - crates/puzzle-wasm/src/lib.rs
    - web/src/main.ts
decisions:
  - Taper range tightened from 0.50..=1.20 to 0.57..=1.32 — removes mildest 10% and extends aggressive end by 10%
  - User slider still 0-1 with formula internal = 0.57 + user * 0.75
metrics:
  duration: 3 min
  completed: 2026-03-09
---

# Quick Task 16: Adjust Taper Parameter Range (Min Stays 0) Summary

**One-liner:** Internal taper range adjusted from 0.50..=1.20 to 0.57..=1.32 with updated JS linear interpolation formula (0.57 + val * 0.75)

## What Was Done

### Task 1: Update Rust config validation, defaults, and WASM clamping
**Commit:** `700bf4b`

- Changed `default_taper()` return value from `0.5` to `0.57`
- Changed `TabConfig::default()` taper field from `0.5` to `0.57`
- Updated `validate()` taper bounds: `0.50..=1.20` → `0.57..=1.32` (both for `taper` and `taper_max`)
- Updated doc comments with new range descriptions and neck ratio examples
- Updated all test cases referencing old taper boundary values
- Updated `safe_tab_max()` WASM clamp calls from `clamp(0.50, 1.20)` → `clamp(0.57, 1.32)`
- All 107 Rust tests pass

### Task 2: Update JS mapping formula and rebuild WASM
**Commit:** `666518c`

- Changed `buildConfig()` taper mapping: `0.5 + val * 0.7` → `0.57 + val * 0.75`
- Changed `taper_max` mapping: same formula update
- Rebuilt WASM binary with updated clamp bounds
- Web project builds cleanly (no TS errors)

## Verification Results

1. `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — 107 tests passed
2. `npm run build` (web) — builds cleanly, no errors
3. Mapping verified: slider 0 → internal 0.57, slider 1 → internal 1.32

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Included stale Cargo.lock in Task 2 commit**
- **Found during:** Task 2
- **Issue:** Cargo.lock had uncommitted changes from quick-015 (js-sys dependency addition)
- **Fix:** Included Cargo.lock in Task 2 commit as it relates to WASM rebuild
- **Files modified:** Cargo.lock

## Success Criteria Verification

| Criterion | Status |
|-----------|--------|
| Internal taper range is 0.57..=1.32 (was 0.50..=1.20) | PASS |
| User-facing slider still shows 0.00-1.00 | PASS |
| Linear mapping: internal = 0.57 + user * 0.75 | PASS |
| All Rust tests pass | PASS (107/107) |
| Web builds cleanly | PASS |
| WASM binary rebuilt | PASS |

## Self-Check: PASSED

- All modified files exist on disk
- All commit hashes verified in git log
- SUMMARY.md created at expected path
