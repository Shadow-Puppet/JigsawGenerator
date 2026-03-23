# Requirements

This file is the explicit capability and coverage contract for the project.

## Active

### R001 — Shape library with preset SVG shapes
- Class: core-capability
- Status: active
- Description: Heart and star shapes defined as kurbo BezPaths in the Rust core, available for both border and whimsy use
- Why it matters: Foundation for all non-rectangular geometry — both custom borders and whimsy pieces consume shapes from this library
- Source: user
- Primary owning slice: M002/S01
- Supporting slices: M002/S02, M002/S03, M002/S04
- Validation: unmapped
- Notes: Start with heart + star; library is extensible for future shapes

### R002 — Mask operation — generate puzzle grid inside arbitrary boundary
- Class: core-capability
- Status: active
- Description: Given a closed BezPath boundary, generate a puzzle grid where edges are clipped to the boundary and pieces outside are removed. Grid adapts piece count to fill the shape naturally.
- Why it matters: Core geometric primitive that enables both custom borders (mask mode) and whimsy sub-puzzles (mask mode inside whimsy contour)
- Source: user
- Primary owning slice: M002/S02
- Supporting slices: M002/S03, M002/S05
- Validation: unmapped
- Notes: Uses linesweeper for boolean path ops on kurbo BezPaths

### R003 — Custom border — user selects non-rectangular puzzle outline
- Class: primary-user-loop
- Status: active
- Description: User picks a shape from the library as the puzzle's outer boundary. The grid generates inside that shape with adapted piece count.
- Why it matters: First user-visible feature that uses the mask operation — transforms the rectangular puzzle into arbitrary shapes
- Source: user
- Primary owning slice: M002/S03
- Supporting slices: none
- Validation: unmapped
- Notes: Shape selection via UI dropdown; preview updates live

### R004 — Reverse-mask operation — remove grid edges inside a shape
- Class: core-capability
- Status: active
- Description: Given a placed whimsy shape, remove all grid edges that fall inside it. Grid edges terminate at the whimsy boundary. The whimsy boundary itself is the cut line — no tab connectors at the boundary.
- Why it matters: Core geometric primitive for whimsy placement — creates the "hole" in the grid where the whimsy piece sits
- Source: user
- Primary owning slice: M002/S02
- Supporting slices: M002/S04
- Validation: unmapped
- Notes: Same boolean op engine as mask, just keeping the opposite side

### R005 — Whimsy placement — drag-and-drop shape onto grid
- Class: primary-user-loop
- Status: active
- Description: User drags a shape from the library onto the puzzle grid preview, positioning it freely anywhere (no grid snap)
- Why it matters: Primary interaction for placing whimsy pieces — must feel responsive and natural
- Source: user
- Primary owning slice: M002/S04
- Supporting slices: none
- Validation: unmapped
- Notes: Free-form positioning; one whimsy at a time for v1

### R006 — Whimsy resize — user can scale placed whimsy shape
- Class: primary-user-loop
- Status: active
- Description: After placing a whimsy shape, user can resize it by dragging handles or using a control. Grid adapts in real-time.
- Why it matters: Users need to control how much of the puzzle the whimsy covers
- Source: user
- Primary owning slice: M002/S04
- Supporting slices: none
- Validation: unmapped
- Notes: Scale uniformly to preserve shape proportions

### R007 — Grid adaptation — grid edges terminate at whimsy boundary
- Class: core-capability
- Status: active
- Description: When a whimsy shape is placed, grid edges inside the shape are removed. Edges crossing the boundary are trimmed. The whimsy outline becomes a smooth cut line with no tab connectors.
- Why it matters: Geometric correctness — pieces adjacent to the whimsy must have valid cut paths that follow the whimsy contour
- Source: user
- Primary owning slice: M002/S04
- Supporting slices: M002/S06
- Validation: unmapped
- Notes: Whimsy boundary = cut line, no tabs. Connectors only on grid-to-grid edges.

### R008 — Sub-puzzle splitting — user picks piece count for whimsy interior
- Class: core-capability
- Status: active
- Description: User sets how many sub-pieces the whimsy splits into (2, 3, 4...). The whimsy interior is subdivided using the same connector generation as the parent puzzle.
- Why it matters: Turns the whimsy from a single solid piece into a mini-puzzle — same mask operation applied inside the whimsy contour
- Source: user
- Primary owning slice: M002/S05
- Supporting slices: M002/S06
- Validation: unmapped
- Notes: Reuses mask operation + connector generation inside whimsy boundary

### R009 — SVG export includes all new geometry
- Class: launchability
- Status: active
- Description: Downloaded SVG contains custom border contour, whimsy cut lines, sub-puzzle internal cuts, and all modified grid edges — complete and valid for laser cutting
- Why it matters: The entire point — if the SVG isn't correct, the feature doesn't ship
- Source: user
- Primary owning slice: M002/S06
- Supporting slices: none
- Validation: unmapped
- Notes: Must work with existing laser cutter software (Lightburn, Glowforge UI)

### R010 — Geometric correctness — all paths valid for laser cutting
- Class: quality-attribute
- Status: active
- Description: No overlapping paths, no gaps, no self-intersecting cuts. Every generated piece is physically cuttable. Connectors mate correctly.
- Why it matters: Broken geometry wastes material and time on the laser cutter — this is non-negotiable
- Source: user
- Primary owning slice: M002/S06
- Supporting slices: M002/S02, M002/S04, M002/S05
- Validation: unmapped
- Notes: Both geometric correctness AND interaction quality are equally important to the user

### R011 — Responsive interaction — preview updates in real-time
- Class: quality-attribute
- Status: active
- Description: During whimsy drag/resize and border selection, the Canvas preview updates responsively without perceptible lag
- Why it matters: Clunky interaction is as disappointing as broken geometry per user's explicit emphasis
- Source: user
- Primary owning slice: M002/S04
- Supporting slices: M002/S03
- Validation: unmapped
- Notes: WASM generation must be fast enough for interactive use; may need incremental updates

### R012 — One whimsy per puzzle (v1 constraint)
- Class: constraint
- Status: active
- Description: Only one whimsy shape can be placed on a puzzle at a time
- Why it matters: Avoids whimsy-whimsy intersection complexity; simplifies UI and geometry
- Source: user
- Primary owning slice: M002/S04
- Supporting slices: none
- Validation: unmapped
- Notes: Multiple whimsy deferred to R014

### R013 — Seed determinism preserved across new features
- Class: quality-attribute
- Status: active
- Description: Same seed + same whimsy config + same border = identical output. Determinism extends to all new geometry.
- Why it matters: Users share and reproduce puzzles via seeds — breaking determinism breaks a validated capability
- Source: inferred
- Primary owning slice: M002/S02
- Supporting slices: M002/S04, M002/S05
- Validation: unmapped
- Notes: New RNG streams for whimsy/border operations must be isolated from existing grid/connector RNG

## Validated

### GRID-01 — User can configure puzzle grid as rows x columns
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### GRID-02 — User can set puzzle physical size in millimeters or inches
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### GRID-03 — User can control tab/knob size as percentage of edge length
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### GRID-04 — User can control jitter/randomness amount per edge
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### GRID-05 — User can set rounded corner radius on puzzle border
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### GRID-06 — User can see piece count breakdown (total, edge, corner, interior)
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### CONN-01 — Puzzle generates classic knob connector shapes
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### CONN-02 — Each edge is procedurally varied
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### CONN-03 — User can set a seed value to reproduce exact puzzle configurations
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### EXPT-01 — User can export puzzle as SVG
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### EXPT-02 — User can apply kerf compensation
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### GUI-01 — User can configure all parameters via web-based controls
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### GUI-02 — User sees live SVG preview that updates as parameters change
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### GUI-03 — User can share puzzle configuration via URL
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### INFR-01 — Puzzle generation runs in Rust compiled to WASM
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

### INFR-02 — Connector generation uses pluggable trait/interface
- Class: core-capability
- Status: validated
- Source: inferred
- Primary owning slice: M001
- Validation: validated

## Deferred

### R014 — Multiple whimsy pieces per puzzle
- Class: core-capability
- Status: deferred
- Description: Place multiple whimsy shapes on the same puzzle, including whimsy-whimsy intersection handling
- Why it matters: Natural extension once single whimsy is solid
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: Deferred to avoid whimsy-whimsy intersection complexity in v1

### R015 — User-imported SVG outlines for whimsy shapes
- Class: core-capability
- Status: deferred
- Description: User uploads their own SVG file to use as a whimsy or border shape
- Why it matters: Unlocks infinite shape variety beyond the preset library
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: Requires SVG parsing, path simplification, and validation

### R016 — Multi-piece whimsy spanning multiple grid cells
- Class: core-capability
- Status: deferred
- Description: A single whimsy shape that spans multiple grid pieces, with each piece containing part of the whimsy outline
- Why it matters: Creates more complex and interesting puzzle designs
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: Different from sub-puzzle splitting; this is about the whimsy shape appearing across piece boundaries

## Out of Scope

### R017 — Whimsy snap-to-grid
- Class: anti-feature
- Status: out-of-scope
- Description: Snapping whimsy placement to grid cell boundaries
- Why it matters: User explicitly chose free-form placement over grid snapping
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: User wants free-form anywhere placement

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| R001 | core-capability | active | M002/S01 | M002/S02,S03,S04 | unmapped |
| R002 | core-capability | active | M002/S02 | M002/S03,S05 | unmapped |
| R003 | primary-user-loop | active | M002/S03 | none | unmapped |
| R004 | core-capability | active | M002/S02 | M002/S04 | unmapped |
| R005 | primary-user-loop | active | M002/S04 | none | unmapped |
| R006 | primary-user-loop | active | M002/S04 | none | unmapped |
| R007 | core-capability | active | M002/S04 | M002/S06 | unmapped |
| R008 | core-capability | active | M002/S05 | M002/S06 | unmapped |
| R009 | launchability | active | M002/S06 | none | unmapped |
| R010 | quality-attribute | active | M002/S06 | M002/S02,S04,S05 | unmapped |
| R011 | quality-attribute | active | M002/S04 | M002/S03 | unmapped |
| R012 | constraint | active | M002/S04 | none | unmapped |
| R013 | quality-attribute | active | M002/S02 | M002/S04,S05 | unmapped |
| R014 | core-capability | deferred | none | none | unmapped |
| R015 | core-capability | deferred | none | none | unmapped |
| R016 | core-capability | deferred | none | none | unmapped |
| R017 | anti-feature | out-of-scope | none | none | n/a |

## Coverage Summary

- Active requirements: 13
- Mapped to slices: 13
- Validated: 0 (M002 not started)
- Unmapped active requirements: 0
