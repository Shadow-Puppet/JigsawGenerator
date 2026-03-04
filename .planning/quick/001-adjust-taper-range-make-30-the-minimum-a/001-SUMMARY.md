---
phase: quick-001
plan: 01
subsystem: puzzle-core, web-gui
tags: [taper, validation, slider, url-params]
dependency_graph:
  requires: []
  provides: [taper-range-0.30-1.10]
  affects: [config-validation, web-ui, url-sharing]
tech_stack:
  added: []
  patterns: [clamping-for-backwards-compat]
key_files:
  created: []
  modified:
    - crates/puzzle-core/src/config.rs
    - web/index.html
    - web/src/main.ts
decisions:
  - "Clamp old URL taper values to 0.30 minimum rather than rejecting them"
metrics:
  duration: 2 min
  completed: "2026-03-04T05:09:16Z"
---

# Quick Task 001: Adjust Taper Range — Make 0.30 the Minimum

**One-liner:** Taper range narrowed to 0.30..=1.10 across Rust validation, HTML slider, and JS URL clamping for meaningful snap-fit necks on all puzzles.

## What Changed

### Rust Validation (config.rs)
- `TabConfig::validate()` now enforces `0.30..=1.10` instead of `0.0..=1.0`
- Doc comment on `taper` field updated to reflect new range
- `neck_ratio()` formula unchanged — works correctly with new bounds (0.85 at min, 0.45 at max)
- Default taper of 0.5 remains valid and unchanged
- Boundary tests updated to test new min (0.30) and max (1.10) values

### HTML Slider (index.html)
- Taper slider `min` changed from `0` to `0.30`
- Taper slider `max` changed from `1` to `1.10`
- Default value `0.5` and step `0.01` unchanged

### JS URL Handling (main.ts)
- `loadFromURL()` now clamps decoded taper value to `[0.30, 1.10]`
- Old shared URLs with taper=0 get clamped to 0.30 instead of failing Rust validation
- `updateURL()` and `updateReadouts()` unchanged — they already handle arbitrary numeric values

## Commits

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Update Rust taper validation and neck_ratio formula | `94e0137` | `crates/puzzle-core/src/config.rs` |
| 2 | Update HTML slider and JS URL handling | `9e8041d` | `web/index.html`, `web/src/main.ts` |
| 3 | Rebuild WASM and verify end-to-end | *(verification only)* | — |

## Verification Results

- `cargo test --workspace` — 111 tests pass (96 core + 15 wasm)
- `wasm-pack build` — WASM compiles successfully
- `npm run build` — Vite frontend builds without errors
- Built HTML confirms slider has `min="0.30" max="1.10"`
- Rust validation rejects taper=0.0 (below minimum)
- Rust validation accepts taper=0.30 (new minimum) and taper=1.10 (new maximum)

## Deviations from Plan

None — plan executed exactly as written.

## Decisions Made

1. **Clamp old URL values instead of rejecting:** Old shared URLs may have taper=0 (encoded as `taper=0` in URL params). Rather than letting these fail Rust validation, the JS layer clamps to the valid range. This preserves backward compatibility for shared puzzle links.

## Self-Check: PASSED

- All modified files exist on disk
- All commit hashes verified in git log
