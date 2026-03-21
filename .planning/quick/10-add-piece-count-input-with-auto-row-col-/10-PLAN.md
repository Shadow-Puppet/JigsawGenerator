---
phase: quick-10
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - web/index.html
  - web/src/main.ts
  - web/src/style.css
autonomous: true
requirements: [PIECE-COUNT-INPUT, MIN-PIECE-SIZE-WARNING]
must_haves:
  truths:
    - "User can enter a target piece count and rows/cols auto-calculate to best fit"
    - "When rows or cols change manually, the piece count input updates to rows*cols"
    - "A warning appears when piece dimensions fall below 10mm threshold"
    - "Warning is visible but non-blocking — puzzle still generates"
  artifacts:
    - path: "web/index.html"
      provides: "Piece count input field and warning element in Grid Size section"
      contains: "piece-target"
    - path: "web/src/main.ts"
      provides: "Auto-calc logic, piece size warning logic, bidirectional sync"
      contains: "calcBestGrid"
    - path: "web/src/style.css"
      provides: "Warning styling"
      contains: "piece-size-warning"
  key_links:
    - from: "piece-target input"
      to: "rows/cols inputs"
      via: "calcBestGrid() on input event"
      pattern: "calcBestGrid"
    - from: "rows/cols inputs"
      to: "piece-target input"
      via: "syncPieceCount() updates value"
      pattern: "syncPieceCount"
    - from: "rows/cols/width/height/unit inputs"
      to: "warning element"
      via: "checkPieceSize() on any change"
      pattern: "checkPieceSize"
---

<objective>
Add a piece count input that auto-calculates rows/cols, and a minimum piece size warning.

Purpose: Users think in "I want ~100 pieces" not "I want 8 rows and 12 columns." This makes the UI more intuitive. The piece size warning prevents users from creating impractically tiny pieces.

Output: Updated HTML with new inputs, TypeScript with calculation logic, CSS with warning styles.
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@web/index.html
@web/src/main.ts
@web/src/style.css

<interfaces>
<!-- Existing patterns the executor needs to follow -->

From web/src/main.ts — buildConfig pattern:
```typescript
function buildConfig(): object {
  return {
    rows: parseInt(rowsInput.value, 10),
    cols: parseInt(colsInput.value, 10),
    width: parseFloat(widthInput.value),
    height: parseFloat(heightInput.value),
    unit: unitSelect.value,
    // ...
  };
}
```

From web/src/main.ts — event wiring pattern:
```typescript
const numberInputs = [rowsInput, colsInput, widthInput, heightInput];
for (const input of numberInputs) {
  input.addEventListener("input", () => {
    updateTabMax();
    updateReadouts();
    generatePuzzle();
  });
}
```

From web/src/main.ts — URL param sync:
```typescript
function updateURL(): void {
  // Uses abbreviated param names for compact URLs
  params.set("rows", String(config.rows));
  params.set("cols", String(config.cols));
  // ...
}
```

Units: "Millimeters" or "Inches" in select value. Internal always mm.
Conversion: 1 inch = 25.4 mm.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add piece count input with auto row/col calculation</name>
  <files>web/index.html, web/src/main.ts, web/src/style.css</files>
  <action>
**HTML (web/index.html):**

In the "Grid Size" section, add a piece count input ABOVE the rows/cols row. Structure:

```html
<div class="control-row piece-count-row">
  <div class="input-group">
    <label for="piece-target">Piece Count</label>
    <input type="number" id="piece-target" min="4" max="10000" value="48" />
  </div>
</div>
```

Default value 48 matches the default 6*8 grid.

Below the rows/cols row, add a warning container (initially hidden):

```html
<p id="piece-size-warning" class="piece-size-warning"></p>
```

**TypeScript (web/src/main.ts):**

1. Add DOM reference: `let pieceTargetInput: HTMLInputElement;` and `let pieceSizeWarning: HTMLElement;`
2. Cache in main(): `pieceTargetInput = document.getElementById("piece-target") as HTMLInputElement;` and `pieceSizeWarning = document.getElementById("piece-size-warning")!;`

3. Add `calcBestGrid(target: number)` function:
   - Given the current width and height from the inputs, find the (rows, cols) pair where:
     - `rows * cols` is closest to `target`
     - Both rows >= 2 and cols >= 2
     - Both rows <= 100 and cols <= 100
     - Among ties (same distance from target), prefer the pair where individual piece aspect ratio `(width/cols) / (height/rows)` is closest to 1.0 (squarest pieces)
   - Algorithm: iterate `rows` from 2 to min(target, 100). For each rows, compute `cols = Math.round(target / rows)`, clamp cols to [2, 100]. Compute `total = rows * cols`. Track best (rows, cols) by min distance to target, then by aspect ratio closest to 1.
   - After finding best pair: set `rowsInput.value` and `colsInput.value`, then call `updateTabMax()` and `generatePuzzle()`

4. Add `syncPieceCount()` function:
   - Sets `pieceTargetInput.value = String(parseInt(rowsInput.value) * parseInt(colsInput.value))`
   - Called whenever rows or cols change manually

5. Add `checkPieceSize()` function:
   - Compute piece width in mm: `widthMM / cols` and piece height in mm: `heightMM / rows`
   - Where `widthMM` and `heightMM` are the dimensions converted to mm (if unit is Inches, multiply by 25.4)
   - Take the smaller of piece width and height as `minDim`
   - If `minDim < 10` (mm): show warning text like "Pieces are very small (~{X}mm). May be difficult to cut/handle."
     - Where X is `Math.round(minDim)` or `minDim.toFixed(1)` for values < 10
   - If `minDim >= 10`: hide warning (set textContent to empty string)

6. Wire events:
   - `pieceTargetInput.addEventListener("input", () => { calcBestGrid(parseInt(pieceTargetInput.value)); syncPieceCount(); checkPieceSize(); })`
     - Actually: calcBestGrid already sets rows/cols which triggers generatePuzzle. After calcBestGrid, call syncPieceCount (to ensure piece count shows the actual total, which may differ from target) and checkPieceSize.
   - Modify existing rows/cols input handlers: after the existing `updateTabMax(); updateReadouts(); generatePuzzle();` also call `syncPieceCount(); checkPieceSize();`
   - Also call `checkPieceSize()` from the width/height/unit input handlers (width and height are already in numberInputs; unit change handler also needs it)
   - On initial load (after `loadFromURL()` and before `generatePuzzle()`): call `syncPieceCount()` to set the piece count field to the current rows*cols

7. URL param sync: Add `pc` param for piece count (optional — only useful if user wants to share). Actually, skip URL param for piece count — it's derived from rows*cols, so rows/cols in the URL is sufficient. On load, just syncPieceCount() to populate it.

8. Set a flag `let updatingFromPieceCount = false;` to prevent circular updates:
   - In calcBestGrid: set flag true before updating rows/cols, false after
   - In rows/cols input handler: if flag is true, skip calling syncPieceCount (it will be called by the piece count handler)
   - Actually simpler: just don't wire rows/cols handlers to calcBestGrid. The flow is:
     - User changes piece count → calcBestGrid → updates rows/cols → calls generatePuzzle
     - User changes rows/cols → generatePuzzle + syncPieceCount (just updates the number display)
   - To prevent the rows/cols input event from triggering during programmatic value set from calcBestGrid, DON'T dispatch events. Just set `.value` directly and call updateTabMax/generatePuzzle from within calcBestGrid.

**CSS (web/src/style.css):**

```css
.piece-count-row {
  margin-bottom: 0.5rem;
}

.piece-count-row input[type="number"] {
  width: 100%;
}

.piece-size-warning {
  font-size: 0.75rem;
  color: #e67e22;
  margin-top: 0.35rem;
  line-height: 1.3;
}

.piece-size-warning:empty {
  display: none;
}
```
  </action>
  <verify>
    Run `npm run build` in web/ to verify TypeScript compiles without errors. Then `npm run dev` and manually verify: change piece count to 100, observe rows/cols auto-update; change rows manually, observe piece count updates; set a large grid on small dimensions and see the warning appear.
  </verify>
  <done>
    - Piece count input exists in Grid Size section
    - Changing piece count auto-calculates closest rows/cols with squarest pieces
    - Changing rows or cols updates piece count to show actual total
    - Warning appears when min piece dimension < 10mm (e.g., 50mm wide puzzle with 20 cols = 2.5mm pieces)
    - Warning disappears when pieces are large enough
    - Existing URL param loading populates piece count correctly on page load
    - Build passes with no TypeScript errors
  </done>
</task>

</tasks>

<verification>
```bash
cd web && npm run build
```
Build succeeds with no errors. Manual verification: open dev server, test piece count ↔ rows/cols bidirectional sync, verify warning appears/disappears at threshold.
</verification>

<success_criteria>
- Piece count input shows in Grid Size section, defaults to 48
- Entering "100" in piece count → rows/cols update to ~10x10 (exact depends on aspect ratio)
- Manually changing rows to 5 → piece count updates to 5 * current_cols
- Setting 297x210mm with 30x42 grid → warning about ~7mm pieces
- Setting 297x210mm with 6x8 grid → no warning (pieces ~37x26mm)
- TypeScript build passes cleanly
</success_criteria>

<output>
After completion, create `.planning/quick/10-add-piece-count-input-with-auto-row-col-/10-SUMMARY.md`
</output>
