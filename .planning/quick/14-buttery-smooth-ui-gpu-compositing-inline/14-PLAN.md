---
phase: quick-14
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - web/src/style.css
  - web/src/main.ts
  - crates/puzzle-wasm/Cargo.toml
  - Cargo.toml
autonomous: true
requirements: [PERF-CSS, PERF-JS, PERF-WASM]

must_haves:
  truths:
    - "SVG container is GPU-composited for smooth pan/zoom"
    - "SVG viewport isolates layout/paint for reflow containment"
    - "Tab max calculation runs in pure JS without WASM roundtrip"
    - "Subsequent puzzle renders update path d attr instead of full innerHTML"
    - "Pan/zoom is rAF-throttled to avoid redundant repaints"
    - "URL sync is debounced at 300ms trailing"
    - "WASM is compiled with -O3 and LTO for max throughput"
  artifacts:
    - path: "web/src/style.css"
      provides: "GPU compositing and containment CSS"
      contains: "will-change: transform"
    - path: "web/src/main.ts"
      provides: "Inline tab max, SVG path diffing, rAF transform, debounced URL"
    - path: "crates/puzzle-wasm/Cargo.toml"
      provides: "wasm-opt -O3"
      contains: "-O3"
    - path: "Cargo.toml"
      provides: "LTO + codegen-units=1 release profile"
      contains: "lto = true"
  key_links:
    - from: "web/src/main.ts"
      to: "web/src/style.css"
      via: "#svg-container will-change enables GPU layer for transform updates"
      pattern: "will-change"
---

<objective>
Apply 7 targeted performance optimizations: GPU compositing CSS, layout containment, inline JS tab-max math, SVG path attribute diffing, rAF-throttled pan/zoom transforms, debounced URL sync, and WASM -O3 + LTO build.

Purpose: Eliminate jank in pan/zoom, reduce unnecessary WASM calls, minimize DOM thrashing on re-renders, and optimize WASM binary for maximum generation throughput.
Output: Faster, smoother puzzle generator with no behavioral changes.
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@web/src/style.css
@web/src/main.ts
@crates/puzzle-wasm/Cargo.toml
@Cargo.toml
</context>

<tasks>

<task type="auto">
  <name>Task 1: CSS GPU compositing + JS performance optimizations</name>
  <files>web/src/style.css, web/src/main.ts</files>
  <action>
**CSS changes (web/src/style.css):**

1. Add `will-change: transform;` to `#svg-container` (line ~442-446) — promotes to GPU layer for hardware-accelerated zoom/pan.

2. Add `contain: layout style paint;` to `#svg-viewport` (line ~428-436) — isolates SVG containment so layout changes don't propagate up.

**JS changes (web/src/main.ts):**

3. **Inline tab max math.** Replace the `updateTabMax()` function (lines 169-189) to compute safe tab max in pure JS instead of calling `safe_tab_max()` WASM function. The math is:
```typescript
function updateTabMax(): void {
  const rows = parseInt(rowsInput.value, 10) || 1;
  const cols = parseInt(colsInput.value, 10) || 1;
  const w = parseFloat(widthInput.value) || 1;
  const h = parseFloat(heightInput.value) || 1;
  const cellW = w / cols;
  const cellH = h / rows;
  const maxH = cellH / (2.0 * cellW * 1.2);
  const maxV = cellW / (2.0 * cellH * 1.2);
  const maxApproach = 1.0 / (2.0 * 1.2);
  const safeMax = Math.min(maxH, maxV, maxApproach) * 0.9;
  const tabMax = Math.min(safeMax, 0.25);

  tabSlider.max = String(tabMax);
  tabMaxSlider.max = String(tabMax);
  if (parseFloat(tabSlider.value) > tabMax) tabSlider.value = String(tabMax);
  if (parseFloat(tabMaxSlider.value) > tabMax) tabMaxSlider.value = String(tabMax);
  tabReadout.textContent = parseFloat(tabSlider.value).toFixed(2);
  tabMaxReadout.textContent = parseFloat(tabMaxSlider.value).toFixed(2);
}
```
Also remove `safe_tab_max` from the WASM import at line 1-5 (keep `init`, `generate_svg`, `init_panic_hook`). Add `let tabMaxReadout: HTMLElement;` declaration near the other readout refs, and wire it to `document.getElementById("tab-max-readout")` if that element exists (check HTML), otherwise use `tabReadout` for both. Actually, looking at the readout updates in `updateReadouts()`, the readout display is handled there — so in `updateTabMax()` just skip the readout lines and let `updateReadouts()` handle display. The function should just update `.max` and clamp `.value`:
```typescript
function updateTabMax(): void {
  const rows = parseInt(rowsInput.value, 10) || 1;
  const cols = parseInt(colsInput.value, 10) || 1;
  const w = parseFloat(widthInput.value) || 1;
  const h = parseFloat(heightInput.value) || 1;
  const cellW = w / cols;
  const cellH = h / rows;
  const maxH = cellH / (2.0 * cellW * 1.2);
  const maxV = cellW / (2.0 * cellH * 1.2);
  const maxApproach = 1.0 / (2.0 * 1.2);
  const safeMax = Math.min(maxH, maxV, maxApproach) * 0.9;
  const tabMax = Math.min(safeMax, 0.25);

  tabSlider.max = String(tabMax);
  tabMaxSlider.max = String(tabMax);
  if (parseFloat(tabSlider.value) > tabMax) tabSlider.value = String(tabMax);
  if (parseFloat(tabMaxSlider.value) > tabMax) tabMaxSlider.value = String(tabMax);
}
```

4. **SVG path attribute diffing.** Add a module-level `let svgEl: SVGSVGElement | null = null;` variable. In `generatePuzzle()`, after getting `svgResult`:
   - First render (`svgEl === null`): use `innerHTML` as before, then cache `svgEl = svgContainer.querySelector("svg")` and `cachedSvgPath = svgEl?.querySelector("path")`.
   - Subsequent renders (`svgEl !== null`): extract `d` attr via `svgResult.match(/d='([^']*)'/)?.[1]` and `viewBox` via `svgResult.match(/viewBox='([^']*)'/)?.[1]`, then do `cachedSvgPath!.setAttribute('d', newD)` and `svgEl.setAttribute('viewBox', newViewBox)`. Skip the SVG normalization block (removeAttribute width/height) on subsequent renders since we're only updating attrs.
   - The existing `resetZoom()` function uses `svgContainer.querySelector("svg")` — update it to use the cached `svgEl` variable instead.

5. **rAF-throttle pan/zoom.** Add a `scheduleTransform()` function:
```typescript
let transformRafPending = false;
function scheduleTransform(): void {
  if (transformRafPending) return;
  transformRafPending = true;
  requestAnimationFrame(() => {
    transformRafPending = false;
    applyTransform();
  });
}
```
Replace `applyTransform()` with `scheduleTransform()` in these handlers ONLY:
- `mousemove` handler (line ~793)
- `touchmove` single-finger pan (line ~860)
- `touchmove` pinch zoom (line ~881)
- `wheel` handler (line ~775)

Keep direct `applyTransform()` calls in: `resetZoom()`, `zoomInBtn` click, `zoomOutBtn` click.

6. **Debounce URL sync.** Add:
```typescript
let urlTimeout: ReturnType<typeof setTimeout> | null = null;
function scheduleURLUpdate(): void {
  if (urlTimeout !== null) clearTimeout(urlTimeout);
  urlTimeout = setTimeout(updateURL, 300);
}
```
In `generatePuzzle()`, replace `updateURL()` (line ~292) with `scheduleURLUpdate()`.
  </action>
  <verify>
Run `npx tsc --noEmit` in web/ to verify TypeScript compiles without errors. Then run `npx vite build` in web/ to confirm production build succeeds.
  </verify>
  <done>
CSS has `will-change: transform` on #svg-container and `contain: layout style paint` on #svg-viewport. JS computes tab max inline without WASM call. SVG re-renders update path `d` attribute instead of full innerHTML. Pan/zoom transforms are rAF-throttled. URL sync is debounced at 300ms. TypeScript compiles and Vite builds successfully.
  </done>
</task>

<task type="auto">
  <name>Task 2: WASM -O3 + LTO build optimization</name>
  <files>crates/puzzle-wasm/Cargo.toml, Cargo.toml</files>
  <action>
1. In `crates/puzzle-wasm/Cargo.toml`, change `wasm-opt = ["-Os"]` to `wasm-opt = ["-O3"]` (optimize for speed instead of size).

2. In root `Cargo.toml`, add a release profile after the workspace members block:
```toml
[profile.release]
lto = true
codegen-units = 1
```

3. Rebuild WASM with the new settings:
```bash
cd crates/puzzle-wasm && wasm-pack build --target web --release --out-dir ../../web/pkg
```

4. Verify the WASM binary was produced and the web app still builds:
```bash
ls -la web/pkg/puzzle_wasm_bg.wasm
cd web && npx vite build
```
  </action>
  <verify>
`ls web/pkg/puzzle_wasm_bg.wasm` exists (file was rebuilt). `cd web && npx vite build` succeeds. Run the dev server briefly to confirm puzzle generation still works: `cd web && npx vite --host 0.0.0.0 &` then `sleep 3 && curl -s http://localhost:5173/ | head -5` shows HTML, then kill the server.
  </verify>
  <done>
WASM compiled with `-O3` instead of `-Os` and release profile has `lto = true` + `codegen-units = 1`. Binary exists in web/pkg/ and Vite build succeeds.
  </done>
</task>

</tasks>

<verification>
1. `cd web && npx tsc --noEmit` — TypeScript compiles clean
2. `cd web && npx vite build` — production build succeeds
3. `ls web/pkg/puzzle_wasm_bg.wasm` — WASM binary exists
4. Manual: Open dev server, verify puzzle generates, pan/zoom is smooth, sliders work, URL updates after 300ms pause
</verification>

<success_criteria>
- All 7 performance optimizations applied
- No TypeScript compilation errors
- Vite production build succeeds
- WASM binary rebuilt with -O3 and LTO
- No behavioral regressions (puzzle generation, download, URL sharing all work)
</success_criteria>

<output>
After completion, create `.planning/quick/14-buttery-smooth-ui-gpu-compositing-inline/14-SUMMARY.md`
</output>
