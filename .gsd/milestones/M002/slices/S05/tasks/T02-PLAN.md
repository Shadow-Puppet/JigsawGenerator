---
estimated_steps: 5
estimated_files: 3
---

# T02: Add sub-piece count UI input with Canvas rendering and URL persistence

**Slice:** S05 — Whimsy Sub-Puzzle Splitting
**Milestone:** M002

## Description

Add a numeric input for sub-piece count to the Whimsy Shape UI section, wire it to `buildConfig()` and URL params, and verify that sub-puzzle cut lines appear in Canvas preview. No new Canvas drawing code is needed — sub-puzzle edges flow through the existing `drawVisibleEdges()` and `drawBorder()` functions via the same Float64Array binary format.

**Relevant skills:** None specific — straightforward HTML + TypeScript wiring following existing patterns.

## Steps

1. **Rebuild WASM binary** to include T01 changes:
   - Run `wasm-pack build crates/puzzle-wasm --target web --out-dir pkg` from the project root
   - This builds to `crates/puzzle-wasm/pkg/` which is where Vite resolves the `puzzle-wasm` import (K010)
   - Verify the build succeeds with no errors

2. **Add sub-pieces numeric input to HTML** in `web/index.html`:
   - Inside the Whimsy Shape `<section>`, after the whimsy-info span, add:
     ```html
     <div class="control-row whimsy-sub-pieces" id="whimsy-sub-pieces-row" style="display:none">
       <label for="whimsy-sub-pieces">Sub-pieces</label>
       <input type="number" id="whimsy-sub-pieces" min="2" max="16" step="1" value="4" />
     </div>
     ```
   - The `style="display:none"` hides it by default — shown via JS when whimsy is active

3. **Wire sub-pieces state in TypeScript** in `web/src/main.ts`:
   - Add state variable: `let whimsySubPieces: number = 0;` (0 = disabled, 2-16 = active)
   - Add element reference: `let whimsySubPiecesInput: HTMLInputElement;` and `let whimsySubPiecesRow: HTMLElement;`
   - In the DOMContentLoaded handler, grab elements: `whimsySubPiecesInput = document.getElementById("whimsy-sub-pieces")! as HTMLInputElement;` and `whimsySubPiecesRow = document.getElementById("whimsy-sub-pieces-row")!;`
   - Add change event listener on `whimsySubPiecesInput`: parse value to int, clamp to [2,16], set `whimsySubPieces`, call `scheduleGenerate()`
   - Show/hide the sub-pieces row when whimsy is activated/deactivated:
     - When whimsy dropdown changes to a shape: `whimsySubPiecesRow.style.display = ""` (or "flex")
     - When whimsy is cleared (dropdown to "None" or Remove button): `whimsySubPiecesRow.style.display = "none"` and reset `whimsySubPieces = 0`
   - In `clearWhimsy()`: add `whimsySubPieces = 0; whimsySubPiecesRow.style.display = "none"; whimsySubPiecesInput.value = "4";`

4. **Wire into `buildConfig()` and URL params**:
   - In `buildConfig()`: when `whimsyShape && whimsySubPieces >= 2`, add `config.whimsy_sub_pieces = whimsySubPieces;`
   - In `updateUrl()`: when `whimsySubPieces >= 2`, add `params.set("wsp", whimsySubPieces.toString())`
   - In `restoreFromUrl()`: read `wsp` param, parse to int, set `whimsySubPieces` and `whimsySubPiecesInput.value`, show the sub-pieces row if whimsy is active

5. **Update piece count display**:
   - In the `generatePuzzle()` success handler, when whimsy is active and sub-pieces > 0, add sub-piece info to the extras array: `extras.push(\`${whimsySubPieces} sub-pieces\`)`
   - This produces text like "44 pieces (heart whimsy, 4 sub-pieces)"

## Must-Haves

- [ ] Numeric input for sub-piece count (2–16 range) in Whimsy Shape section
- [ ] Input hidden when no whimsy is active, shown when whimsy is selected
- [ ] Sub-piece count wired to `buildConfig()` as `whimsy_sub_pieces`
- [ ] URL param `wsp` persists and restores sub-piece count
- [ ] Sub-puzzle internal cut lines visible in Canvas when sub-pieces are set
- [ ] Piece count text includes sub-piece info
- [ ] `clearWhimsy()` resets sub-piece state

## Verification

- `wasm-pack build crates/puzzle-wasm --target web --out-dir pkg` — build succeeds
- Start dev server: `cd web && npm run dev`
- Browser: select heart whimsy → click to place → sub-pieces input appears → set to 4 → internal cut lines visible inside heart
- Browser: URL contains `wsp=4` → reload → state preserved → cut lines still visible
- Browser: set sub-pieces to 8 → more internal cut lines → download SVG → SVG file contains sub-puzzle paths (more M commands)
- Browser: click Remove → sub-pieces input disappears → cut lines gone → URL has no `wsp`

## Inputs

- `crates/puzzle-wasm/src/lib.rs` — T01 output: WASM endpoints now accept `whimsy_sub_pieces` and return sub-puzzle data in binary/SVG output
- `web/index.html` — existing Whimsy Shape section HTML
- `web/src/main.ts` — existing whimsy state, buildConfig(), updateUrl(), restoreFromUrl(), clearWhimsy(), generatePuzzle()

## Expected Output

- `web/index.html` — sub-pieces numeric input added to Whimsy Shape section
- `web/src/main.ts` — whimsySubPieces state, input wiring, buildConfig, URL params, piece count display, clearWhimsy update

## Observability Impact

- **URL param `wsp`**: Reflects sub-piece count when active (2-16). Absent when no sub-pieces or no whimsy. Inspect via browser URL bar.
- **Piece count text**: Shows `N sub-pieces` suffix when sub-pieces are active (e.g. "44 pieces (heart whimsy, 4 sub-pieces)"). Inspect via `#piece-count` element.
- **Sub-pieces input visibility**: `#whimsy-sub-pieces-row` display is `none` when whimsy inactive, `""` when active. Inspect via DOM or screenshot.
- **Failure visibility**: If `whimsy_sub_pieces` config field is missing or not wired, no sub-puzzle edges appear in Canvas and piece count text omits sub-piece info.
