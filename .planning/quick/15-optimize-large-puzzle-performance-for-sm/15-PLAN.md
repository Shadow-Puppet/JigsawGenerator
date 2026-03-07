---
phase: quick-15
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/puzzle-core/src/binary_export.rs
  - crates/puzzle-core/src/lib.rs
  - crates/puzzle-wasm/src/lib.rs
  - web/src/main.ts
  - web/src/style.css
  - web/index.html
autonomous: true
requirements: [PERF-CANVAS, PERF-CULL, PERF-BINARY, PERF-CACHE]
must_haves:
  truths:
    - "Zoom/pan on 6000-10000 piece puzzles is smooth (60fps)"
    - "Only edges visible in the viewport are drawn each frame"
    - "Download button produces identical SVG to before without regenerating"
    - "Visual output matches previous SVG rendering (black strokes, no fill)"
  artifacts:
    - path: "crates/puzzle-core/src/binary_export.rs"
      provides: "Binary edge data serialization for WASM transfer"
      exports: ["edges_to_binary", "border_to_binary"]
    - path: "web/src/main.ts"
      provides: "Canvas 2D renderer with viewport culling"
  key_links:
    - from: "crates/puzzle-wasm/src/lib.rs"
      to: "crates/puzzle-core/src/binary_export.rs"
      via: "generate_edges_binary() calls edges_to_binary()"
      pattern: "edges_to_binary"
    - from: "web/src/main.ts"
      to: "crates/puzzle-wasm/src/lib.rs"
      via: "JS calls generate_edges_binary(), receives Float64Array"
      pattern: "generate_edges_binary"
    - from: "web/src/main.ts"
      to: "Canvas 2D context"
      via: "drawVisibleEdges() uses bezierCurveTo for viewport-culled rendering"
      pattern: "bezierCurveTo"
---

<objective>
Switch puzzle display rendering from a single monolithic SVG `<path>` to Canvas 2D with viewport culling and binary data transfer. This eliminates the #1 performance bottleneck: browser SVG renderer choking on a single path with ~100K curve segments for large puzzles.

Purpose: Make zoom/pan smooth (60fps) on 6000-10000 piece puzzles by rendering only visible edges via Canvas 2D, transferring binary edge data from WASM (~1.2MB vs 6MB SVG string), and caching SVG for download.

Output: Canvas-based puzzle display with viewport culling, binary WASM data transfer, cached SVG download.
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@web/src/main.ts
@crates/puzzle-core/src/svg_export.rs
@crates/puzzle-core/src/edge.rs
@crates/puzzle-wasm/src/lib.rs
@crates/puzzle-core/src/grid.rs
@crates/puzzle-core/src/lib.rs
@web/src/style.css
@web/index.html

<interfaces>
<!-- From crates/puzzle-core/src/edge.rs -->
```rust
pub struct Edge {
    pub start: Point,   // kurbo::Point {x: f64, y: f64}
    pub end: Point,
    pub is_border: bool,
    pub direction: TabDirection,
    pub connector: Option<Vec<CubicBez>>,  // kurbo::CubicBez {p0, p1, p2, p3}
}
```

<!-- From crates/puzzle-core/src/grid.rs -->
```rust
pub struct PuzzleGrid {
    pub config: PuzzleConfig,
    pub h_edges: Vec<Edge>,
    pub v_edges: Vec<Edge>,
}
```

<!-- From crates/puzzle-core/src/svg_export.rs -->
```rust
pub fn generate_svg(grid: &PuzzleGrid) -> String;
fn build_border_path(grid: &PuzzleGrid) -> BezPath;
fn build_connector_paths(grid: &PuzzleGrid) -> BezPath;
fn edge_transform(start: Point, end: Point) -> Affine;
```

<!-- kurbo::CubicBez layout -->
```rust
pub struct CubicBez {
    pub p0: Point,  // start
    pub p1: Point,  // control point 1
    pub p2: Point,  // control point 2
    pub p3: Point,  // end
}
```

<!-- From crates/puzzle-wasm/src/lib.rs -->
```rust
#[wasm_bindgen]
pub fn generate_svg(config_json: &str) -> String;
```

<!-- From web/src/main.ts key state -->
```typescript
let svgEl: SVGSVGElement | null = null;
let cachedSvgPath: SVGPathElement | null = null;
let zoomLevel = 1;
let panX = 0;
let panY = 0;

function applyTransform(): void {
  svgContainer.style.transform = `translate(${panX}px, ${panY}px) scale(${zoomLevel})`;
  // ...adjusts stroke-width
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: WASM binary edge data export</name>
  <files>
    crates/puzzle-core/src/binary_export.rs
    crates/puzzle-core/src/lib.rs
    crates/puzzle-wasm/src/lib.rs
  </files>
  <action>
**Create `crates/puzzle-core/src/binary_export.rs`** — a new module that serializes edge geometry as a flat `Vec<f64>` for zero-copy transfer to JS.

The binary format encodes each internal edge as a sequence of cubic bezier curves in GLOBAL coordinates (already transformed from edge-local). Each edge's data is:

```
[edge_header: 4 floats] [curve0: 6 floats] [curve1: 6 floats] ... [curveN: 6 floats]
```

**Edge header (4 floats):**
- `start.x, start.y` — edge start in mm (for bounding box / culling on JS side)
- `end.x, end.y` — edge end in mm

**Each curve (6 floats):** `p1.x, p1.y, p2.x, p2.y, p3.x, p3.y` — the 3 control/end points of a cubic bezier (p0 is implicit: it's p3 of previous curve, or the first transformed p0 for curve[0]).

**Special: curve[0] has 8 floats:** `p0.x, p0.y, p1.x, p1.y, p2.x, p2.y, p3.x, p3.y` — includes the moveTo point.

Wait — simpler approach. Use a **segment-count prefix** per edge:

```
Per edge: [start.x, start.y, end.x, end.y, num_curves (as f64), p0.x, p0.y, p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, <next curves: p1.x, p1.y, p2.x, p2.y, p3.x, p3.y>...]
```

Actually, simplest and fastest: **fixed 5 curves per connector** (this is always true — see classic_connector.rs which always produces exactly 5 CubicBez segments). So the format is:

```
Per edge (37 floats):
  [start.x, start.y, end.x, end.y]     — 4 floats, bounding hint for culling
  [p0.x, p0.y]                          — 2 floats, moveTo point (first curve's p0, transformed)
  [c0.p1.x, c0.p1.y, c0.p2.x, c0.p2.y, c0.p3.x, c0.p3.y]  — 6 floats per curve * 5 curves = 30 floats
  [sentinel: NaN]                       — 1 float, edge delimiter
Total: 4 + 2 + 30 + 1 = 37 floats per edge
```

No — skip the sentinel. Fixed stride of 36 floats per edge (4 header + 2 moveTo + 30 curve data). JS knows the stride and can iterate: `for (let i = 0; i < data.length; i += 36)`.

**Implementation in `binary_export.rs`:**

```rust
use kurbo::{Affine, Point};
use crate::grid::PuzzleGrid;

/// Serialize all internal edge connector curves as a flat f64 array.
///
/// Layout: chunks of 36 f64s per edge.
/// [0..4]: start.x, start.y, end.x, end.y (bounding for viewport culling)
/// [4..6]: moveTo x, y (first curve's p0, transformed to global coords)
/// [6..36]: 5 curves * 6 floats (p1.x, p1.y, p2.x, p2.y, p3.x, p3.y)
///
/// All coordinates are in mm (puzzle coordinate space).
pub fn edges_to_binary(grid: &PuzzleGrid) -> Vec<f64> {
    // ... iterate h_edges then v_edges, skip borders,
    // apply edge_transform (same as svg_export.rs), write to Vec<f64>
}
```

Use the same `edge_transform` logic from `svg_export.rs` — replicate the function or make it `pub(crate)` in `svg_export.rs`. Prefer making `edge_transform` pub(crate) in svg_export.rs and importing it, to avoid duplication.

Also create `border_to_binary(grid: &PuzzleGrid) -> Vec<f64>` for the border path. The border is small and always drawn, so encode it as a simple sequence of drawing commands using a command-type prefix:
- `0.0` = moveTo, followed by 2 floats (x, y)
- `1.0` = lineTo, followed by 2 floats (x, y)
- `2.0` = curveTo, followed by 6 floats (p1.x, p1.y, p2.x, p2.y, p3.x, p3.y)
- `3.0` = closePath, followed by 0 floats

This mirrors the BezPath elements already generated by `build_border_path()`. Make `build_border_path` pub(crate) in svg_export.rs and iterate its PathEl elements.

**Update `crates/puzzle-core/src/lib.rs`:** Add `pub mod binary_export;` and `pub use binary_export::*;`.

**Update `crates/puzzle-wasm/src/lib.rs`:** Add a new `#[wasm_bindgen]` function:

```rust
use wasm_bindgen::prelude::*;
use js_sys::Float64Array;

/// Generate binary edge data for Canvas 2D rendering.
///
/// Returns a JS object with two Float64Array properties:
/// - edges: internal edge connector curves (36 floats per edge)
/// - border: border path drawing commands
/// - width: puzzle width in mm
/// - height: puzzle height in mm
///
/// Uses wasm_bindgen's JsValue return for structured data.
#[wasm_bindgen]
pub fn generate_edges_binary(config_json: &str) -> JsValue {
    // 1. Parse config, handle empty seed
    // 2. Create PuzzleGrid, generate connectors
    // 3. Call edges_to_binary() and border_to_binary()
    // 4. Create Float64Arrays from the Vec<f64>s
    // 5. Build a JS object with edges, border, width, height fields
    //    using js_sys::Object, js_sys::Reflect::set
}
```

Add `js-sys` to `crates/puzzle-wasm/Cargo.toml` dependencies: `js-sys = "0.3"`.

**Also cache the SVG string:** Add a second WASM function `generate_svg_cached(config_json: &str) -> String` that is identical to the existing `generate_svg` but also stores the result in a `thread_local!` static. Then add `get_cached_svg() -> String` that returns the last cached SVG. Actually — simpler: just have `generate_edges_binary` also generate and cache the SVG string internally. Add:

```rust
use std::cell::RefCell;

thread_local! {
    static CACHED_SVG: RefCell<String> = RefCell::new(String::new());
}

#[wasm_bindgen]
pub fn get_cached_svg() -> String {
    CACHED_SVG.with(|c| c.borrow().clone())
}
```

And in `generate_edges_binary`, after generating the grid+connectors, also call `puzzle_core::generate_svg(&grid)` and store it in `CACHED_SVG`. This way the SVG is generated once alongside the binary data, and the download button can call `get_cached_svg()` instead of regenerating.

**Important:** Keep the existing `generate_svg` function unchanged — it's still used as a fallback and for direct SVG downloads.
  </action>
  <verify>
    Run `cargo test -p puzzle-core` and `cargo test -p puzzle-wasm` — all existing tests pass plus new binary export tests.
    Run `wasm-pack build crates/puzzle-wasm --target web --release` — WASM builds successfully with new exports.
    Verify the built WASM exports `generate_edges_binary` and `get_cached_svg` by checking the generated .d.ts or .js glue file.
  </verify>
  <done>
    - `binary_export.rs` exists with `edges_to_binary()` and `border_to_binary()` functions
    - WASM exports `generate_edges_binary()` returning JsValue with edges/border Float64Arrays + dimensions
    - WASM exports `get_cached_svg()` returning cached SVG string from last generation
    - `js-sys` dependency added to puzzle-wasm Cargo.toml
    - `edge_transform` and `build_border_path` made pub(crate) in svg_export.rs
    - All existing tests pass, new unit tests for binary export format
    - WASM builds successfully
  </done>
</task>

<task type="auto">
  <name>Task 2: Canvas 2D renderer with viewport culling, cached SVG download</name>
  <files>
    web/src/main.ts
    web/src/style.css
    web/index.html
  </files>
  <action>
**Replace SVG display with Canvas 2D rendering in `web/src/main.ts`.**

This is the critical task — switching from "set SVG innerHTML and let browser render everything" to "draw only visible edges on a canvas each frame."

**HTML changes (`web/index.html`):**
- Inside `#svg-container`, add a `<canvas id="puzzle-canvas"></canvas>` element alongside the existing SVG setup. The canvas will be used for display; SVG elements are no longer needed for display.

**CSS changes (`web/src/style.css`):**
- Style `#puzzle-canvas` to fill `#svg-container`: `width: 100%; height: 100%;`
- Remove `#svg-container svg` and `#svg-container svg path` rules (SVG no longer in DOM for display)
- Keep `#svg-container` `will-change: transform` and `transform-origin: 0 0`
- BUT: change zoom/pan approach. Instead of CSS transform on a container, zoom/pan is now done by adjusting the Canvas 2D transform matrix before drawing. Remove `will-change: transform` from `#svg-container`. The canvas itself handles the coordinate transform.

**New rendering architecture in `main.ts`:**

1. **State variables (add near top):**
```typescript
let edgesData: Float64Array | null = null;     // 36 floats per edge
let borderData: Float64Array | null = null;    // command-prefixed border path
let puzzleWidth = 0;                            // mm
let puzzleHeight = 0;                           // mm
let canvas: HTMLCanvasElement | null = null;
let ctx: CanvasRenderingContext2D | null = null;
const EDGE_STRIDE = 36;                         // floats per edge
```

2. **`generatePuzzle()` rewrite:**
```typescript
function generatePuzzle(): void {
  const config = buildConfig();
  const configJson = JSON.stringify(config);

  // Generate binary edge data (also caches SVG internally for download)
  const result = generate_edges_binary(configJson);
  if (!result || result.error) {
    // handle error...
    return;
  }

  edgesData = result.edges;     // Float64Array
  borderData = result.border;   // Float64Array
  puzzleWidth = result.width;   // f64
  puzzleHeight = result.height; // f64

  // Update piece count display (same JS math as before)
  // ...existing piece count code...

  // Resize canvas to match viewport pixel dimensions
  resizeCanvas();

  // Draw the puzzle
  drawPuzzle();

  errorDisplay.style.display = 'none';
}
```

3. **Canvas setup and resize:**
```typescript
function resizeCanvas(): void {
  if (!canvas || !ctx) return;
  const viewport = document.getElementById('svg-viewport')!;
  const dpr = window.devicePixelRatio || 1;
  const rect = viewport.getBoundingClientRect();
  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;
  canvas.style.width = rect.width + 'px';
  canvas.style.height = rect.height + 'px';
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
}
```

Add a `ResizeObserver` on `#svg-viewport` to call `resizeCanvas()` + `drawPuzzle()` on resize.

4. **`drawPuzzle()` — the core renderer with viewport culling:**
```typescript
function drawPuzzle(): void {
  if (!ctx || !edgesData || !borderData) return;

  const viewport = document.getElementById('svg-viewport')!;
  const vpW = viewport.clientWidth;
  const vpH = viewport.clientHeight;

  // Clear
  ctx.clearRect(0, 0, vpW, vpH);

  // Compute the transform: puzzle mm coords -> screen pixels
  // The puzzle should fill the viewport width (matching previous SVG behavior)
  const baseScale = vpW / puzzleWidth;  // px per mm at zoom=1
  const scale = baseScale * zoomLevel;

  // Viewport bounds in puzzle mm coordinates (for culling)
  const vpLeft = -panX / scale;
  const vpTop = -panY / scale;
  const vpRight = vpLeft + vpW / scale;
  const vpBottom = vpTop + vpH / scale;

  // Set up canvas transform
  ctx.save();
  ctx.translate(panX, panY);
  ctx.scale(scale, scale);

  // Style
  ctx.strokeStyle = '#000000';
  ctx.lineWidth = 0.2 / scale;  // visually consistent stroke width regardless of zoom
  ctx.fillStyle = 'none';
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';

  // Draw border (always visible, small data)
  drawBorder(ctx);

  // Draw internal edges with viewport culling
  drawVisibleEdges(ctx, vpLeft, vpTop, vpRight, vpBottom);

  ctx.restore();
}
```

5. **`drawBorder()` — draw border from binary command data:**
```typescript
function drawBorder(ctx: CanvasRenderingContext2D): void {
  if (!borderData) return;
  ctx.beginPath();
  let i = 0;
  while (i < borderData.length) {
    const cmd = borderData[i];
    if (cmd === 0) {        // moveTo
      ctx.moveTo(borderData[i+1], borderData[i+2]);
      i += 3;
    } else if (cmd === 1) { // lineTo
      ctx.lineTo(borderData[i+1], borderData[i+2]);
      i += 3;
    } else if (cmd === 2) { // curveTo
      ctx.bezierCurveTo(
        borderData[i+1], borderData[i+2],
        borderData[i+3], borderData[i+4],
        borderData[i+5], borderData[i+6]
      );
      i += 7;
    } else if (cmd === 3) { // closePath
      ctx.closePath();
      i += 1;
    } else {
      i += 1; // skip unknown
    }
  }
  ctx.stroke();
}
```

6. **`drawVisibleEdges()` — viewport culling:**
```typescript
function drawVisibleEdges(
  ctx: CanvasRenderingContext2D,
  vpL: number, vpT: number, vpR: number, vpB: number
): void {
  if (!edgesData) return;
  const data = edgesData;
  const len = data.length;

  ctx.beginPath();

  for (let i = 0; i < len; i += EDGE_STRIDE) {
    // Read edge bounding box from header (start/end points)
    const sx = data[i], sy = data[i+1], ex = data[i+2], ey = data[i+3];

    // Quick AABB cull: edge bounding box vs viewport
    // Add margin for connector protrusion (knob extends ~25% of edge length perpendicular)
    const edgeLen = Math.abs(ex - sx) + Math.abs(ey - sy); // Manhattan approx
    const margin = edgeLen * 0.35; // generous margin for knob height
    const minX = Math.min(sx, ex) - margin;
    const maxX = Math.max(sx, ex) + margin;
    const minY = Math.min(sy, ey) - margin;
    const maxY = Math.max(sy, ey) + margin;

    if (maxX < vpL || minX > vpR || maxY < vpT || minY > vpB) {
      continue; // Edge entirely outside viewport — skip
    }

    // MoveTo (first curve's p0)
    ctx.moveTo(data[i+4], data[i+5]);

    // 5 curves, 6 floats each, starting at offset 6
    for (let c = 0; c < 5; c++) {
      const base = i + 6 + c * 6;
      ctx.bezierCurveTo(
        data[base], data[base+1],
        data[base+2], data[base+3],
        data[base+4], data[base+5]
      );
    }
  }

  ctx.stroke();
}
```

7. **Update `applyTransform()`:**
Replace the CSS transform approach. Instead of setting `svgContainer.style.transform`, just call `drawPuzzle()` which applies the transform via canvas context:

```typescript
function applyTransform(): void {
  drawPuzzle();
  zoomLevelDisplay.textContent = `${Math.round(zoomLevel * 100)}%`;
}
```

The existing `scheduleTransform()` rAF throttle stays — it calls `applyTransform()` which now calls `drawPuzzle()`.

8. **Update `resetZoom()`:**
Same logic (compute vertical centering) but using canvas dimensions instead of SVG element bounds:
```typescript
function resetZoom(): void {
  zoomLevel = 1;
  panX = 0;
  panY = 0;
  if (canvas && puzzleWidth > 0) {
    const viewport = document.getElementById('svg-viewport')!;
    const vpH = viewport.clientHeight;
    const baseScale = viewport.clientWidth / puzzleWidth;
    const svgH = puzzleHeight * baseScale;
    panY = Math.max(0, (vpH - svgH) / 2);
  }
  applyTransform();
}
```

9. **Update download button handler (lines 924-939):**
Replace `generate_svg(configJson)` call with `get_cached_svg()`:
```typescript
downloadBtn.addEventListener('click', () => {
  const svgContent = get_cached_svg();
  if (!svgContent || !svgContent.startsWith('<svg')) return;
  const config = buildConfig() as Record<string, unknown>;
  const filename = `puzzle-${config.rows}x${config.cols}-seed-${config.seed}.svg`;
  const blob = new Blob([svgContent], { type: 'image/svg+xml' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
});
```

10. **Clean up removed SVG state:**
- Remove `svgEl`, `cachedSvgPath` variables and all references
- Remove SVG innerHTML / attribute diffing code in `generatePuzzle()`
- Remove the CSS `#svg-container svg` and `#svg-container svg path` rules
- Remove `will-change: transform` from `#svg-container` CSS (canvas handles its own rendering)
- Remove the stroke-width adjustment in `applyTransform()` (canvas lineWidth handles this)
- Keep the `contain: layout style paint` on `#svg-viewport`

11. **Initialize canvas in `main()` function:**
```typescript
canvas = document.getElementById('puzzle-canvas') as HTMLCanvasElement;
ctx = canvas.getContext('2d');
```

12. **Import the new WASM functions:**
Add `generate_edges_binary` and `get_cached_svg` to the import from the WASM module (same import pattern as existing `generate_svg`).

13. **Ruler display:** The `updateRuler()` function reads from input values, not SVG — no changes needed.

14. **Remove the import of `generate_svg` from WASM** if it was the only display-render path. Actually, KEEP `generate_svg` imported — `get_cached_svg` needs it internally in WASM, and it's still useful for any fallback. Just remove calls to it from the display path. The download path now uses `get_cached_svg()`.

**Key architectural decision:** Canvas 2D with manual transform (not CSS transform on container). This is critical because:
- CSS transform on a canvas just scales pixels (blurry)
- Canvas context transform redraws at native resolution at any zoom level
- Viewport culling requires knowing the visible region, which is trivial with canvas transform math

**Performance characteristics at 10K pieces:**
- ~20,000 internal edges, each 36 floats = 720,000 floats = 5.6MB Float64Array
- At high zoom (10x+), viewport might show ~25-100 edges = ~150-600 bezierCurveTo calls = sub-millisecond
- At zoom=1, all ~20K edges drawn = ~100K bezierCurveTo calls, but Canvas 2D handles this in ~5-15ms
- Border: ~20-30 commands, always drawn, negligible cost
  </action>
  <verify>
    Run `npm run build` from `web/` — TypeScript compiles without errors.
    Run `npm run dev` from `web/`, open browser, verify:
    1. Puzzle renders visually (black lines on white, same as before)
    2. Zoom in/out with mouse wheel — smooth, no jank
    3. Pan with mouse drag — smooth
    4. Change piece count to 6000+ — zoom/pan still smooth
    5. Download button produces valid SVG file
    6. Zoom level % display updates correctly
    7. Reset zoom button works
  </verify>
  <done>
    - Canvas element replaces SVG for puzzle display
    - `drawPuzzle()` renders border + viewport-culled internal edges via Canvas 2D
    - Zoom/pan operates through canvas context transform, not CSS transform
    - Only edges intersecting the viewport AABB are drawn each frame
    - Download uses `get_cached_svg()` instead of regenerating
    - Stroke width visually consistent across zoom levels
    - ResizeObserver handles viewport resize
    - All existing zoom/pan controls (wheel, buttons, touch) work with canvas renderer
    - No SVG elements in DOM for display (canvas only)
  </done>
</task>

</tasks>

<verification>
1. **Performance test:** Set piece count to 8000 (80x100 grid). Zoom to 5x. Pan around. Should be 60fps smooth.
2. **Visual test:** Compare a 48-piece puzzle (6x8) screenshot between old SVG and new canvas — lines should match.
3. **Download test:** Download SVG at 8000 pieces, open in browser — should be valid SVG with physical mm dimensions.
4. **Build test:** `npm run build` produces no errors. WASM builds successfully.
5. **Regression test:** `cargo test -p puzzle-core && cargo test -p puzzle-wasm` — all tests pass.
</verification>

<success_criteria>
- Zoom/pan on 6000+ piece puzzles is smooth (no visible frame drops)
- Canvas displays puzzle correctly at all zoom levels (crisp lines, not blurry)
- Download produces identical SVG output (laser-cutter compatible with mm dimensions)
- Binary data transfer reduces WASM→JS payload from ~6MB SVG string to ~5.6MB Float64Array (structured, no parsing)
- Viewport culling reduces per-frame draw calls from O(all_edges) to O(visible_edges)
- All existing features work: zoom buttons, wheel zoom, pan, touch pan/pinch, reset zoom, piece count display
</success_criteria>

<output>
After completion, create `.planning/quick/15-optimize-large-puzzle-performance-for-sm/15-SUMMARY.md`
</output>
