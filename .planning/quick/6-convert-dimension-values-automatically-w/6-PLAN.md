---
phase: quick
plan: 6
type: execute
wave: 1
depends_on: []
files_modified:
  - web/src/main.ts
autonomous: true
requirements: [QUICK-006]
must_haves:
  truths:
    - "Switching from mm to inches converts width/height values (e.g. 297mm -> ~11.69in)"
    - "Switching from inches to mm converts width/height values (e.g. 11.69in -> ~297mm)"
    - "Round-tripping mm->in->mm preserves values within rounding tolerance"
    - "The puzzle SVG output is identical before and after switching (same physical size)"
  artifacts:
    - path: "web/src/main.ts"
      provides: "Unit conversion on dropdown change"
      contains: "convertDimensions"
  key_links:
    - from: "unitSelect change handler"
      to: "widthInput.value, heightInput.value"
      via: "conversion factor 25.4"
      pattern: "25\\.4"
---

<objective>
Auto-convert width and height input values when the unit dropdown changes between mm and inches.

Purpose: Currently switching units leaves the numeric values unchanged, causing the physical puzzle size to change drastically (e.g., 297 "inches" instead of the intended ~11.69 inches). Users expect the physical size to stay the same.

Output: Updated `main.ts` with conversion logic in the unit change handler.
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@web/src/main.ts
@web/index.html
</context>

<interfaces>
<!-- Existing unit conversion factors from Rust (puzzle-core/src/config.rs): -->
<!-- 1 inch = 25.4 mm -->
<!-- Unit::Inches.to_mm(value) = value * 25.4 -->
<!-- Unit::Inches.from_mm(value_mm) = value_mm / 25.4 -->

<!-- Width and height inputs are in user-selected units (mm or inches) -->
<!-- Corner radius and kerf are always in mm — do NOT convert these -->
<!-- The Rust WASM layer converts width/height from user units to mm internally -->
</interfaces>

<tasks>

<task type="auto">
  <name>Task 1: Add unit conversion to unit dropdown change handler</name>
  <files>web/src/main.ts</files>
  <action>
In `web/src/main.ts`, modify the `unitSelect` change event handler (lines 370-373) to convert width and height input values when the unit changes.

Add a helper function `convertDimensions(oldUnit: string, newUnit: string)` above the event wiring section:

```typescript
function convertDimensions(oldUnit: string, newUnit: string): void {
  if (oldUnit === newUnit) return;
  const factor = newUnit === "Inches" ? 1 / 25.4 : 25.4;
  const w = parseFloat(widthInput.value);
  const h = parseFloat(heightInput.value);
  if (!isNaN(w)) {
    widthInput.value = parseFloat((w * factor).toFixed(2)).toString();
  }
  if (!isNaN(h)) {
    heightInput.value = parseFloat((h * factor).toFixed(2)).toString();
  }
}
```

Key details:
- mm -> inches: divide by 25.4 (factor = 1/25.4)
- inches -> mm: multiply by 25.4 (factor = 25.4)
- Round to 2 decimal places to avoid long floating-point tails, then parseFloat to strip trailing zeros (e.g., "11.69" not "11.6929...")
- Use `parseFloat(...toFixed(2)).toString()` to get clean display (e.g., "297" not "297.00")

Modify the unitSelect change handler to capture the old unit before the DOM update:

```typescript
let previousUnit = unitSelect.value;
unitSelect.addEventListener("change", () => {
  const newUnit = unitSelect.value;
  convertDimensions(previousUnit, newUnit);
  previousUnit = newUnit;
  updateTabMax();
  generatePuzzle();
});
```

The `previousUnit` variable must be declared right before the event listener wiring (inside the `main()` function scope, after `loadFromURL()` and before event wiring begins) so it captures the initial unit from URL params or default.

Do NOT convert corner_radius or kerf_width — those are always in mm regardless of unit selection.
  </action>
  <verify>
    <automated>cd web && npx tsc --noEmit</automated>
  </verify>
  <done>
- Switching mm->inches converts 297x210 to ~11.69x8.27
- Switching inches->mm converts back to ~297x210
- The generated puzzle SVG is identical before and after switching (same physical dimensions, just different unit display)
- URL params update correctly with converted values
- TypeScript compiles without errors
  </done>
</task>

</tasks>

<verification>
1. `cd web && npx tsc --noEmit` — TypeScript compiles
2. `cd web && npx vite build` — Production build succeeds
3. Manual check: Load app, note default 297x210mm, switch to inches, confirm ~11.69x8.27, switch back, confirm ~297x210
</verification>

<success_criteria>
- Unit dropdown change converts width/height values automatically
- Conversion preserves physical puzzle size (SVG output unchanged)
- Round-trip conversion stays within 0.01 tolerance of original values
- No regression: all existing functionality (URL params, download, etc.) unaffected
</success_criteria>

<output>
After completion, create `.planning/quick/6-convert-dimension-values-automatically-w/6-SUMMARY.md`
</output>
