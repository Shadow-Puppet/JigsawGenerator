# Notes

Miscellaneous gotchas, open questions, and deferred items. Invariants and testing gotchas already captured in `CLAUDE.md` aren't repeated here.

## Still-relevant gotchas

- **linesweeper returns `Contours`, not `BezPath`.** `linesweeper::binary_op` output has per-contour `.path: BezPath` fields; to get a single usable path, iterate all contours and append their `PathEl` elements into a new `BezPath`. Wrapped inside `crates/puzzle-core/src/masking.rs`. If downstream code needs per-contour access (e.g. to count separate regions of a difference), call `binary_op` directly instead of using the wrappers.
- **`BezPath` has no built-in translate.** Manual `PathEl` iteration required to offset a path — roughly 15 lines per translation. A `translate_path(path, dx, dy) -> BezPath` helper would be worth extracting; currently duplicated in test code and boundary wiring.
- **Empty seed fallback.** WASM has no OS entropy (no `getrandom`), so an empty seed string defaults to `"default"`. The JS frontend is responsible for generating random seed strings when the user wants a fresh puzzle.

## Open architecture questions

- **Binary export format for whimsy edges.** The current fixed-stride format is 36 floats per edge (5 cubic bezier segments). Whimsy boundary contours and sub-puzzle edges won't fit this mould — they're variable-length. Open: extend fixed-stride with a shape-contour escape, or switch to a variable-length format with per-edge length prefixes. Fixed-stride plus a CMD-prefixed shape-contour encoding is what S02 actually shipped for border contours (`boundary_border_to_binary`); S04 will decide whether to follow that pattern or introduce variable-length proper.
- **Sub-puzzle subdivision strategy (S05).** Leaning toward grid-within-boundary (reuse `BoundaryPuzzle::new(sub_grid, whimsy_contour)` with an isolated RNG seed `"{seed}-whimsy-sub"`), because it reuses the existing connector system. Alternatives considered and not chosen: Voronoi subdivision, simple vertical/horizontal cuts.
- **Interactive drag performance.** Boolean ops during whimsy drag must be fast enough to feel responsive. Plan: draw the whimsy overlay on Canvas directly during drag (no WASM call), debounce actual regeneration via `scheduleGenerate()` rAF throttle, fire an immediate regeneration on mouseup. Unverified until S04 lands.
- **linesweeper edge cases.** v0.3.0 is "early beta." Unknown behaviour at tangent intersections, shapes touching the grid boundary exactly at a vertex, or very small intersection regions. No issues observed on heart/star through S03, but complex user-imported shapes (deferred) would need more testing.

## Deferred items (past v1)

- Multiple whimsies per puzzle + whimsy–whimsy intersection handling
- User-imported SVG outlines for borders or whimsies
- Multi-piece whimsy: single figural shape spanning multiple grid cells
- Snap-to-grid placement (explicitly rejected — freeform is the point)
- Arbitrary tessellations (hex, triangle, Penrose, Truchet, custom periodic/aperiodic)
- Additional connector styles (flat tabs, wavy, angular)
- Irregular / no-edge / all-edge border variants
- Laser-cutter presets (Glowforge, LightBurn, Epilog)
- DXF/PDF export
