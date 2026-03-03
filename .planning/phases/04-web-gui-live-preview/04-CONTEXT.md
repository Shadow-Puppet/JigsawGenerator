# Phase 4: Web GUI & Live Preview - Context

**Gathered:** 2026-03-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can configure, preview, and export puzzles entirely in the browser through an intuitive web interface. The WASM engine is complete — it accepts a JSON config (rows, cols, width, height, unit, tab size, jitter, corner radius, seed, kerf width) and returns SVG. This phase replaces the minimal demo page with the full GUI, adds live preview, SVG download, and URL-based configuration sharing.

</domain>

<decisions>
## Implementation Decisions

### Page layout & visual style
- Left controls panel (~300px), right SVG preview area — classic tool layout
- Collapses to stacked layout on mobile (controls above, preview below)
- Minimal utility tool aesthetic: system fonts, subtle borders, light gray background — the SVG is the star, not the UI chrome
- Minimal header with title "Puzzle Pattern Generator" at top of controls panel only — no nav bar, no logo, no footer
- SVG preview fits to container (scales to fill available space), physical dimensions shown as text label

### Parameter controls design
- Grid size (rows, cols): number inputs only (no sliders) — users type the exact grid size they want, min/max constrained
- Physical dimensions (width, height): number inputs with a single mm/inches unit dropdown
- Tab size %, jitter amount, corner radius, kerf width: range sliders with visible number readout — these are "feel it out" parameters where visual feedback from the live preview matters
- Seed: text input showing current seed + a randomize (dice/shuffle) button to generate a random seed. Users can type a custom seed or click to randomize
- On initial page load (no URL params): generate a puzzle with sensible defaults and a random seed so the user sees something immediately

### Live preview behavior
- Instant SVG regeneration on every parameter change — no debounce, no manual refresh button. WASM is fast; if it lags on very large grids, debounce can be added later
- While generating: keep showing the previous SVG (don't flash blank). Optionally dim or show a subtle loading indicator
- Static fit-to-container display — no zoom or pan on the preview
- Piece count breakdown displayed below the SVG preview as a compact text line: "48 pieces (4 corner, 20 edge, 24 interior)"

### Download
- "Download SVG" button that saves the current SVG as a file
- Filename should include key params for identification (e.g., `puzzle-6x8-seed-abc123.svg`)

### URL sharing
- All puzzle parameters encoded as query params: `?rows=6&cols=8&w=297&h=210&unit=mm&tab=25&jitter=50&radius=2&kerf=0&seed=abc123`
- URL updates live via `history.replaceState` as user changes any parameter — no history spam, just always reflects current state
- A "Copy link" button copies the current URL to clipboard with brief "Copied!" confirmation toast/feedback
- Opening a shared URL auto-generates the puzzle immediately — parse params, populate controls, generate SVG on load

### Claude's Discretion
- Exact slider range bounds and step sizes for each parameter
- Specific Tailwind/CSS class choices and spacing
- Loading indicator style (dim, spinner, opacity)
- Download button placement (controls panel vs near preview)
- Mobile breakpoint and responsive details
- Error state display (invalid params, WASM failures)
- Whether to use any lightweight CSS framework or stay vanilla CSS
- Query param abbreviations (e.g., `w` vs `width`)

</decisions>

<specifics>
## Specific Ideas

- The existing page has a working vanilla CSS aesthetic (system fonts, clean inputs, blue accent color) that can be carried forward or evolved — no need to start from scratch
- The WASM `generate_svg(config_json)` function returns a complete SVG string ready for `innerHTML` injection or blob download
- JS must generate random seeds since WASM can't access OS entropy — existing code decision from earlier phases
- Piece breakdown data available via `compute_pieces(config_json)` or can be parsed from `generate_grid()` response

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-web-gui-live-preview*
*Context gathered: 2026-03-03*
