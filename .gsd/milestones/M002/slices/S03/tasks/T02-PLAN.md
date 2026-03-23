---
estimated_steps: 7
estimated_files: 2
---

# T02: Add border shape dropdown, wire config/URL, fix piece count display

**Slice:** S03 — Custom Border UI
**Milestone:** M002

## Description

Wire the border shape feature end-to-end in the web UI. After T01, the WASM `generate_edges_binary()` returns `piece_count` in its response. This task adds a `<select>` dropdown for border shape selection, passes the choice through `buildConfig()` to WASM, displays the correct piece count from the WASM response, persists the selection in URL params, and includes the shape in the SVG download filename.

The Canvas rendering already handles boundary puzzles — `drawBorder()` processes CMD_* commands including cubic beziers, and `drawVisibleEdges()` renders filtered edges. No Canvas code changes are needed.

**Relevant skill:** `frontend-design` — for dropdown styling consistency with existing controls.

## Steps

1. **Add dropdown to HTML** (`web/index.html`): Insert a new `<section class="control-section">` between the Dimensions and Parameters sections. Use the same styling pattern as the existing Unit select. Structure:
   ```html
   <section class="control-section">
     <h2 class="section-header">Border Shape</h2>
     <div class="control-row">
       <div class="input-group" style="flex:1">
         <select id="border-shape">
           <option value="">Rectangle</option>
           <option value="heart">Heart</option>
           <option value="star">Star</option>
         </select>
       </div>
     </div>
   </section>
   ```
   Note: The `value=""` for Rectangle means "no border_shape" — `buildConfig()` will omit the field when empty.

2. **Add DOM reference** (`web/src/main.ts`): Add `let borderShapeSelect: HTMLSelectElement;` in the DOM references section. In `main()`, add `borderShapeSelect = document.getElementById("border-shape") as HTMLSelectElement;`.

3. **Update `buildConfig()`** (`web/src/main.ts`): After building the config object, conditionally add `border_shape`:
   ```typescript
   const config: Record<string, unknown> = {
     rows: ..., cols: ..., // existing fields
   };
   const borderVal = borderShapeSelect.value;
   if (borderVal) {
     config.border_shape = borderVal;
   }
   return config;
   ```
   **Critical:** Do NOT include `border_shape` when the value is empty string (Rectangle). The Rust `PuzzleConfig` uses `#[serde(default)]` on `border_shape: Option<String>`, so omitting the field = `None` = rectangular puzzle. Sending `""` would cause an "Unknown border shape" error.

4. **Fix piece count display in `generatePuzzle()`** (`web/src/main.ts`): Replace the JS-computed breakdown block with:
   ```typescript
   const count = result.piece_count as number;
   const borderVal = borderShapeSelect.value;
   if (borderVal) {
     pieceCount.textContent = `${count} pieces (${borderVal} border)`;
   } else {
     const rows = parseInt(rowsInput.value, 10);
     const cols = parseInt(colsInput.value, 10);
     const corners = 4;
     const edges = 2 * (rows - 2) + 2 * (cols - 2);
     const interior = (rows - 2) * (cols - 2);
     pieceCount.textContent = `${count} pieces (${corners} corner, ${edges} edge, ${interior} interior)`;
   }
   ```
   For boundary puzzles the corner/edge/interior breakdown doesn't apply cleanly, so show a simpler format. For rectangular, keep the existing detailed breakdown but use the WASM count as the total (should match `rows * cols`).

5. **Update `loadFromURL()`** (`web/src/main.ts`): After existing param restoration, add:
   ```typescript
   const border = params.get("border") ?? "";
   borderShapeSelect.value = border;
   ```
   Place this after the `borderShapeSelect` DOM reference is guaranteed to exist (it's called from `main()` after DOM caching).

6. **Update `updateURL()`** (`web/src/main.ts`): Add border param persistence:
   ```typescript
   const borderVal = borderShapeSelect.value;
   if (borderVal) {
     params.set("border", borderVal);
   }
   ```

7. **Wire change event + update download filename** (`web/src/main.ts`):
   - In `main()` event wiring section, add: `borderShapeSelect.addEventListener("change", scheduleGenerate);`
   - In the download button handler, update the filename to include border shape when active:
     ```typescript
     const border = (buildConfig() as Record<string, unknown>).border_shape as string | undefined;
     const shapeSuffix = border ? `-${border}` : "";
     const filename = `puzzle-${config.rows}x${config.cols}${shapeSuffix}-seed-${config.seed}.svg`;
     ```

## Must-Haves

- [ ] `<select id="border-shape">` dropdown appears in the HTML with Rectangle (default), Heart, Star options
- [ ] `buildConfig()` includes `border_shape` only when a non-rectangular shape is selected (omits for empty/Rectangle)
- [ ] Piece count display uses `result.piece_count` from WASM response
- [ ] Boundary puzzles show simplified piece count (e.g. "28 pieces (heart border)") instead of incorrect corner/edge/interior breakdown
- [ ] Rectangular puzzles still show detailed breakdown (corners, edges, interior)
- [ ] URL param `border` persists the selection — reload restores the dropdown and puzzle renders correctly
- [ ] Download SVG filename includes shape name when border is active
- [ ] Selecting Heart/Star in the dropdown immediately regenerates the puzzle on Canvas

## Verification

- `grep -q 'id="border-shape"' web/index.html` — dropdown exists in HTML
- `grep -q 'borderShapeSelect' web/src/main.ts` — DOM reference exists
- `grep -q 'border_shape' web/src/main.ts` — config wiring exists
- `grep -q 'piece_count' web/src/main.ts` — WASM piece count consumed
- `grep -q '"border"' web/src/main.ts` — URL param name exists
- TypeScript compiles without errors (verified by `cd web && npx tsc --noEmit` or successful `npm run dev:wasm && npm run build`)

## Inputs

- `crates/puzzle-wasm/src/lib.rs` — T01's output: `generate_edges_binary()` now returns `piece_count` in the JS object
- `web/index.html` — existing HTML structure with controls panel
- `web/src/main.ts` — existing JS with `buildConfig()`, `generatePuzzle()`, `loadFromURL()`, `updateURL()`, event wiring

## Expected Output

- `web/index.html` — modified with border shape dropdown section
- `web/src/main.ts` — modified with border shape DOM reference, config wiring, piece count display fix, URL param sync, event wiring, download filename update

## Observability Impact

- **Piece count display now driven by WASM:** The frontend piece count text is sourced from `result.piece_count` (returned by `generate_edges_binary()`) rather than JS-computed `rows * cols`. If `piece_count` is missing (old WASM binary), a `console.warn` is emitted and the display falls back to `rows * cols` without the corner/edge/interior breakdown.
- **Border shape in URL:** The `border` URL param is present when a non-rectangular shape is active. Inspect `window.location.search` to confirm the param. Its absence when Rectangle is selected confirms the conditional omission logic works.
- **Simplified vs detailed piece count:** Boundary puzzles show `"N pieces (shape border)"` format; rectangular puzzles show `"N pieces (C corner, E edge, I interior)"`. If a boundary puzzle shows the detailed breakdown, the `borderShapeSelect.value` check is broken.
- **Download filename includes shape:** When downloading SVG with a border shape active, the filename includes the shape name (e.g., `puzzle-6x8-heart-seed-abc.svg`). If the filename lacks the shape suffix when a border is selected, inspect `buildConfig().border_shape`.
