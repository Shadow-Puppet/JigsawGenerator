---
phase: quick
plan: 002
subsystem: taper-config
tags: [ui, config, wasm-bridge]
dependency_graph:
  requires: [quick-001]
  provides: [normalized-taper-slider, internal-taper-mapping]
  affects: [puzzle-generation, url-sharing]
tech_stack:
  added: []
  patterns: [linear-interpolation-slider-mapping]
key_files:
  created: []
  modified:
    - crates/puzzle-core/src/config.rs
    - web/index.html
    - web/src/main.ts
decisions:
  - "Linear interpolation (0.5 + slider * 0.7) chosen for user 0-1 → internal 0.5-1.2 mapping"
  - "URL param stores user-facing slider value (0-100 integer) not internal value"
  - "Default taper remains 0.5 internally (slider at 0), so existing puzzles get mildest taper"
metrics:
  duration: "2 min"
  completed: "2026-03-04T15:28:00Z"
  tasks_completed: 2
  tasks_total: 2
---

# Quick Task 002: Change Taper Range to 0-1 (User) / 0.5-1.2 (Internal) Summary

Normalized taper slider to user-friendly 0-1 range with linear interpolation to internal 0.5-1.2 range for connector generation.

## What Changed

### Task 1: Rust Taper Validation (b3b89eb)

- Updated `TabConfig` validation bounds from `0.30..=1.10` to `0.50..=1.20`
- Updated field documentation to describe new range and UI mapping relationship
- Updated `neck_ratio()` doc comment with accurate range examples (0.50→0.75, 0.85→0.575, 1.20→0.40)
- Updated boundary test values: min `0.30→0.50`, max `1.10→1.20` (both occurrences)
- Formula `1.0 - self.taper * 0.5` unchanged — naturally works with new range

### Task 2: HTML Slider & TypeScript Mapping (31e6755)

- **HTML:** Slider changed from `min="0.30" max="1.10" value="0.5"` to `min="0" max="1" value="0"`
- **HTML:** Default readout changed from `0.50` to `0.00`
- **buildConfig():** Added linear interpolation `taper: 0.5 + parseFloat(taperSlider.value) * 0.7`
- **loadFromURL():** URL taper param now defaults to `"0"` (was `"50"`), clamps to 0-1 user range
- **updateURL():** Stores raw slider value as 0-100 integer via `parseFloat(taperSlider.value) * 100`

## Mapping Reference

| Slider (User) | URL Param | Internal Value | neck_ratio |
|---------------|-----------|----------------|------------|
| 0.00          | 0         | 0.50           | 0.75       |
| 0.50          | 50        | 0.85           | 0.575      |
| 1.00          | 100       | 1.20           | 0.40       |

## Verification Results

- **Rust tests:** 96 passed, 0 failed
- **Vite build:** Successful (165.80 KB WASM, 8.65 KB JS)
- **Backward compat:** Old URLs with taper values outside 0-100 are clamped to valid 0-1 range

## Deviations from Plan

None - plan executed exactly as written.

## Commits

| # | Hash | Message |
|---|------|---------|
| 1 | b3b89eb | feat(quick-002): update Rust taper validation to 0.50..=1.20 range |
| 2 | 31e6755 | feat(quick-002): normalize taper slider to 0-1 with internal 0.5-1.2 mapping |

## Self-Check: PASSED

- All 4 modified/created files exist on disk
- Both commit hashes (b3b89eb, 31e6755) found in git log
- Key link pattern `0.5 + .* * 0.7` found in web/src/main.ts line 54
