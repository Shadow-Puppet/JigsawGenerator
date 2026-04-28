# 007 — Export & integration polish

**What:** Validate that SVG export is complete and geometrically correct across all config combinations, fix the download filename to include sub-piece count, and verify full URL round-trip with every param.
**Why:** Final-assembly slice for M002. Turns "probably works" into tests that prove it works across all the combinations the earlier slices produced.

## Must-haves

- **R009 (completeness) tests** cover five config combinations — rect / border only / whimsy only / both / whimsy+sub-pieces — asserting the SVG starts with `<svg`, has expected M-command count ranges, and any boundary or hole contour is closed with `Z`.
- **R010 (geometric correctness) tests** assert coordinates stay within puzzle bounds (with ~20% margin for connector overshoot), no degenerate subpaths (M immediately followed by M or Z), and sub-puzzle coordinates stay inside the whimsy bounding box.
- Download filename gains a sub-piece suffix when `whimsy_sub_pieces >= 2` (e.g. `-4sp`).
- Full URL round-trip including `wsp` manually verified in the browser: set all params → copy URL → reload → identical state.
- Existing test counts hold: 38+ WASM, 133+ core, zero regressions.

## Task split

1. **R009/R010 validation tests in puzzle-wasm** (~30m). ~8 new tests operating on SVG string content (no `js_sys::Reflect::get` — that path can't be tested natively).
2. **Filename + round-trip verification** (~20m). Update download handler (~line 1573 of `web/src/main.ts`), `npx tsc --noEmit` from `web/`, manual browser verification of the full-state URL round-trip.

## Verification

- `cargo test -p puzzle-wasm` and `cargo test -p puzzle-core` both green.
- `grep -q 'sub.*pieces\|SubPieces\|sp' web/src/main.ts` confirms the filename suffix lives in the download code.
- `npx tsc --noEmit` from `web/` compiles clean.
- Browser: full URL round-trip with `border=heart`, whimsy placed, sub-pieces set — reload from the copied URL produces an identical Canvas and piece count.

## Integration points

- Consumes everything from 003 through 006.
- Produces nothing for later work — M002 is complete after this slice.
