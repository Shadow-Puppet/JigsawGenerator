---
phase: quick-005
plan: 5
subsystem: puzzle-core, puzzle-wasm, web-gui
tags: [per-edge-randomization, dual-range-slider, tab-size, taper, UI]
dependency_graph:
  requires: []
  provides: [per-edge-tab-randomization, dual-range-slider-ui, randomize-url-params]
  affects: [config.rs, grid.rs, lib.rs, index.html, main.ts, style.css]
tech_stack:
  added: []
  patterns: [optional-serde-fields, rng-range-per-edge, dual-range-slider-overlay]
key_files:
  created: []
  modified:
    - crates/puzzle-core/src/config.rs
    - crates/puzzle-core/src/grid.rs
    - crates/puzzle-wasm/src/lib.rs
    - web/index.html
    - web/src/main.ts
    - web/src/style.css
decisions:
  - "Optional size_pct_max/taper_max fields with serde skip_serializing_if for backward compat"
  - "randomize_tab_size() and randomize_neck_ratio() consume zero RNG when ranges are None"
  - "Dual-range slider overlay with CSS pointer-events: none on container, all on thumbs"
  - "URL params tabr/tabmax/taperr/tapermax for randomize state (only serialized when active)"
metrics:
  duration: 5 min
  completed: "2026-03-04T18:31:00Z"
---

# Quick Task 5: Add Randomize-Per-Edge Option to Tab Size and Taper

Per-edge tab size and taper randomization with dual-range slider UI, seeded RNG, and URL sharing.

## One-Liner

Per-edge randomization for tab size/taper via optional range fields in TabConfig with dual-thumb range sliders in UI.

## What Was Done

### Task 1: Rust Core - TabConfig Range Fields and Per-Edge Randomization
- Added `size_pct_max: Option<f64>` and `taper_max: Option<f64>` to `TabConfig` with `#[serde(default, skip_serializing_if)]`
- Added `randomize_tab_size(safe_max, rng)` — returns fixed value when None (zero RNG consumption), random value in [min, max] when Some
- Added `randomize_neck_ratio(rng)` — same pattern for taper/neck_ratio
- Updated `validate()` to check range bounds (0.15..=0.25, 0.50..=1.20) and max >= min
- Updated `generate_connectors()` to call per-edge randomize helpers instead of pre-computed fixed values
- Updated WASM `safe_tab_max()` to clamp optional max fields
- Added 8 new tests (range returns, fixed returns, RNG non-consumption, validation errors)
- **Backward compatible**: When ranges are None, zero RNG values consumed = identical output to before

### Task 2: Frontend - Dual-Range Slider UI with Checkbox Toggles
- Added dice icon checkbox toggles next to Tab Size and Taper slider labels
- When enabled, second range thumb appears overlaid on same track (CSS position: absolute)
- Readouts switch to "min%-max%" format (e.g., "15%-25%") when in range mode
- `buildConfig()` conditionally includes `size_pct_max` / `taper_max` fields
- Min/max sliders mutually constrain each other to maintain min <= max invariant
- `updateTabMax()` also applies safe max to the max slider
- URL params: `tabr=1`, `tabmax=25`, `taperr=1`, `tapermax=80` (only when active)
- `loadFromURL()` restores randomize checkbox state and max slider values
- CSS: hidden checkbox, toggle-icon color transition, range-slider-container overlay

## Deviations from Plan

None - plan executed exactly as written.

## Commits

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Rust core per-edge randomization | `9523163` | config.rs, grid.rs, lib.rs |
| 2 | Dual-range slider UI | `2f589ef` | index.html, main.ts, style.css |

## Verification

- `cargo test --workspace` — 119 tests pass (104 core + 15 wasm)
- `npm run build` in web/ — no TypeScript errors
- WASM built successfully via wasm-pack

## Self-Check: PASSED

All 6 modified files exist. Both task commits (9523163, 2f589ef) verified in git log.
