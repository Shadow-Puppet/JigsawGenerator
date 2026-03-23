---
estimated_steps: 3
estimated_files: 1
---

# T02: Add sub-piece count to download filename and verify full URL round-trip

**Slice:** S06 — Export & Integration Polish
**Milestone:** M002

## Description

The download filename currently omits the sub-piece count — e.g., `puzzle-6x8-heart-whimsy-seed-abc.svg` even when 4 sub-pieces are active. Add a sub-piece suffix to the filename. Then build WASM, start the dev server, and browser-verify the filename and full URL round-trip with all params (including `wsp`).

**Relevant skills:** `agent-browser` (for browser verification)

## Steps

1. **Add sub-piece count suffix to download filename.** In `web/src/main.ts`, in the download button click handler (around line 1565–1578), after the `whimsySuffix` computation, add a sub-piece suffix. When `whimsySubPieces >= 2` and whimsy is active, append `-${whimsySubPieces}sp` to the filename. The resulting filename pattern becomes: `puzzle-6x8-heart-star-whimsy-4sp-seed-abc.svg`.

2. **Build WASM and verify TypeScript compiles.** Run:
   - `wasm-pack build crates/puzzle-wasm --target web --out-dir pkg` (builds to the correct directory per K010)
   - `cd web && npx tsc --noEmit` (TypeScript type-check)

3. **Browser verification.** Start the dev server (`cd web && npx vite --port 5173`), navigate to the app, and verify:
   - Set heart border, star whimsy (place it), 4 sub-pieces → download SVG → filename includes all suffixes (border, whimsy, sub-piece count)
   - Full URL round-trip: set all params (border=heart, whimsy=star with position/scale, sub-pieces=4) → copy URL → open in new tab → verify all state restored (border dropdown, whimsy shape, sub-piece input value, piece count text)
   - Piece count text shows correct format with all combinations active

## Must-Haves

- [ ] Download filename includes sub-piece count (e.g., `-4sp`) when whimsy sub-pieces are active
- [ ] TypeScript compiles without errors
- [ ] Browser confirms download filename correctness
- [ ] Browser confirms full URL round-trip with all params including `wsp`

## Verification

- `grep -q 'sp' web/src/main.ts` — sub-piece suffix code present in download handler (search near the filename construction)
- `cd web && npx tsc --noEmit` — TypeScript compiles clean
- Browser: download filename with all features active includes border, whimsy, and sub-piece suffixes

## Inputs

- `web/src/main.ts` — download handler at ~line 1565-1578 (filename construction), `updateURL()`/`loadFromURL()` for URL round-trip verification
- `crates/puzzle-wasm/src/lib.rs` — WASM binary must be rebuilt after T01 changes (T01 only adds tests, but rebuild ensures fresh binary)

## Expected Output

- `web/src/main.ts` — modified download handler with sub-piece count suffix in filename

## Key Context for Executor

- **Current filename code** (around line 1573):
  ```typescript
  const shapeSuffix = border ? `-${border}` : "";
  const whimsySuffix = whimsy ? `-${whimsy}-whimsy` : "";
  const filename = `puzzle-${config.rows}x${config.cols}${shapeSuffix}${whimsySuffix}-seed-${config.seed}.svg`;
  ```
- **Add sub-piece suffix** between `whimsySuffix` and `-seed-`:
  ```typescript
  const subSuffix = (whimsy && whimsySubPieces >= 2) ? `-${whimsySubPieces}sp` : "";
  const filename = `puzzle-${config.rows}x${config.cols}${shapeSuffix}${whimsySuffix}${subSuffix}-seed-${config.seed}.svg`;
  ```
- **`whimsySubPieces`** is a module-level `let` variable (number, starts at 0). It's ≥ 2 when sub-pieces are active.
- **WASM build command** (K010): `wasm-pack build crates/puzzle-wasm --target web --out-dir pkg` — builds to `crates/puzzle-wasm/pkg/` which is where Vite resolves the `puzzle-wasm` alias.
- **Dev server**: `cd web && npx vite --port 5173` — the app runs at `http://localhost:5173`
- **URL params to verify**: `rows`, `cols`, `w`, `h`, `seed`, `border`, `ws`, `wx`, `wy`, `wsc`, `wsp` — all should survive a reload.
