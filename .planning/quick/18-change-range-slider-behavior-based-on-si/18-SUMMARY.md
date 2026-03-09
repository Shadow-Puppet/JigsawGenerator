---
phase: quick-18
plan: 01
subsystem: web-gui
tags: [ui, slider, range-mode]
dependency_graph:
  requires: []
  provides: [center-aware-range-toggle]
  affects: [toggleRandomize, tab-slider, taper-slider]
tech_stack:
  added: []
  patterns: [center-aware-knob-placement]
key_files:
  modified:
    - web/src/main.ts
decisions:
  - Used minSlider.min/max (not maxSlider) for slider bounds since minSlider.max is dynamically clamped by updateTabMax()
metrics:
  duration: 1 min
  completed: "2026-03-09T14:01:15Z"
---

# Quick Task 18: Change Range Slider Behavior Based on Single Knob Position

Center-aware range toggle: when switching to dual-knob mode, single knob position determines which end it anchors (left-of-center stays min with max at maximum; right-of-center becomes max with min at minimum).

## What Changed

### Task 1: Update toggleRandomize with center-aware knob placement
**Commit:** `4e1065f`

Replaced the `if (checkbox.checked)` branch in `toggleRandomize()` with center-aware logic:

1. Computes the midpoint of the slider range: `(sliderMin + sliderMax) / 2`
2. If current value < midpoint (left half):
   - Value stays as the min/left knob (unchanged)
   - Max/right knob jumps to slider maximum
3. If current value >= midpoint (right half or center):
   - Current value becomes the max/right knob
   - Min/left knob jumps to slider minimum

This replaces the old logic that only ensured max > min by one step. The new logic always produces a valid range by design since one end is pinned to the slider's boundary.

Both tab size and taper sliders share this function via their existing event listeners.

## Deviations from Plan

None - plan executed exactly as written.

## Verification

- `npm run build` passes with no errors
- TypeScript compiles cleanly
- Tab size slider: left-of-center values anchor left knob, right-of-center values anchor right knob
- Taper slider: same center-aware behavior
- updateReadouts() and scheduleGenerate() called after placement (unchanged)
- Range highlight gradient updates correctly via updateRangeHighlight()

## Self-Check: PASSED

- [x] web/src/main.ts exists
- [x] Commit 4e1065f found in git log
