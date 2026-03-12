---
phase: 20-restyle-toggles
plan: 01
subsystem: web-gui
tags: [ui, toggles, icons, tab-max]
dependency_graph:
  requires: []
  provides: [pill-toggle-switches, inline-svg-icons, tab-max-20]
  affects: [web/index.html, web/src/style.css, web/src/main.ts]
tech_stack:
  added: []
  patterns: [inline-svg-icons, pill-toggle-switch, css-active-state-management]
key_files:
  created: []
  modified:
    - web/index.html
    - web/src/style.css
    - web/src/main.ts
decisions:
  - Inline SVG icons instead of emoji for lock and dice — cleaner monochrome aesthetic matching Web Awesome style
  - SVG shackle path d-attribute swap for lock/unlock state instead of emoji innerHTML swap
  - CSS active class toggled by JS on parent .pill-toggle label drives all visual state
metrics:
  completed: "2026-03-12"
  tasks_completed: 2
  tasks_total: 2
---

# Quick Task 20: Restyle Lock and Random Range Toggles as Pill Switches

Pill-shaped toggle switches with inline SVG icons (lock/dice) replacing emoji toggles; tab max capped at 20%

## Changes Made

### HTML (index.html)
- Replaced lock `<button>` elements with `<label class="pill-toggle">` wrapping a hidden checkbox, inline SVG lock icon, and pill track/knob structure
- Replaced randomize toggle `<label class="randomize-toggle">` with `<label class="pill-toggle pill-toggle-sm">` wrapping inline SVG dice icon and pill track/knob
- Wrapped parameter labels and their randomize toggles in `<span class="label-with-toggle">` to group them before the readout
- Changed tab slider `max="0.25"` to `max="0.20"` and default `value="0.25"` to `value="0.20"` (both tab and tab-max)
- Changed default tab readout text from "25%" to "20%"

### CSS (style.css)
- Removed old `.lock-toggle` styles (button-based) and `.randomize-toggle` / `.toggle-icon` styles
- Added `.pill-toggle` with inline-flex layout, hidden checkbox, and `.active` class-driven states
- Added `.pill-track` (26x14px pill shape with blue active color) and `.pill-knob` (10px sliding circle with left transition)
- Added `.pill-toggle-sm` variant (22x12px track, 8px knob) for randomize toggles next to parameter names
- Added `.label-with-toggle` for inline-flex grouping of label text + toggle
- SVG `.pill-icon` sized at 14px (12px for sm), opacity transitions from 0.4 to 0.85 on active
- Lock shackle path has CSS transform transition for subtle animation
- Changed `.slider-label` align-items from `baseline` to `center` for SVG alignment

### TypeScript (main.ts)
- Changed `gridLockBtn`/`dimsLockBtn` (HTMLElement) to `gridLockCheckbox`/`dimsLockCheckbox` (HTMLInputElement) — now references the checkbox inside the pill toggle
- Rewrote `toggleLock()` to work with checkbox + SVG: toggles `.active` class on parent `.pill-toggle`, swaps lock shackle SVG path `d` attribute between open and closed
- Changed lock event listeners from `click` on button to `change` on checkbox
- Added `.active` class toggle to parent `.pill-toggle` in `toggleRandomize()` function
- Changed tab max cap from `0.25` to `0.20` in `updateTabMax()`
- Changed `loadFromURL()` tab clamp from `Math.min(0.25, ...)` to `Math.min(0.20, ...)`
- Changed default tab URL param fallback from `"25"` to `"20"`
- Added `.pill-toggle.active` class restoration in `loadFromURL()` for both tab and taper randomize toggles

## Deviations from Plan

### Feedback-Driven Change

**1. [Rule 1 - Bug/UX] Replaced emoji icons with inline SVG icons**
- **Found during:** Checkpoint review (user feedback)
- **Issue:** Emoji icons (🔓/🔒/⚄) looked emoji-style; user wanted Web Awesome-style outlined icons
- **Fix:** Replaced all emoji with clean inline SVG icons — outlined padlock with animated shackle for lock toggles, outlined die face with 3 diagonal dots for randomize toggles
- **Files modified:** web/index.html, web/src/style.css, web/src/main.ts
- **Commit:** 1944631

## Verification

- `npm run build` succeeds with no errors
- All toggle switches render as pill-shaped with sliding knob
- Lock icons are clean outlined SVGs (monochrome, not emoji)
- Randomize icons positioned right after parameter names
- Tab size max is 20%, offset max scales accordingly (0.35 - tab_size)
- URL param round-trip preserves all toggle states including .active pill classes

## Commits

| # | Hash | Message |
|---|------|---------|
| 1 | 65f2c48 | feat(20-01): restyle lock and randomize toggles as pill switches, cap tab max at 20% |
| 2 | 1944631 | fix(20-01): replace emoji icons with inline SVG icons matching Web Awesome style |

## Self-Check: PASSED
