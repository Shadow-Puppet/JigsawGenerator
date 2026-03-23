# M002: Whimsy & Custom Borders

**Vision:** Add a mask/reverse-mask system to the puzzle generator that enables non-rectangular puzzle borders and whimsy piece placement, using boolean path operations on kurbo BezPaths. The user's framing: a shape is a stencil — mask mode keeps the inside (borders, sub-puzzles), reverse-mask mode keeps the outside (whimsy placement).

## Success Criteria

- User can select a heart or star shape as the puzzle border and generate a valid puzzle inside it
- User can drag a whimsy shape onto the grid, position it freely, resize it, and see the grid adapt live
- User can split a whimsy into N sub-pieces with working connectors between them
- Exported SVG includes all new geometry and is valid for laser cutting
- Same seed + same config produces identical output (determinism preserved)

## Key Risks / Unknowns

- **linesweeper stability on complex shapes** — v0.3.0 beta; boolean ops may fail on edge cases with bezier curves
- **Interactive performance** — boolean ops during drag must be fast enough for responsive feel
- **Sub-puzzle subdivision** — generating pieces inside an arbitrary contour is harder than clipping a rectangular grid
- **Binary export format** — current fixed-stride format won't work for variable-segment edges

## Proof Strategy

- linesweeper WASM compatibility and basic correctness → retire in S01 by proving boolean ops compile to WASM and produce valid output for heart/star shapes
- Boundary-aware grid generation → retire in S02 by proving a non-rectangular puzzle generates with valid connectors and exports to SVG
- Interactive performance → retire in S04 by proving drag-and-drop regeneration stays responsive

## Verification Classes

- Contract verification: Rust unit tests for boolean ops, grid clipping, connector generation at boundaries; SVG output validation
- Integration verification: WASM endpoint produces correct binary data; Canvas renders all geometry; SVG export round-trip
- Operational verification: none (client-side only)
- UAT / human verification: visual inspection of generated puzzles; laser-cut test if user has access

## Milestone Definition of Done

This milestone is complete only when all are true:

- All 6 slices are complete with verification passing
- Shape library, mask/reverse-mask engine, and grid adaptation are wired end-to-end through WASM
- Custom border and whimsy placement both work in the browser with Canvas preview
- Sub-puzzle splitting generates valid connectors inside whimsy contours
- SVG export produces correct, complete cut paths for all new geometry
- Seed determinism holds across all new features
- Final integrated acceptance scenarios pass against the live browser app

## Requirement Coverage

- Covers: R001, R002, R003, R004, R005, R006, R007, R008, R009, R010, R011, R012, R013
- Partially covers: none
- Leaves for later: R014 (multiple whimsy), R015 (user SVG import), R016 (multi-piece whimsy)
- Orphan risks: none

## Slices

- [x] **S01: Shape Library & Boolean Op Foundation** `risk:high` `depends:[]`
  > After this: unit tests prove linesweeper compiles to WASM and boolean intersection/difference works on heart and star BezPaths; shapes defined as reusable kurbo paths in puzzle-core

- [ ] **S02: Boundary-Aware Grid Generation** `risk:high` `depends:[S01]`
  > After this: WASM endpoint generates a puzzle grid clipped to a non-rectangular boundary; SVG export shows a heart-shaped puzzle with valid connectors and correct edge trimming

- [ ] **S03: Custom Border UI** `risk:medium` `depends:[S02]`
  > After this: user selects a border shape from a dropdown in the web UI, Canvas preview shows the non-rectangular puzzle, and exported SVG has the correct outline

- [ ] **S04: Whimsy Drag-Drop & Grid Adaptation** `risk:medium` `depends:[S02]`
  > After this: user drags a heart/star onto the puzzle, positions and resizes it freely, grid edges adapt in real-time with whimsy boundary as smooth cut line — no tabs at boundary

- [ ] **S05: Whimsy Sub-Puzzle Splitting** `risk:medium` `depends:[S04]`
  > After this: user sets piece count for a placed whimsy, the whimsy splits into N sub-pieces with connectors, visible in Canvas preview

- [ ] **S06: Export & Integration Polish** `risk:low` `depends:[S03,S04,S05]`
  > After this: downloaded SVG includes all geometry (custom border + whimsy + sub-puzzle), URL params capture full state, piece count updates correctly, and round-trip verification confirms valid cut paths

## Boundary Map

### S01 → S02

Produces:
- `shapes.rs` → `heart_path() -> BezPath`, `star_path() -> BezPath` (preset shape library)
- `masking.rs` → `mask_grid(grid_path, shape_path) -> BezPath` (intersection), `unmask_grid(grid_path, shape_path) -> BezPath` (difference) — thin wrappers around linesweeper binary_op
- linesweeper dependency verified to compile on wasm32-unknown-unknown

Consumes:
- nothing (first slice)

### S01 → S03

Produces:
- Shape definitions consumable by the WASM layer for UI enumeration

Consumes:
- nothing (first slice)

### S01 → S04

Produces:
- Shape BezPaths for whimsy placement
- masking primitives for grid adaptation

Consumes:
- nothing (first slice)

### S02 → S03

Produces:
- `BoundaryGrid` or extended `PuzzleGrid` that generates inside an arbitrary boundary
- WASM endpoint accepting border shape parameter
- Binary export extended to handle clipped/boundary edges
- SVG export producing correct paths for non-rectangular puzzles

Consumes from S01:
- shape BezPaths, mask operation

### S02 → S04

Produces:
- Grid clipping engine (reused for whimsy — reverse direction)
- Extended binary export format

Consumes from S01:
- shape BezPaths, mask/unmask operations

### S02 → S05

Produces:
- Boundary-aware grid generation (reused for sub-puzzle inside whimsy contour)

Consumes from S01:
- mask operation

### S04 → S05

Produces:
- Whimsy placement state (shape, position, size) in puzzle config
- Grid with whimsy hole (reverse-masked)
- WASM endpoint accepting whimsy parameters

Consumes from S02:
- boundary-aware grid generation, extended binary export

### S05 → S06

Produces:
- Sub-puzzle pieces with connectors inside whimsy contour
- Extended WASM output with sub-puzzle edge data

Consumes from S04:
- whimsy placement state, grid with whimsy hole

### S03 → S06

Produces:
- Working custom border UI with shape selection
- Canvas rendering of non-rectangular puzzles

Consumes from S02:
- boundary-aware grid generation, WASM border endpoint

### S04 → S06

Produces:
- Working whimsy drag-drop UI
- Canvas rendering of grid with whimsy

Consumes from S02:
- grid clipping, binary export
