---
phase: quick-18
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - web/src/main.ts
autonomous: true
requirements: [QUICK-18]

must_haves:
  truths:
    - "When randomize is toggled ON and single knob value is in the left half (< midpoint), value A becomes the left/min knob and right/max knob is set to slider maximum"
    - "When randomize is toggled ON and single knob value is in the right half (>= midpoint), value A becomes the right/max knob and left/min knob is set to slider minimum"
    - "Both tab size and taper sliders follow this same behavior"
    - "Range highlight and readouts update correctly after the toggle"
  artifacts:
    - path: "web/src/main.ts"
      provides: "Updated toggleRandomize function with center-aware knob placement"
      contains: "midpoint"
  key_links:
    - from: "toggleRandomize()"
      to: "updateReadouts()"
      via: "called after knob placement to refresh display"
      pattern: "updateReadouts"
---

<objective>
Change the range slider toggle behavior so that when switching from single-knob to dual-knob (range) mode, the position of the single knob determines which end it becomes:

- If value A is left of center (< midpoint): A becomes the left/min knob, right/max knob = slider maximum
- If value A is right of center (>= midpoint): A becomes the right/max knob, left/min knob = slider minimum

This applies to both the Tab Size and Taper sliders.

Purpose: More intuitive range selection — the user's current value anchors the nearest end of the range.
Output: Updated `toggleRandomize()` in `web/src/main.ts`
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@web/src/main.ts
@web/index.html
</context>

<interfaces>
<!-- Key function and slider references the executor needs -->

From web/src/main.ts (lines 478-503):
```typescript
function toggleRandomize(
  checkbox: HTMLInputElement,
  maxSlider: HTMLInputElement,
  minSlider: HTMLInputElement,
): void {
  if (checkbox.checked) {
    maxSlider.style.display = "";
    // Ensure max > min (at least one step apart)
    const step = parseFloat(minSlider.step) || 0.01;
    const minVal = parseFloat(minSlider.value);
    const maxVal = parseFloat(maxSlider.value);
    const sliderMax = parseFloat(maxSlider.max);
    if (maxVal <= minVal) {
      if (minVal + step <= sliderMax) {
        maxSlider.value = String(minVal + step);
      } else {
        minSlider.value = String(sliderMax - step);
        maxSlider.value = String(sliderMax);
      }
    }
  } else {
    maxSlider.style.display = "none";
  }
  updateReadouts();
  scheduleGenerate();
}
```

Called from (lines 868-873):
```typescript
tabRandomize.addEventListener("change", () => {
  toggleRandomize(tabRandomize, tabMaxSlider, tabSlider);
});
taperRandomize.addEventListener("change", () => {
  toggleRandomize(taperRandomize, taperMaxSlider, taperSlider);
});
```

Slider HTML attributes:
- Tab: min="0.15" max="0.25" step="0.01" (note: max is dynamically updated by updateTabMax())
- Taper: min="0" max="1" step="0.01"
</interfaces>

<tasks>

<task type="auto">
  <name>Task 1: Update toggleRandomize with center-aware knob placement</name>
  <files>web/src/main.ts</files>
  <action>
Replace the body of the `if (checkbox.checked)` branch in `toggleRandomize()` (lines 483-496) with center-aware logic:

1. Show the max slider: `maxSlider.style.display = ""`
2. Read the current single-knob value (`minSlider.value`), the slider's min (`minSlider.min`), and the slider's max (`minSlider.max`). Note: use `minSlider.max` not `maxSlider.max` since both sliders share the same range and `minSlider.max` is the dynamically-clamped value from `updateTabMax()`.
3. Compute the midpoint: `midpoint = (sliderMin + sliderMax) / 2`
4. If `currentValue < midpoint` (left of center):
   - Keep `minSlider.value` as-is (it's already the left knob)
   - Set `maxSlider.value = String(sliderMax)` (right knob goes to maximum)
5. Else (right of center or exactly at midpoint):
   - Set `maxSlider.value = String(currentValue)` (current value becomes the right/max knob)
   - Set `minSlider.value = String(sliderMin)` (left knob goes to minimum)

This replaces the old logic which only ensured max > min by one step. The new logic always produces a valid range (min <= max) by design since one end is pinned to the slider's min or max.

Do NOT change any other part of the function (the `else` branch for unchecking, or the `updateReadouts()` / `scheduleGenerate()` calls at the end).
  </action>
  <verify>
Run `npm run build` in web/ directory to confirm TypeScript compiles without errors. Then manually verify:
1. Open the app, set tab size slider to ~20% (left of center between 15-25%), toggle randomize ON — left knob stays at 20%, right knob jumps to max (25%)
2. Turn randomize OFF, set tab to ~24% (right of center), toggle ON — left knob jumps to min (15%), right knob stays at 24%
3. Same behavior for taper slider
  </verify>
  <done>
toggleRandomize() uses center-aware placement: left-of-center values anchor the left knob with max on right; right-of-center values anchor the right knob with min on left. Both tab and taper sliders exhibit this behavior. Build passes.
  </done>
</task>

</tasks>

<verification>
- `npm run build` succeeds in web/ directory
- Tab size slider: toggle randomize at various positions confirms center-aware behavior
- Taper slider: toggle randomize at various positions confirms center-aware behavior
- Readouts display correct min-max range after toggle
- Range highlight (blue gradient) correctly spans the new range
</verification>

<success_criteria>
- Single task modifying only `toggleRandomize()` in main.ts
- When randomize toggled ON with value left of center: value stays as min, max set to slider maximum
- When randomize toggled ON with value right of center: value becomes max, min set to slider minimum
- Build compiles, readouts and highlights update correctly
</success_criteria>

<output>
After completion, create `.planning/quick/18-change-range-slider-behavior-based-on-si/18-SUMMARY.md`
</output>
