# M002: Whimsy & Custom Borders

**Gathered:** 2026-03-21
**Status:** Ready for planning

## Project Description

Extend the puzzle generator with a mask/reverse-mask system that enables non-rectangular puzzle borders and whimsy piece placement. An SVG shape acts as either a mask (keep grid lines inside → custom borders, sub-puzzles) or a reverse mask (remove grid lines inside → whimsy hole in grid). Both operations share the same geometric engine: boolean path operations on kurbo BezPaths via the linesweeper crate.

## Why This Milestone

The generator currently only produces rectangular puzzles. Real jigsaw puzzles often have custom outlines and special-shaped "whimsy" pieces. This milestone adds both capabilities using a unified geometric approach — the mask/reverse-mask framing that the user identified as the core abstraction.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Select a non-rectangular border shape (heart, star) and generate a puzzle that fills that shape
- Drag a whimsy shape onto the puzzle grid, position it freely, resize it, and see the grid adapt in real-time
- Split a placed whimsy into N sub-pieces with connectors
- Export an SVG with all new geometry (custom border + whimsy + sub-puzzle cuts) that is valid for laser cutting

### Entry point / environment

- Entry point: Same web GUI at localhost (Vite dev server)
- Environment: Browser with WASM
- Live dependencies involved: none

## Completion Class

- Contract complete means: unit tests prove boolean path ops produce valid geometry; exported SVGs contain correct paths; seed determinism holds
- Integration complete means: WASM endpoint accepts whimsy/border config, generates correct binary edge data, Canvas renders all new geometry, SVG export includes everything
- Operational complete means: none (client-side only)

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- A heart-shaped border puzzle generates, renders in Canvas, and exports as a valid SVG with no overlapping or missing cut paths
- A whimsy piece (star) placed on a rectangular puzzle produces correct grid trimming, renders with the whimsy boundary visible, and exports correctly
- A whimsy with sub-puzzle splitting (3 pieces) shows internal connectors and exports with all cuts
- Seed determinism: same seed + same whimsy config produces identical SVG output across regenerations

## Risks and Unknowns

- **linesweeper maturity** — v0.3.0, "early beta state." Boolean ops on complex bezier paths may have edge cases. Mitigation: start with simple shapes, validate thoroughly.
- **WASM compatibility** — linesweeper's dependency tree is pure Rust but untested on wasm32-unknown-unknown. Need to verify compilation early.
- **Performance of boolean ops** — interactive regeneration during drag requires the boolean op to complete in <16ms for 60fps feel. May need incremental updates or debouncing.
- **Sub-puzzle subdivision** — generating a grid inside an arbitrary contour is geometrically harder than clipping a rectangular grid. The adaptive grid approach (varying piece count to fill the shape) needs a clear algorithm.
- **Binary export format** — current format assumes 36-float stride per edge (5 cubic segments). Whimsy boundary edges and clipped grid edges won't follow this pattern. Format needs extension.

## Existing Codebase / Prior Art

- `crates/puzzle-core/src/grid.rs` — `PuzzleGrid` with h_edges/v_edges shared-edge model, `generate_connectors()`, piece classification
- `crates/puzzle-core/src/connector.rs` — `ConnectorGenerator` trait, pluggable connector generation
- `crates/puzzle-core/src/classic_connector.rs` — Classic knob connector with 5 cubic bezier segments
- `crates/puzzle-core/src/svg_export.rs` — SVG generation with border path + connector paths, `edge_transform()` for local→global coords
- `crates/puzzle-core/src/binary_export.rs` — Binary edge data for Canvas rendering, 36-float stride per edge
- `crates/puzzle-core/src/config.rs` — `PuzzleConfig` with validation, `TabConfig` with randomization
- `crates/puzzle-wasm/src/lib.rs` — WASM endpoints: `generate_edges_binary()`, `generate_svg()`, `get_cached_svg()`
- `web/src/main.ts` — Vanilla TS frontend, Canvas 2D rendering with zoom/pan, all event wiring
- `web/src/style.css` — Layout with controls panel + preview area

> See `.gsd/DECISIONS.md` for all architectural and pattern decisions — it is an append-only register; read it during planning, append to it during execution.

## Relevant Requirements

- R001 — Shape library (foundation for all new geometry)
- R002, R004 — Mask/reverse-mask operations (core geometric engine)
- R003 — Custom border (first mask application)
- R005-R007 — Whimsy placement and grid adaptation (reverse-mask application)
- R008 — Sub-puzzle splitting (mask inside whimsy contour)
- R009-R010 — Export and geometric correctness (launchability)
- R011 — Responsive interaction (quality bar)
- R013 — Seed determinism preservation

## Scope

### In Scope

- Shape library with heart and star as kurbo BezPaths
- Boolean path operations via linesweeper (intersection, difference)
- Custom border shape selection and grid generation inside arbitrary boundary
- Whimsy drag-and-drop placement with free positioning
- Whimsy resize with uniform scaling
- Grid edge trimming at whimsy boundary (boundary = smooth cut line, no tabs)
- Sub-puzzle splitting with user-controlled piece count
- Canvas rendering of all new geometry
- SVG export including all new geometry
- Seed determinism for new features
- One whimsy per puzzle (v1)

### Out of Scope / Non-Goals

- Multiple whimsy pieces per puzzle (R014 — deferred)
- User-imported SVG shapes (R015 — deferred)
- Multi-piece whimsy spanning grid cells (R016 — deferred)
- Snap-to-grid placement (R017 — explicitly rejected)
- DXF/PDF export formats

## Technical Constraints

- linesweeper must compile to wasm32-unknown-unknown (verified: pure Rust deps)
- Boolean ops must be fast enough for interactive preview (<50ms for typical puzzle sizes)
- Binary export format must be extended to handle variable-segment edges
- Existing seed determinism must not break — new RNG streams isolated from existing ones

## Integration Points

- **linesweeper** — boolean path ops on kurbo BezPaths (new dependency for puzzle-core)
- **kurbo** — existing dependency, used for BezPath shape definitions and transforms
- **WASM bridge** — new endpoints or extended config for whimsy/border parameters
- **Canvas rendering** — new draw paths for whimsy boundary, sub-puzzle cuts, clipped edges
- **URL params** — new params for border shape, whimsy position/size/shape/sub-pieces

## Open Questions

- **Sub-puzzle subdivision strategy** — how to split an arbitrary contour into N pieces? Options: grid-within-boundary (reuse mask), voronoi subdivision, or simple vertical/horizontal cuts. Leaning toward grid-within-boundary since it reuses the mask operation and the existing connector system.
- **linesweeper edge cases** — how does it handle tangent intersections, shapes touching the grid boundary exactly at a vertex, or very small intersection regions? Need to test during S01.
- **Binary format extension** — should we extend the fixed-stride format or switch to a variable-length format? The variable-length format is cleaner but requires more JS-side parsing.
