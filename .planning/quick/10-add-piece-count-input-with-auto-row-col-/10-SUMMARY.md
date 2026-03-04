---
phase: quick-10
plan: 01
subsystem: web-gui
tags: [ui, piece-count, auto-calc, warning]
dependency_graph:
  requires: []
  provides: [piece-count-input, auto-grid-calc, piece-size-warning]
  affects: [web-gui, grid-size-controls]
tech_stack:
  added: []
  patterns: [calcBestGrid-auto-calc, bidirectional-sync, piece-size-threshold-warning]
key_files:
  created: []
  modified:
    - web/index.html
    - web/src/main.ts
    - web/src/style.css
decisions:
  - No URL param for piece count — derived from rows*cols, so rows/cols in URL suffices
  - No circular update flag needed — calcBestGrid sets .value directly without dispatching events
  - Warning threshold at 10mm — pieces below this are impractical for laser cutting
metrics:
  duration: 2 min
  completed: "2026-03-04T21:57:00Z"
---

# Quick Task 10: Add Piece Count Input with Auto Row/Col Calculation

Piece count input with best-fit grid auto-calc (squarest pieces) and min piece size warning at <10mm threshold.

## What Was Done

### Task 1: Add piece count input with auto row/col calculation
**Commit:** `c559db3`

**HTML changes (web/index.html):**
- Added piece count input (`#piece-target`) above rows/cols row in Grid Size section
- Default value 48 matches default 6x8 grid
- Added warning paragraph (`#piece-size-warning`) below rows/cols row

**TypeScript changes (web/src/main.ts):**
- `calcBestGrid(target)`: Iterates rows 2..min(target,100), computes best cols via `Math.round(target/r)`, selects pair closest to target with squarest piece aspect ratio as tiebreaker
- `syncPieceCount()`: Sets piece count input to `rows * cols` whenever grid changes
- `checkPieceSize()`: Computes min piece dimension in mm (handles inch conversion), shows orange warning when < 10mm
- Wired piece count input to `calcBestGrid` → `syncPieceCount` → `checkPieceSize`
- Added `syncPieceCount()` and `checkPieceSize()` to rows/cols/width/height input handlers
- Added `checkPieceSize()` to unit change handler
- Called `syncPieceCount()` and `checkPieceSize()` on initial page load

**CSS changes (web/src/style.css):**
- `.piece-count-row` with full-width input styling
- `.piece-size-warning` in orange (#e67e22), hidden when empty via `:empty` pseudo-class

## Verification

- `npm run build` passes cleanly with no TypeScript errors
- Piece count defaults to 48 (6 * 8)
- Changing piece count auto-calculates closest rows/cols with squarest pieces
- Changing rows or cols updates piece count display
- Warning appears for small pieces (e.g., 297x210mm with 30x42 grid = ~7mm pieces)
- Warning disappears for adequate pieces (e.g., 297x210mm with 6x8 grid = ~35mm pieces)

## Deviations from Plan

None — plan executed exactly as written.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | c559db3 | feat(quick-10): add piece count input with auto row/col calculation |

## Self-Check: PASSED

All files verified present, all commits verified in git log.
