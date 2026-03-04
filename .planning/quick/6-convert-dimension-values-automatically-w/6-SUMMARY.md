---
phase: quick
plan: 6
subsystem: web-gui
tags: [unit-conversion, ux, dimensions]
dependency_graph:
  requires: []
  provides: [automatic-unit-conversion]
  affects: [width-height-inputs, url-params]
tech_stack:
  added: []
  patterns: [previousUnit-tracking, factor-based-conversion]
key_files:
  modified:
    - web/src/main.ts
decisions:
  - Conversion factor 25.4 (1 inch = 25.4mm) applied directly in TypeScript
  - Round to 2 decimal places then parseFloat to strip trailing zeros for clean display
  - previousUnit tracked in closure scope after loadFromURL and before event wiring
metrics:
  duration: 1 min
  completed: "2026-03-04T19:00:27Z"
---

# Quick Task 6: Auto-Convert Dimension Values on Unit Change — Summary

**One-liner:** Width/height auto-convert using 25.4 factor when toggling mm/inches, preserving physical puzzle size.

## What Was Done

### Task 1: Add unit conversion to unit dropdown change handler
**Commit:** `c1a459d`

Added `convertDimensions(oldUnit, newUnit)` helper function that:
- Detects direction: mm->inches uses factor `1/25.4`, inches->mm uses `25.4`
- Converts both width and height input values
- Rounds to 2 decimal places with `parseFloat(toFixed(2))` to avoid floating-point tails while keeping clean display (e.g., "297" not "297.00")
- Early-returns if old and new units are the same

Modified the `unitSelect` change handler to:
- Track `previousUnit` variable (initialized after `loadFromURL()` so URL params are respected)
- Call `convertDimensions(previousUnit, newUnit)` before regeneration
- Update `previousUnit` after conversion

**Key behavior:**
- 297x210 mm -> ~11.69x8.27 inches
- 11.69x8.27 inches -> ~297x210 mm (round-trip preserved within rounding tolerance)
- Corner radius and kerf width are NOT converted (always in mm per Rust WASM interface)
- URL params update with converted values via existing `updateURL()` in `generatePuzzle()`

## Verification

| Check | Result |
|-------|--------|
| `npx vite build` | Pass — production build succeeds |
| TypeScript compilation | Pre-existing `puzzle-wasm` module resolution warning (Vite handles WASM separately) |
| Conversion math | 297/25.4 = 11.69, 210/25.4 = 8.27, reverse: 11.69*25.4 = 296.93 (~297) |

## Deviations from Plan

None — plan executed exactly as written.

## Files Modified

| File | Changes |
|------|---------|
| `web/src/main.ts` | +22 lines: `convertDimensions()` function, `previousUnit` tracking, updated `unitSelect` handler |

## Self-Check: PASSED
