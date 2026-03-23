---
estimated_steps: 10
estimated_files: 3
---

# T02: Build whimsy drag-drop Canvas overlay with resize, URL persistence, and generation wiring

**Slice:** S04 — Whimsy Drag-Drop & Grid Adaptation
**Milestone:** M002

## Description

Build the complete user-facing whimsy interaction in the browser. This is the UI half of S04 — it consumes the WASM endpoints updated in T01 (which now accept whimsy config and produce hole-aware output) and adds: (1) Canvas overlay drawing of the whimsy shape, (2) drag-and-drop placement, (3) resize, (4) real-time WASM regeneration with debouncing, (5) URL persistence, and (6) UI controls.

The interaction model: user selects a whimsy shape from a dropdown in the sidebar. This enters "placement mode" — the cursor changes, and clicking on the Canvas places the whimsy at that position (converted from screen pixels to puzzle mm coordinates). Once placed, the user can drag the whimsy to reposition it and scroll-wheel while hovering to resize it. The grid adapts in real-time via debounced WASM calls.

**Key design constraints:**
- Whimsy position is stored in puzzle mm coordinates, not screen pixels — this makes it zoom/pan independent. The coordinate transform is: `mmX = (screenX - panX) / (baseScale * zoomLevel)`, `mmY = (screenY - panY) / (baseScale * zoomLevel)` where `baseScale = viewportWidth / puzzleWidth`.
- During drag, the whimsy overlay is drawn immediately as a Canvas overlay (no WASM call). WASM regeneration is throttled via the existing `scheduleGenerate()` rAF pattern. On mouseup (drop), `generatePuzzle()` is called immediately for final grid adaptation.
- Only one whimsy at a time (R012/D018). Selecting a new shape replaces the old. The "Remove" button clears the whimsy entirely.
- Follow the existing `buildConfig()` optional field pattern (D025): only include `whimsy_shape`, `whimsy_x`, `whimsy_y`, `whimsy_scale` in the config object when a whimsy is active. Omit them when no whimsy is placed.
- The whimsy overlay should be visually distinct from puzzle edges — use a semi-transparent fill and a colored dashed stroke.
- Guard against drag when `puzzleWidth === 0` (no puzzle generated yet).

**Relevant skills to load:** `frontend-design`, `make-interfaces-feel-better` — for polished interaction feel, cursor states, and visual feedback.

## Steps

1. **Add whimsy UI controls to `web/index.html`:**
   - Add a "Whimsy Shape" `<section>` below the "Border Shape" section
   - Include a `<select id="whimsy-shape">` dropdown with options: None (default), Heart, Star
   - Include a `<button id="remove-whimsy">Remove Whimsy</button>` (hidden when no whimsy is placed)
   - Include a `<span id="whimsy-info">` for displaying current whimsy position/scale info

2. **Add whimsy state variables in `web/src/main.ts`:**
   - `let whimsyShape: string = ""` — active whimsy shape name ("heart", "star", or "")
   - `let whimsyX: number = 0` — whimsy center X in puzzle mm
   - `let whimsyY: number = 0` — whimsy center Y in puzzle mm
   - `let whimsyScale: number = 1.0` — whimsy scale factor
   - `let isPlacingWhimsy: boolean = false` — in placement mode (shape selected, not yet placed)
   - `let isDraggingWhimsy: boolean = false` — actively dragging the whimsy
   - `let dragOffsetX: number = 0`, `let dragOffsetY: number = 0` — offset from whimsy center to drag start point
   - DOM references: `whimsyShapeSelect`, `removeWhimsyBtn`, `whimsyInfo`

3. **Implement screen↔mm coordinate conversion helpers:**
   - `screenToMm(screenX, screenY)`: given a point in canvas-local pixels, returns `{x, y}` in puzzle mm. Formula: `mmX = (screenX - panX) / (baseScale * zoomLevel)`, where `baseScale = svgViewport.clientWidth / puzzleWidth`
   - `mmToScreen(mmX, mmY)`: inverse — returns screen pixels from mm. Formula: `screenX = mmX * baseScale * zoomLevel + panX`
   - Guard: if `puzzleWidth === 0`, return `{x: 0, y: 0}` for `screenToMm`

4. **Implement whimsy shape path for Canvas drawing:**
   - Function `getWhimsyPath(shape, x, y, scale)` — returns a Canvas path drawing function that draws the shape outline at the given position/scale in puzzle mm coords. Use hardcoded heart/star path definitions matching the Rust shapes (heart: 4 cubic beziers, star: 10-vertex polygon).
   - This is needed because the WASM binary border data only includes the whimsy contour after a successful generation. During drag, we need to draw the shape immediately without waiting for WASM.

5. **Draw whimsy overlay in `drawPuzzle()`:**
   - After drawing edges and border, if whimsy is active, draw the whimsy shape overlay:
     - Semi-transparent fill (e.g., `rgba(200, 50, 50, 0.15)` for visual feedback of which cells will be removed)
     - Colored dashed stroke (e.g., `#c0392b` red, 2px dashed) for the cut line
   - Use `getWhimsyPath()` to draw in puzzle mm coordinates (already in the canvas transform context)
   - During placement mode (shape selected but not placed), draw the shape at cursor position for preview

6. **Implement drag-drop interaction on Canvas:**
   - **Placement mode:** When user selects a shape from the whimsy dropdown, set `isPlacingWhimsy = true` and change cursor to crosshair. On canvas click during placement mode: set `whimsyX/Y` to the clicked mm position, set `whimsyShape` to the selected value, clear placement mode, call `generatePuzzle()`.
   - **Drag start:** On mousedown on Canvas, check if the click is within the whimsy bounding box (in mm coords). If so, set `isDraggingWhimsy = true`, record the offset between click point and whimsy center.
   - **Drag move:** On mousemove while `isDraggingWhimsy`, update `whimsyX/Y` (applying offset), call `drawPuzzle()` for immediate visual feedback, and `scheduleGenerate()` for throttled WASM regeneration.
   - **Drag end:** On mouseup while `isDraggingWhimsy`, clear the flag, call `generatePuzzle()` immediately for final grid state.
   - Must not interfere with existing pan behavior — pan uses right-click or pan starts only when click is NOT on the whimsy.

7. **Implement resize:**
   - On wheel event while cursor is within the whimsy bounding box: adjust `whimsyScale` (multiply by 1.05 for zoom in, divide for zoom out, clamp to 0.2..3.0 range)
   - Call `drawPuzzle()` for immediate visual feedback and `scheduleGenerate()` for throttled regeneration
   - Must distinguish whimsy-resize wheel from viewport zoom wheel — check if cursor is over the whimsy first; if not, fall through to viewport zoom

8. **Wire whimsy config into `buildConfig()`:**
   - When `whimsyShape` is truthy (non-empty), add `whimsy_shape`, `whimsy_x`, `whimsy_y`, `whimsy_scale` to the config object
   - When `whimsyShape` is falsy, omit all whimsy fields (D025 pattern)

9. **URL param persistence:**
   - In `updateURL()`: when whimsy is active, set params `ws` (shape), `wx` (x, 1 decimal), `wy` (y, 1 decimal), `wsc` (scale, 2 decimal)
   - In `loadFromURL()`: restore whimsy state from params if present; set whimsyShape/X/Y/Scale, update dropdown
   - Remove whimsy params when whimsy is cleared

10. **Wire remove button and shape change:**
    - Remove button: clear whimsyShape/X/Y/Scale, hide button, call `generatePuzzle()`
    - Shape dropdown change: if switching to "None", same as remove. If switching to a shape, enter placement mode. If whimsy already placed and user changes shape, update shape in-place (keep position/scale) and regenerate.
    - Add CSS for cursor states: crosshair during placement, grab during whimsy hover, grabbing during drag

## Must-Haves

- [ ] Whimsy shape dropdown in sidebar with None/Heart/Star options
- [ ] Remove Whimsy button (visible only when whimsy is placed)
- [ ] Click on canvas places whimsy at cursor position (mm coords)
- [ ] Drag existing whimsy to reposition — visual feedback is immediate
- [ ] Scroll-wheel resize over whimsy — grid adapts
- [ ] `buildConfig()` includes whimsy fields only when active (D025 pattern)
- [ ] WASM regeneration debounced during drag, immediate on drop
- [ ] URL params ws/wx/wy/wsc persist and restore whimsy state
- [ ] One whimsy at a time — new shape replaces old (R012)
- [ ] Whimsy coordinates are in puzzle mm, zoom/pan independent
- [ ] Pan behavior preserved — pan does NOT conflict with whimsy drag

## Verification

- `grep -q 'whimsyShape\|whimsy_shape' web/src/main.ts` — whimsy wiring exists in JS
- `grep -q 'whimsy-shape' web/index.html` — whimsy dropdown exists in HTML
- `grep -q 'whimsy' web/src/style.css` — whimsy styles exist
- Browser visual: select Heart whimsy → click on canvas → heart appears, grid cells removed
- Browser visual: drag heart → moves smoothly, grid re-adapts
- Browser visual: scroll wheel on heart → resizes, grid re-adapts
- Browser visual: reload page → whimsy preserved from URL params
- Browser visual: click Remove → whimsy disappears, grid restored

## Inputs

- `crates/puzzle-wasm/src/lib.rs` — WASM endpoints now accept whimsy_shape/whimsy_x/whimsy_y/whimsy_scale (from T01)
- `web/src/main.ts` — existing Canvas drawing, zoom/pan, buildConfig(), scheduleGenerate(), drawPuzzle() patterns
- `web/index.html` — existing sidebar layout with Border Shape section to follow as pattern
- `web/src/style.css` — existing control styles and cursor patterns

## Expected Output

- `web/src/main.ts` — whimsy state variables, coordinate helpers, drag-drop handlers, resize handlers, overlay drawing in drawPuzzle(), buildConfig() wiring, URL param persistence (~300-400 new lines)
- `web/index.html` — whimsy shape section with dropdown, remove button, info display
- `web/src/style.css` — whimsy control styles, cursor state overrides (crosshair/grab/grabbing)
