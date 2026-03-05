---
phase: quick-012
plan: 01
completed: 2026-03-05
duration: "3 min"
tasks_completed: 2
tasks_total: 2
key_files:
  modified:
    - web/index.html
    - web/src/main.ts
    - web/src/style.css
decisions:
  - "Lock buttons use Unicode padlock icons (open/closed) — lightweight, no icon library"
  - "enforceConstraints(source) replaces checkPieceSize() — single function handles both directions"
  - "Grid ratio violations always warn (grid problem, not dims) even when dims unlocked"
  - "Auto-adjust scales dimensions UP for small pieces but reduces grid DOWN — preserves user intent"
---

# Quick Task 12: Auto-adjust dimensions/grid with lock/unlock toggles

**One-liner:** Lock/unlock toggles on Grid Size and Dimensions sections; when unlocked, automatically adjusts the other to prevent <10mm piece sizes and >5:1 grid ratios.

## Changes Made

### Task 1: Lock toggle buttons (f07a69c)
- Added lock/unlock toggle buttons to Grid Size and Dimensions section headers
- Unicode padlock icons: &#128275; (unlocked/open) and &#128274; (locked/closed)
- CSS styling: unlocked = muted gray, locked = accent blue, hover states

### Task 2: Auto-adjust logic (fac44e9)
- Replaced `checkPieceSize()` with `enforceConstraints(source: 'grid' | 'dims')`
- **Grid changes (source='grid'):** When dimensions unlocked, auto-scales width/height up so smallest piece >= 10mm
- **Dimension changes (source='dims'):** When grid unlocked, auto-reduces rows/cols so pieces >= 10mm, and clamps grid ratio to 5:1
- **Locked behavior:** Shows warning with hint to unlock the other section
- Split event listeners: grid inputs call `enforceConstraints('grid')`, dimension inputs call `enforceConstraints('dims')`
- Piece count and tab max synced after auto-adjustments

## Deviations from Plan

None - plan executed exactly as written.

## Verification

- Vite build passes cleanly
- All existing functionality preserved (URL sync, randomize, unit conversion)

## Self-Check: PASSED
