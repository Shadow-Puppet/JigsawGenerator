---
phase: quick
plan: 7
type: execute
wave: 1
depends_on: []
files_modified:
  - web/index.html
  - web/src/main.ts
  - web/src/style.css
autonomous: true
requirements: [QUICK-007]

must_haves:
  truths:
    - "SVG preview fills container width regardless of puzzle dimensions"
    - "A ruler above the preview shows actual puzzle dimensions (width × height with unit)"
    - "Mouse wheel zooms the SVG preview in/out"
    - "Click-drag pans the zoomed SVG preview"
    - "Double-click resets zoom to fit-to-width"
  artifacts:
    - path: "web/src/main.ts"
      provides: "SVG viewBox normalization, zoom/pan state management, ruler updates"
    - path: "web/src/style.css"
      provides: "Container fill styles, ruler styles, zoom/pan cursor styles"
    - path: "web/index.html"
      provides: "Ruler element, updated preview structure"
  key_links:
    - from: "generatePuzzle()"
      to: "SVG viewBox normalization"
      via: "post-innerHTML processing"
      pattern: "setAttribute.*viewBox"
    - from: "zoom/pan handlers"
      to: "SVG transform"
      via: "wheel/mouse events on svg-container"
      pattern: "transform.*scale|translate"
---

<objective>
Make the SVG preview always fill the container width (regardless of puzzle physical dimensions), add a ruler showing actual dimensions, and enable zoom/pan for detail inspection.

Purpose: Currently the SVG uses fixed width/height attributes from WASM (in mm or inches), making the preview size vary with puzzle dimensions. The preview should consistently fill the available space, show real-world dimensions via a ruler, and allow zooming in to inspect connector details.

Output: Updated preview area with responsive SVG, dimension ruler, and interactive zoom/pan.
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
<!-- Current SVG injection pattern in main.ts -->
From web/src/main.ts (generatePuzzle):
```typescript
const svgResult = generate_svg(configJson);
if (svgResult.startsWith("<svg")) {
  svgContainer.innerHTML = svgResult;
  // SVG has width="297mm" height="210mm" style attributes from WASM
}
```

From web/src/main.ts (buildConfig):
```typescript
function buildConfig(): object {
  return {
    rows, cols, width, height, unit, tab, border, seed, kerf_width
  };
}
```

<!-- Key DOM elements -->
- `svgContainer = document.getElementById("svg-container")` — receives innerHTML from WASM
- `.preview-area` — main content area, flexbox column, centered
- `#svg-container svg` — currently `max-width: 100%`, `max-height: calc(100vh - 160px)`
- WASM SVG output has `width` and `height` attributes in physical units (e.g. `width="297mm"`)
- WASM SVG output has a `viewBox` attribute matching the physical dimensions
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: SVG container fill-width + dimension ruler</name>
  <files>web/index.html, web/src/style.css, web/src/main.ts</files>
  <action>
**HTML (web/index.html):**
Add a dimension ruler element and a zoom controls bar inside `.preview-area`, before `#svg-container`:

```html
<main class="preview-area">
  <div id="dimension-ruler">
    <span id="ruler-width" class="ruler-label"></span>
    <span class="ruler-x">×</span>
    <span id="ruler-height" class="ruler-label"></span>
  </div>
  <div id="svg-viewport">
    <div id="svg-container"></div>
  </div>
  <div class="preview-footer">
    <p id="piece-count"></p>
    <div id="zoom-controls">
      <button type="button" id="zoom-in" title="Zoom in">+</button>
      <span id="zoom-level">100%</span>
      <button type="button" id="zoom-out" title="Zoom out">−</button>
      <button type="button" id="zoom-reset" title="Fit to width">⊡</button>
    </div>
  </div>
  <p id="error-display"></p>
</main>
```

The new `#svg-viewport` wrapper is the overflow-clipping container for zoom/pan. The existing `#svg-container` stays inside it as the transformable layer.

**CSS (web/src/style.css):**

Update `.preview-area` to be a column flex filling available space. Replace existing `#svg-container` and `#svg-container svg` rules:

```css
.preview-area {
  display: flex;
  flex-direction: column;
  padding: 1rem 2rem;
  overflow: hidden;
  background: #fafafa;
  height: 100vh;
}

/* Dimension ruler */
#dimension-ruler {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.35rem;
  padding: 0.4rem 0;
  font-size: 0.8rem;
  color: #666;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
  border-bottom: 1px solid #e0e0e0;
  margin-bottom: 0.75rem;
  user-select: none;
}

.ruler-label {
  font-weight: 500;
  color: #444;
}

.ruler-x {
  color: #999;
  font-size: 0.7rem;
}

/* SVG viewport — clips overflow for zoom/pan */
#svg-viewport {
  flex: 1;
  overflow: hidden;
  position: relative;
  cursor: grab;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
  background: #fff;
}

#svg-viewport:active {
  cursor: grabbing;
}

#svg-container {
  width: 100%;
  height: 100%;
  transform-origin: 0 0;
  /* transform set by JS */
}

#svg-container svg {
  display: block;
  width: 100%;
  height: auto;
}

#svg-container svg path {
  stroke-width: 0.5px !important;
}

/* Footer with piece count + zoom controls */
.preview-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0 0;
  flex-shrink: 0;
}

#zoom-controls {
  display: flex;
  align-items: center;
  gap: 0.35rem;
}

#zoom-controls button {
  width: 28px;
  height: 28px;
  border: 1px solid #ccc;
  border-radius: 4px;
  background: #fff;
  color: #444;
  font-size: 1rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

#zoom-controls button:hover {
  background: #f5f5f5;
  border-color: #bbb;
}

#zoom-level {
  font-size: 0.75rem;
  color: #666;
  min-width: 3.5ch;
  text-align: center;
  font-variant-numeric: tabular-nums;
}
```

Also update `#piece-count` to remove `margin-top: 1rem` (now handled by `.preview-footer` padding). Keep the `:empty` display none rule.

Update mobile responsive section: remove the old `#svg-container svg { max-height: 60vh }` rule and ensure `.preview-area` works with `height: auto` on mobile.

**TypeScript (web/src/main.ts):**

After `svgContainer.innerHTML = svgResult;` in `generatePuzzle()`, add SVG normalization:

```typescript
// Normalize SVG: remove fixed width/height, ensure viewBox, fill container
const svgEl = svgContainer.querySelector('svg');
if (svgEl) {
  // Preserve viewBox (WASM sets it), remove fixed dimensions
  svgEl.removeAttribute('width');
  svgEl.removeAttribute('height');
}
```

Add a `updateRuler()` function called after each `generatePuzzle()`:

```typescript
function updateRuler(): void {
  const w = parseFloat(widthInput.value);
  const h = parseFloat(heightInput.value);
  const unit = unitSelect.value === 'Inches' ? 'in' : 'mm';
  const fmt = unit === 'mm' ? 0 : 2;
  rulerWidth.textContent = `${w.toFixed(fmt)} ${unit}`;
  rulerHeight.textContent = `${h.toFixed(fmt)} ${unit}`;
}
```

Cache DOM references for `rulerWidth`, `rulerHeight` as `HTMLElement` in the DOM References section. Call `updateRuler()` at end of `generatePuzzle()` and also during initial setup after `generatePuzzle()` call.

IMPORTANT: When removing SVG width/height, verify the WASM-generated SVG includes a `viewBox` attribute. If it doesn't, parse the width/height values before removing them and set a viewBox from them: `svgEl.setAttribute('viewBox', '0 0 {width} {height}')` where width/height are the numeric values extracted from the attributes (strip unit suffix like "mm").
  </action>
  <verify>
Run `npm run build` in web/ directory — no TypeScript errors. Open `http://localhost:5173` (or dev server), verify:
1. SVG fills container width regardless of puzzle dimensions (try 100x100mm vs 500x200mm)
2. Ruler shows "297.0 mm × 210.0 mm" (or current values)
3. Changing dimensions updates the ruler immediately
  </verify>
  <done>SVG always fills container width. Dimension ruler shows width × height with correct unit. Ruler updates on every parameter change.</done>
</task>

<task type="auto">
  <name>Task 2: Zoom and pan interaction</name>
  <files>web/src/main.ts</files>
  <action>
Add zoom/pan state and handlers to `main.ts`. This task builds on Task 1's `#svg-viewport` wrapper and `#svg-container` transform setup.

**State variables** (add after DOM references section):

```typescript
// ─── Zoom/Pan State ──────────────────────────────────────────
let zoomLevel = 1;
let panX = 0;
let panY = 0;
let isPanning = false;
let panStartX = 0;
let panStartY = 0;

const MIN_ZOOM = 0.5;
const MAX_ZOOM = 20;
const ZOOM_STEP = 1.15; // 15% per wheel tick
```

**DOM refs** — cache `svgViewport`, `zoomLevelDisplay`, `zoomInBtn`, `zoomOutBtn`, `zoomResetBtn`:

```typescript
let svgViewport: HTMLElement;
let zoomLevelDisplay: HTMLElement;
let zoomInBtn: HTMLElement;
let zoomOutBtn: HTMLElement;
let zoomResetBtn: HTMLElement;
```

**Transform application function:**

```typescript
function applyTransform(): void {
  svgContainer.style.transform = `translate(${panX}px, ${panY}px) scale(${zoomLevel})`;
  zoomLevelDisplay.textContent = `${Math.round(zoomLevel * 100)}%`;
}
```

**Reset zoom function** (called on puzzle regeneration and double-click):

```typescript
function resetZoom(): void {
  zoomLevel = 1;
  panX = 0;
  panY = 0;
  applyTransform();
}
```

**Wheel zoom** — zoom toward cursor position:

```typescript
svgViewport.addEventListener('wheel', (e: WheelEvent) => {
  e.preventDefault();
  const rect = svgViewport.getBoundingClientRect();
  const mouseX = e.clientX - rect.left;
  const mouseY = e.clientY - rect.top;

  const oldZoom = zoomLevel;
  if (e.deltaY < 0) {
    zoomLevel = Math.min(MAX_ZOOM, zoomLevel * ZOOM_STEP);
  } else {
    zoomLevel = Math.max(MIN_ZOOM, zoomLevel / ZOOM_STEP);
  }

  // Adjust pan so zoom centers on cursor
  const zoomRatio = zoomLevel / oldZoom;
  panX = mouseX - zoomRatio * (mouseX - panX);
  panY = mouseY - zoomRatio * (mouseY - panY);

  applyTransform();
}, { passive: false });
```

**Mouse drag pan:**

```typescript
svgViewport.addEventListener('mousedown', (e: MouseEvent) => {
  if (e.button !== 0) return; // left click only
  isPanning = true;
  panStartX = e.clientX - panX;
  panStartY = e.clientY - panY;
  e.preventDefault();
});

window.addEventListener('mousemove', (e: MouseEvent) => {
  if (!isPanning) return;
  panX = e.clientX - panStartX;
  panY = e.clientY - panStartY;
  applyTransform();
});

window.addEventListener('mouseup', () => {
  isPanning = false;
});
```

**Double-click to reset:**

```typescript
svgViewport.addEventListener('dblclick', () => {
  resetZoom();
});
```

**Zoom button handlers:**

```typescript
zoomInBtn.addEventListener('click', () => {
  const rect = svgViewport.getBoundingClientRect();
  const cx = rect.width / 2;
  const cy = rect.height / 2;
  const oldZoom = zoomLevel;
  zoomLevel = Math.min(MAX_ZOOM, zoomLevel * ZOOM_STEP);
  const zoomRatio = zoomLevel / oldZoom;
  panX = cx - zoomRatio * (cx - panX);
  panY = cy - zoomRatio * (cy - panY);
  applyTransform();
});

zoomOutBtn.addEventListener('click', () => {
  const rect = svgViewport.getBoundingClientRect();
  const cx = rect.width / 2;
  const cy = rect.height / 2;
  const oldZoom = zoomLevel;
  zoomLevel = Math.max(MIN_ZOOM, zoomLevel / ZOOM_STEP);
  const zoomRatio = zoomLevel / oldZoom;
  panX = cx - zoomRatio * (cx - panX);
  panY = cy - zoomRatio * (cy - panY);
  applyTransform();
});

zoomResetBtn.addEventListener('click', () => {
  resetZoom();
});
```

**On puzzle regeneration** — call `resetZoom()` at the end of `generatePuzzle()` (after `updateRuler()`) so that generating a new puzzle resets the viewport to fit.

**Touch support** — add basic pinch-zoom and touch-drag for mobile:

```typescript
let lastTouchDist = 0;
let lastTouchX = 0;
let lastTouchY = 0;

svgViewport.addEventListener('touchstart', (e: TouchEvent) => {
  if (e.touches.length === 1) {
    isPanning = true;
    panStartX = e.touches[0].clientX - panX;
    panStartY = e.touches[0].clientY - panY;
  } else if (e.touches.length === 2) {
    isPanning = false;
    const dx = e.touches[0].clientX - e.touches[1].clientX;
    const dy = e.touches[0].clientY - e.touches[1].clientY;
    lastTouchDist = Math.sqrt(dx * dx + dy * dy);
    lastTouchX = (e.touches[0].clientX + e.touches[1].clientX) / 2;
    lastTouchY = (e.touches[0].clientY + e.touches[1].clientY) / 2;
  }
}, { passive: true });

svgViewport.addEventListener('touchmove', (e: TouchEvent) => {
  e.preventDefault();
  if (e.touches.length === 1 && isPanning) {
    panX = e.touches[0].clientX - panStartX;
    panY = e.touches[0].clientY - panStartY;
    applyTransform();
  } else if (e.touches.length === 2) {
    const dx = e.touches[0].clientX - e.touches[1].clientX;
    const dy = e.touches[0].clientY - e.touches[1].clientY;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const midX = (e.touches[0].clientX + e.touches[1].clientX) / 2;
    const midY = (e.touches[0].clientY + e.touches[1].clientY) / 2;
    const rect = svgViewport.getBoundingClientRect();

    if (lastTouchDist > 0) {
      const oldZoom = zoomLevel;
      zoomLevel = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoomLevel * (dist / lastTouchDist)));
      const zoomRatio = zoomLevel / oldZoom;
      const cx = midX - rect.left;
      const cy = midY - rect.top;
      panX = cx - zoomRatio * (cx - panX);
      panY = cy - zoomRatio * (cy - panY);
      applyTransform();
    }

    lastTouchDist = dist;
    lastTouchX = midX;
    lastTouchY = midY;
  }
}, { passive: false });

svgViewport.addEventListener('touchend', () => {
  isPanning = false;
  lastTouchDist = 0;
});
```

Wire all event listeners inside `main()` after DOM caching.

IMPORTANT: Do NOT use any external zoom/pan library. Implement with vanilla event handlers as shown. Keep the `#svg-container svg path { stroke-width: 0.5px !important; }` CSS rule — this ensures visible strokes at all zoom levels for screen display (downloaded SVGs still use hairline strokes).
  </action>
  <verify>
Run `npm run build` in web/ — no TypeScript errors. Open dev server, verify:
1. Mouse wheel zooms in/out centered on cursor position
2. Click-drag pans the SVG when zoomed in
3. Double-click resets to fit-to-width
4. Zoom buttons (+/−/⊡) work correctly
5. Zoom percentage display updates
6. Changing puzzle parameters resets zoom to fit
7. `cursor: grab` shown, switches to `grabbing` while dragging
  </verify>
  <done>Zoom via mouse wheel (cursor-centered), drag to pan, double-click to reset. Zoom buttons functional. Touch pinch-zoom and drag for mobile. Zoom resets on puzzle regeneration. Zoom level percentage displayed.</done>
</task>

</tasks>

<verification>
1. `npm run build` in `web/` completes without errors
2. SVG preview fills container width with 100x100mm puzzle AND with 500x200mm puzzle
3. Dimension ruler shows correct values and updates on parameter change
4. Zoom in with wheel → SVG detail visible, stroke width remains visible
5. Pan with drag → can navigate zoomed SVG
6. Double-click → resets to fit view
7. Generate new puzzle → zoom resets, ruler updates
8. Download SVG → still has original dimensions (not modified by preview normalization)
</verification>

<success_criteria>
- SVG preview consistently fills available width regardless of puzzle physical dimensions
- Ruler displays actual puzzle dimensions (width × height + unit) and updates live
- Wheel zoom works centered on cursor position, range 50%-2000%
- Drag pan works when zoomed
- Double-click and reset button return to fit-to-width
- Zoom percentage displayed and updated
- Downloaded SVGs unaffected (still have physical dimensions for laser cutting)
- Touch zoom/pan works on mobile
</success_criteria>

<output>
After completion, create `.planning/quick/7-svg-preview-fill-container-width-add-rul/7-SUMMARY.md`
</output>
