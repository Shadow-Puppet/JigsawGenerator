---
phase: quick-13
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - web/src/main.ts
autonomous: true
must_haves:
  truths:
    - "Slider drag only triggers one WASM call per animation frame, not per input event"
    - "Zoom/pan does not re-query the DOM for the SVG path element on every frame"
    - "Piece count breakdown is computed in JS without a WASM roundtrip"
  artifacts:
    - path: "web/src/main.ts"
      provides: "Optimized SVG rendering pipeline"
  key_links:
    - from: "rAF throttle"
      to: "generatePuzzle()"
      via: "requestAnimationFrame guard"
      pattern: "requestAnimationFrame"
    - from: "cachedPath variable"
      to: "applyTransform()"
      via: "cached SVG path reference"
      pattern: "cachedPath"
---

<objective>
Optimize SVG rendering performance for large puzzles by eliminating redundant work in the hot path.

Purpose: Slider dragging fires 60+ input events/sec. Each triggers a full WASM call (JSON serialize, deserialize, grid build, connector generation, SVG serialization) plus a second redundant WASM call for piece count. Zoom/pan also re-queries the DOM every frame. These three fixes eliminate >90% of wasted computation.

Output: Updated `web/src/main.ts` with rAF throttling, cached path reference, and inline piece count math.
</objective>

<execution_context>
@.planning/quick/13-optimize-svg-rendering-performance-for-l/13-PLAN.md
</execution_context>

<context>
@web/src/main.ts
@crates/puzzle-wasm/src/lib.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add rAF throttle, cache SVG path, inline piece count</name>
  <files>web/src/main.ts</files>
  <action>
Three targeted changes in `web/src/main.ts`:

**1. requestAnimationFrame throttle on generatePuzzle()**

Add a module-level `rAF` guard variable near the zoom/pan state block:

```typescript
let rafPending = false;
```

Create a new function `scheduleGenerate()` that coalesces calls:

```typescript
function scheduleGenerate(): void {
  if (rafPending) return;
  rafPending = true;
  requestAnimationFrame(() => {
    rafPending = false;
    generatePuzzle();
  });
}
```

Replace ALL `generatePuzzle()` calls from input event handlers with `scheduleGenerate()`. Specifically:
- Line 667 (rows/cols input handler) → `scheduleGenerate()`
- Line 677 (width/height input handler) → `scheduleGenerate()`
- Line 703 (slider input handler) → `scheduleGenerate()`
- Line 711 (tabMaxSlider input handler) → `scheduleGenerate()`
- Line 715 (taperMaxSlider input handler) → `scheduleGenerate()`
- Line 362 in `toggleRandomize()` → `scheduleGenerate()`
- Line 738 (seed input handler) → `scheduleGenerate`
- Line 744 (randomize button click) → `scheduleGenerate()`

Keep `generatePuzzle()` called DIRECTLY (not scheduled) in these places that need synchronous execution:
- `calcBestGrid()` (line 450) — already called from input handler that will schedule
- Initial generate at end of `main()` (line 941) — needs immediate render on page load
- Unit select change handler (line 733) — single event, not rapid-fire

Actually, `calcBestGrid()` calls `generatePuzzle()` directly and is itself called from the pieceTargetInput handler. Change the pieceTargetInput handler to NOT call generatePuzzle separately — `calcBestGrid` already does. But since calcBestGrid is called from an input handler, change the `generatePuzzle()` inside `calcBestGrid()` to `scheduleGenerate()` as well.

**2. Cache SVG path element reference**

Add a module-level variable near the zoom/pan state:

```typescript
let cachedSvgPath: SVGPathElement | null = null;
```

In `generatePuzzle()`, after the SVG normalization block (after removing width/height attrs from the SVG element), cache the path:

```typescript
cachedSvgPath = svgEl?.querySelector("path") as SVGPathElement | null;
```

In `applyTransform()`, replace the `querySelector` lookup:

```typescript
// BEFORE:
const path = svgContainer.querySelector("svg path") as SVGPathElement | null;
if (path) path.style.strokeWidth = `${0.2 / zoomLevel}px`;

// AFTER:
if (cachedSvgPath) cachedSvgPath.style.strokeWidth = `${0.2 / zoomLevel}px`;
```

**3. Remove compute_pieces() WASM call — compute in JS**

In `generatePuzzle()`, replace the entire `compute_pieces` block (lines 265-274) with inline JS math:

```typescript
// Compute piece breakdown in JS (avoids redundant WASM roundtrip)
const rows = parseInt(rowsInput.value, 10);
const cols = parseInt(colsInput.value, 10);
const total = rows * cols;
const corners = 4;
const edges = 2 * (rows - 2) + 2 * (cols - 2);
const interior = (rows - 2) * (cols - 2);
pieceCount.textContent = `${total} pieces (${corners} corner, ${edges} edge, ${interior} interior)`;
```

Remove the `compute_pieces` import from the top of the file (line 3). The WASM function still exists for backward compat, just no longer called from the UI.

Remove the `PieceBreakdown` interface (lines 9-14) — no longer needed since we don't parse WASM JSON.
  </action>
  <verify>
    Run `npm run build` from `web/` directory — no TypeScript errors, builds successfully.
    Run `npm run dev` from `web/` and verify:
    1. Puzzle renders on page load
    2. Dragging sliders updates smoothly without jank
    3. Piece count shows correct numbers
    4. Zoom/pan works correctly
    5. No console errors
  </verify>
  <done>
    - generatePuzzle() is wrapped in rAF for all input event handlers (no more 60+ WASM calls/sec during slider drag)
    - applyTransform() uses cached SVG path reference (no DOM query per zoom/pan frame)
    - Piece count computed in JS inline (no compute_pieces WASM call)
    - Build succeeds with no TS errors
    - All existing functionality preserved (generate, zoom, pan, download, URL sync)
  </done>
</task>

</tasks>

<verification>
```bash
cd web && npm run build
```
Build must succeed with zero TypeScript errors.

Manual smoke test: open dev server, drag each slider rapidly, verify smooth updates without lag. Zoom in/out, pan around. Download SVG. Verify piece count accuracy for a 6x8 grid (48 pieces, 4 corner, 20 edge, 24 interior).
</verification>

<success_criteria>
- TypeScript compiles without errors
- Slider interactions produce at most 1 WASM call per animation frame
- Zoom/pan operates without DOM queries per frame
- Piece count displays correctly without WASM roundtrip
- All existing features (URL sync, download, seed randomize, unit conversion) still work
</success_criteria>

<output>
After completion, create `.planning/quick/13-optimize-svg-rendering-performance-for-l/13-SUMMARY.md`
</output>
