# S03: Custom Border UI — UAT Script

## Preconditions

- WASM is built: `cd web && npm run dev:wasm` succeeds
- Dev server running: `cd web && npm run dev` → opens at `http://localhost:5173`
- Browser with DevTools console accessible

---

## Test 1: Border Shape Dropdown Exists with Correct Options

**Steps:**
1. Open `http://localhost:5173`
2. Locate the border shape dropdown in the controls panel

**Expected:**
- A `<select>` element with id `border-shape` is visible
- Three options: "Rectangle" (selected by default), "Heart", "Star"
- Rectangle option has an empty string value

---

## Test 2: Heart Border Generates Fewer Pieces

**Steps:**
1. Set grid to 6 columns × 8 rows (or any size)
2. Note the piece count displayed (should show 48 = 6×8)
3. Select "Heart" from the border shape dropdown
4. Wait for Canvas to re-render

**Expected:**
- Canvas shows a heart-shaped puzzle outline (not rectangular)
- Piece count display shows fewer pieces than 48 (e.g., "32 pieces (heart border)")
- The piece count format is simplified — no corner/edge/interior breakdown
- Internal grid lines are visible only inside the heart shape

---

## Test 3: Star Border Generates Correctly

**Steps:**
1. Select "Star" from the border shape dropdown
2. Wait for Canvas to re-render

**Expected:**
- Canvas shows a star-shaped puzzle outline with 5 points
- Piece count is fewer than `rows × cols`
- Display shows "N pieces (star border)"

---

## Test 4: Rectangle Restores Full Grid

**Steps:**
1. With Heart or Star selected, switch back to "Rectangle"
2. Wait for Canvas to re-render

**Expected:**
- Canvas shows the standard rectangular puzzle
- Piece count returns to `rows × cols`
- Piece count display shows the full breakdown (N pieces: X corner, Y edge, Z interior)

---

## Test 5: URL Param Persistence

**Steps:**
1. Select "Heart" from the dropdown
2. Check the browser URL bar

**Expected:**
- URL contains `border=heart` parameter

**Steps (continued):**
3. Copy the URL and open it in a new tab (or reload the page)

**Expected:**
- Heart is pre-selected in the dropdown
- Canvas renders the heart-shaped puzzle
- Piece count is correct (fewer than rows × cols)

---

## Test 6: Rectangle Does Not Add URL Param

**Steps:**
1. Select "Rectangle" (default) from the dropdown
2. Check the browser URL bar

**Expected:**
- URL does NOT contain a `border` parameter (omitted when default)

---

## Test 7: SVG Download Includes Border Shape in Filename

**Steps:**
1. Select "Heart" from the dropdown
2. Click the Download SVG button

**Expected:**
- Downloaded filename contains "heart" (e.g., `puzzle-6x8-heart-seed-abc.svg`)
- SVG file contains heart-shaped border path (cubic bezier curves)

**Steps (continued):**
3. Switch to "Rectangle" and download again

**Expected:**
- Downloaded filename does NOT contain a shape suffix

---

## Test 8: piece_count Diagnostic in Console

**Steps:**
1. Open browser DevTools console
2. Select "Heart" from the dropdown
3. Observe console output during generation

**Expected:**
- No `console.warn` about missing `piece_count` (WASM binary is up to date)
- If you inspect the WASM result object, `result.piece_count` is present as a number and is less than `rows * cols`

---

## Test 9: Invalid Border Shape Error Path

**Steps:**
1. In the browser DevTools console, manually call:
   ```js
   // Get WASM module reference and call with invalid border_shape
   ```
   Or inspect `crates/puzzle-wasm/src/lib.rs` test `test_border_shape_invalid_returns_error`

**Expected:**
- Invalid `border_shape` values return `{ error: "Unknown border shape: '...'" }`
- The error is surfaced, not silently swallowed

---

## Test 10: Seed Determinism with Border Shape

**Steps:**
1. Set seed to "test123", select "Heart", note the piece count and visual layout
2. Reload the page (URL should restore all settings)
3. Compare piece count and visual layout

**Expected:**
- Identical piece count
- Identical puzzle layout (same connectors, same included cells)
- Canvas rendering matches pixel-for-pixel

---

## Edge Cases

### EC1: Rapid Shape Switching
- Quickly toggle between Rectangle → Heart → Star → Rectangle
- Expected: No errors, each renders correctly, no stale renders

### EC2: Small Grid with Border
- Set 2×2 grid, select "Heart"
- Expected: Very few pieces (possibly 0-2 inside heart). Display still shows valid count. No crash.

### EC3: Large Grid with Border
- Set 20×20 grid, select "Star"
- Expected: Renders without noticeable delay. Piece count is accurate.
