# 004 — Custom border UI

**What:** Wire a Rectangle/Heart/Star dropdown in the web UI through `buildConfig()` → WASM → Canvas, with URL persistence and accurate boundary-aware piece counts.
**Why:** Surface the S02 boundary engine to users. This is the first slice where a non-rectangular puzzle is visible in the browser.

## What shipped

- `<select id="border-shape">` in `web/index.html` with options Rectangle (default, empty-string value), Heart, Star.
- `buildConfig()` conditionally adds `border_shape` to the config JSON only when the selection is non-rectangular — keeps the JSON minimal and backward-compatible with older WASM builds.
- `piece_count` field added to `generate_edges_binary()`'s return object so the frontend can display the true count for boundary puzzles (`bp.included_cells().len()`), which is fewer than `rows × cols`. Rectangular puzzles still return `rows * cols`.
- Boundary puzzles show a simplified piece-count readout: "N pieces (shape border)" — the corner/edge/interior breakdown doesn't apply.
- URL param `border` read in `loadFromURL()` and written in `updateURL()`.
- Download filename includes the shape suffix (e.g. `puzzle-6x8-heart-seed-abc.svg`) when a non-rectangular border is active.
- `console.warn` fires if `piece_count` is missing from the WASM response — diagnostic for a stale WASM build vs JS mismatch.
- 26 puzzle-wasm tests passing at slice close (4 new via `generate_grid` JSON endpoint; binary WASM endpoints can't be tested natively — `js_sys` APIs panic outside the browser).

## Conventions for later slices

- **Rectangle = empty string = omit from config.** This is the pattern to follow for any other optional-shape field (whimsy shape selector in S04 should mirror this).
- **Frontend trusts WASM for piece_count.** Any slice that changes which cells are included (whimsy, sub-puzzles) must keep returning an accurate `piece_count` so the UI doesn't silently display `rows × cols`.
- **Filename suffix convention.** `puzzle-{rows}x{cols}-{shape?}-{whimsy?}-seed-{seed}.svg`. S04 and S05 will add their own suffix slots.
