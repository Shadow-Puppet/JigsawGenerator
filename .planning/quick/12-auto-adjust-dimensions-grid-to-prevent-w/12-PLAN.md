---
phase: quick-012
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - web/index.html
  - web/src/main.ts
  - web/src/style.css
autonomous: true
---

<objective>
Replace passive warnings with active auto-adjustment: when the user changes grid size or dimensions, automatically adjust the OTHER to maintain valid piece sizes (>=10mm) and grid ratios (<=5:1). Add lock/unlock toggle icons for both Grid Size and Dimensions sections — when a section is locked, it won't be auto-adjusted; instead the warning is shown as currently.

Behavior:
- Grid unlocked + Dimensions unlocked (default): changing either auto-adjusts the other
- Grid locked: changing dimensions shows warnings instead of adjusting grid
- Dimensions locked: changing grid shows warnings instead of adjusting dimensions
- Both locked: warnings shown for any violation (current behavior)
- The "source of truth" is whichever the user just changed; the unlocked other adapts

Auto-adjustment rules:
1. **Piece too small (<10mm)**: Scale up the unlocked dimension(s) to make pieces exactly 10mm on the small axis
2. **Grid ratio >5:1**: Reduce the larger grid dimension to bring ratio to 5:1
</objective>

<context>
@web/index.html
@web/src/main.ts
@web/src/style.css
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add lock toggle buttons to HTML and style them</name>
  <files>
    web/index.html
    web/src/style.css
  </files>
  <action>
1. In index.html, add a lock toggle button to the Grid Size section header and the Dimensions section header:
   ```html
   <h2 class="section-header">Grid Size <button type="button" class="lock-toggle" id="grid-lock" title="Lock grid size">&#128275;</button></h2>
   ```
   ```html
   <h2 class="section-header">Dimensions <button type="button" class="lock-toggle" id="dims-lock" title="Lock dimensions">&#128275;</button></h2>
   ```
   Use Unicode lock characters: &#128274; (locked/closed) and &#128275; (unlocked/open).

2. In style.css, add styles for the lock toggle:
   - `.lock-toggle`: inline button, no border, transparent bg, cursor pointer, font-size matching section header
   - `.lock-toggle.locked`: different color to indicate locked state
   - Subtle visual: unlocked = muted gray, locked = accent color
  </action>
  <verify>
    <automated>ls web/index.html web/src/style.css</automated>
  </verify>
  <done>Lock toggle buttons visible in Grid Size and Dimensions section headers with proper styling</done>
</task>

<task type="auto">
  <name>Task 2: Implement auto-adjust logic with lock state</name>
  <files>
    web/src/main.ts
  </files>
  <action>
1. Add DOM references for the lock toggle buttons:
   ```typescript
   let gridLockBtn: HTMLElement;
   let dimsLockBtn: HTMLElement;
   let gridLocked = false;
   let dimsLocked = false;
   ```

2. Add lock toggle event handlers that toggle the locked state and update the button icon/class:
   ```typescript
   function toggleLock(btn: HTMLElement, isLocked: boolean): boolean {
     isLocked = !isLocked;
     btn.innerHTML = isLocked ? '&#128274;' : '&#128275;';
     btn.classList.toggle('locked', isLocked);
     btn.title = isLocked ? 'Unlock ...' : 'Lock ...';
     return isLocked;
   }
   ```

3. Replace `checkPieceSize()` with a new `enforceConstraints(source: 'grid' | 'dims')` function:
   - Compute pieceW, pieceH, minDim, gridRatio as before
   - If source is 'grid' (user changed grid):
     - If `dimsLocked`: show warnings (current behavior)
     - Else: auto-adjust dimensions to satisfy constraints
       - If minDim < 10: scale width/height so the smallest piece dim = 10mm
       - If gridRatio > 5: do nothing to dimensions (grid ratio is a grid problem, not dims)
   - If source is 'dims' (user changed dimensions):
     - If `gridLocked`: show warnings (current behavior)
     - Else: auto-adjust grid to satisfy constraints
       - If minDim < 10: reduce rows/cols proportionally so pieces >= 10mm
       - If gridRatio > 5: clamp the larger of rows/cols to 5x the smaller
   - Clear warnings when auto-adjustment resolves the issue
   - After any auto-adjustment, sync piece count and update tab max

4. Update event listeners:
   - rows/cols/pieceTarget input events call `enforceConstraints('grid')`
   - width/height input events call `enforceConstraints('dims')`
   - Unit change calls `enforceConstraints('dims')` after conversion

5. Wire up lock button click handlers in main().

6. Update `calcBestGrid()` to call `enforceConstraints('grid')` instead of `checkPieceSize()`.
  </action>
  <verify>
    <automated>npx tsc --noEmit --project web/tsconfig.json 2>&1 | head -20 || echo "No tsconfig, skipping type check"</automated>
  </verify>
  <done>Auto-adjustment works: changing grid auto-adjusts dimensions (when unlocked) and vice versa. Lock toggles switch between auto-adjust and warning modes. Piece count synced after adjustments.</done>
</task>

</tasks>

<verification>
1. Visual check: lock icons appear in section headers, toggle between locked/unlocked on click
2. With both unlocked: increase rows to large number → dimensions auto-increase to keep pieces >= 10mm
3. With both unlocked: decrease width to small value → grid cols auto-decrease to keep pieces >= 10mm
4. Lock grid, decrease width → warning appears, grid unchanged
5. Lock dimensions, increase rows → warning appears, dimensions unchanged
6. Piece count input still works correctly with auto-adjustment
</verification>

<success_criteria>
- Lock/unlock toggles visible and functional on Grid Size and Dimensions sections
- Auto-adjustment prevents <10mm piece sizes when the other section is unlocked
- Warnings still appear when the relevant section is locked
- Piece count syncs after auto-adjustments
- No TypeScript compilation errors
</success_criteria>

<output>
After completion, create `.planning/quick/12-auto-adjust-dimensions-grid-to-prevent-w/12-SUMMARY.md`
</output>
