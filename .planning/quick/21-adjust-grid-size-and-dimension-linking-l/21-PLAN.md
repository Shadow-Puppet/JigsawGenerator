---
phase: quick-21
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - web/src/main.ts
autonomous: true
requirements: [QUICK-21]
must_haves:
  truths:
    - "Individual piece aspect ratios exceeding 1:3 trigger auto-adjustment or warning"
    - "When grid is unlocked and pieces are too elongated, rows/cols auto-adjust to bring piece aspect ratio within 1:3"
    - "When dimensions are unlocked and pieces are too elongated, width/height auto-adjust to bring piece aspect ratio within 1:3"
    - "When both are locked, a warning is shown about elongated pieces"
    - "Existing 1:5 grid ratio and <10mm piece size checks still work"
    - "calcBestGrid also considers piece aspect ratio when selecting best grid"
  artifacts:
    - path: "web/src/main.ts"
      provides: "Piece aspect ratio enforcement in enforceConstraints and calcBestGrid"
  key_links:
    - from: "enforceConstraints()"
      to: "piece aspect ratio check"
      via: "pieceW/pieceH ratio compared against 3.0 threshold"
    - from: "calcBestGrid()"
      to: "piece aspect ratio filter"
      via: "skip grids where piece aspect ratio exceeds 3:1"
---

<objective>
Add piece aspect ratio checking to the constraint enforcement system. Currently, the app checks grid ratio (1:5 max) and piece minimum size (10mm), but doesn't check if individual pieces are elongated beyond 1:3. Very elongated pieces produce ugly connectors and are impractical for physical puzzles.

Purpose: Prevent generating puzzles with extremely elongated pieces that look bad and don't work well when laser-cut.
Output: Updated `web/src/main.ts` with piece aspect ratio enforcement.
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@web/src/main.ts

Key existing patterns:
- `enforceConstraints(source: "grid" | "dims")` handles all constraint checking and auto-adjustment
- Grid lock (`gridLocked`) and dims lock (`dimsLocked`) control whether sections auto-adjust or show warnings
- `calcBestGrid(target)` iterates candidate grids and picks the best by distance-to-target + squarest-piece tiebreaker
- `showWarnings(warnings: string[])` displays warning list items in `pieceSizeWarning` element
- Piece dimensions: `pieceW = widthMM / cols`, `pieceH = heightMM / rows`
- When auto-adjusting grid (source="dims"), code reduces rows/cols; when adjusting dims (source="grid"), code scales up width/height
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add piece aspect ratio enforcement to enforceConstraints and calcBestGrid</name>
  <files>web/src/main.ts</files>
  <action>
In `enforceConstraints()`, after the existing piece size check and grid ratio check in BOTH the `source === "grid"` and `source === "dims"` branches, add a piece aspect ratio check. The max allowed piece aspect ratio is 3:1.

**For `source === "grid"` branch** (user changed grid, adjust dimensions if unlocked):
After the existing minDim/grid-ratio logic, add:
```
const pieceAspect = Math.max(pieceW / pieceH, pieceH / pieceW);
if (pieceAspect > 3) {
  if (dimsLocked) {
    warnings.push(`Pieces are very elongated (${pieceAspect.toFixed(1)}:1). Unlock dimensions to auto-adjust.`);
  } else {
    // Scale the shorter dimension up so aspect ratio = 3:1
    // If pieceW > pieceH*3, increase height; if pieceH > pieceW*3, increase width
    if (pieceW > pieceH * 3) {
      // pieces too wide — increase total height so pieceH = pieceW/3
      const needH = (widthMM / cols) / 3 * rows;
      const newHMM = Math.max(heightMM, needH);
      const newH = newHMM / factor;
      heightInput.value = unitSelect.value === "Inches"
        ? parseFloat(newH.toFixed(2)).toString()
        : String(Math.round(newH));
    } else {
      // pieces too tall — increase total width so pieceW = pieceH/3
      const needW = (heightMM / rows) / 3 * cols;
      const newWMM = Math.max(widthMM, needW);
      const newW = newWMM / factor;
      widthInput.value = unitSelect.value === "Inches"
        ? parseFloat(newW.toFixed(2)).toString()
        : String(Math.round(newW));
    }
    adjusted = true;
  }
}
```

Note: Recalculate pieceW/pieceH from the *current* input values (which may have been adjusted by the minDim check above) before doing the aspect ratio check. Read `w`, `h` fresh from the inputs if `adjusted` is already true.

**For `source === "dims"` branch** (user changed dimensions, adjust grid if unlocked):
After the existing minDim/grid-ratio logic, add:
```
// Re-read w/h in case they were adjusted
const curW = parseFloat(widthInput.value) * factor;
const curH = parseFloat(heightInput.value) * factor;
const curPieceW = curW / cols;
const curPieceH = curH / rows;
const pieceAspect = Math.max(curPieceW / curPieceH, curPieceH / curPieceW);
if (pieceAspect > 3) {
  if (gridLocked) {
    warnings.push(`Pieces are very elongated (${pieceAspect.toFixed(1)}:1). Unlock grid size to auto-adjust.`);
  } else {
    // Adjust rows/cols to bring piece aspect ratio within 3:1
    // Target: (w/cols) / (h/rows) should be between 1/3 and 3
    // Equivalently: rows/cols should be between (h/w)/3 and (h/w)*3
    const dimRatio = curH / curW; // h/w
    const currentGridRatio = rows / cols;
    if (currentGridRatio < dimRatio / 3) {
      // Too few rows — pieces too wide. Increase rows.
      rows = Math.max(2, Math.ceil(cols * dimRatio / 3));
      rowsInput.value = String(rows);
      adjusted = true;
    } else if (currentGridRatio > dimRatio * 3) {
      // Too many rows — pieces too tall. Increase cols.
      cols = Math.max(2, Math.ceil(rows / (dimRatio * 3)));
      colsInput.value = String(cols);
      adjusted = true;
    }
    if (adjusted) syncPieceCount();
  }
}
```

**In `calcBestGrid()`:**
Add a piece aspect ratio filter alongside the existing grid ratio filter. After the `if (gridRatio > 5) continue;` line, add:
```
// Skip grids where individual pieces exceed 1:3 aspect ratio
const pAspect = Math.max((w / c) / (h / r), (h / r) / (w / c));
if (pAspect > 3) continue;
```

IMPORTANT: The aspect ratio check in enforceConstraints must happen AFTER the existing minDim and gridRatio adjustments, using re-read values from inputs (since prior adjustments may have changed width/height/rows/cols). Use fresh reads of the input values when `adjusted` is already true before entering the aspect ratio section.
  </action>
  <verify>
    Run `npm run build` in `web/` directory — should compile without errors.
    Manual test: Set a 100x300mm puzzle with 2 rows, 10 cols (pieces would be 30x50mm = 1.67:1, fine).
    Then set 2 rows, 20 cols (pieces would be 15x50mm = 3.3:1, should auto-adjust).
  </verify>
  <done>
    - Piece aspect ratios exceeding 3:1 trigger auto-adjustment when the counterpart section is unlocked
    - Warning shown when both sections are locked and pieces are elongated
    - calcBestGrid skips grids producing pieces with >3:1 aspect ratio
    - Existing minDim and gridRatio checks unaffected
    - TypeScript compiles cleanly
  </done>
</task>

</tasks>

<verification>
- `npm run build` in `web/` succeeds
- Test scenario 1: 297x210mm, set rows=2, cols=30 → with dims unlocked, height should auto-increase to prevent elongated pieces
- Test scenario 2: Lock both grid and dims, manually create elongated config → warning about elongated pieces appears
- Test scenario 3: Use piece count input to target 100 pieces on a 100x300mm board → calcBestGrid should avoid grids that produce >3:1 piece aspect ratios
</verification>

<success_criteria>
- No piece configuration can produce pieces with aspect ratio > 3:1 without showing a warning
- When the counterpart section is unlocked, auto-adjustment prevents elongated pieces
- calcBestGrid naturally avoids elongated piece configurations
- All existing constraint behavior (minDim, gridRatio) unchanged
</success_criteria>

<output>
After completion, create `.planning/quick/21-adjust-grid-size-and-dimension-linking-l/21-SUMMARY.md`
</output>
