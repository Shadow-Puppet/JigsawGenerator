# T01: 04-web-gui-live-preview 01

**Slice:** S04 — **Milestone:** M001

## Description

Build the complete web GUI with parameter controls panel and live SVG preview.

Purpose: Replace the minimal demo page with a full-featured puzzle configuration interface where users can adjust all parameters and see the SVG update instantly.
Output: Working web app with controls panel, live SVG preview, and piece count display.

## Must-Haves

- [ ] "User sees a left controls panel (~300px) and right SVG preview area"
- [ ] "User can adjust rows/cols via number inputs, dimensions via number inputs with unit dropdown"
- [ ] "User can adjust tab size, jitter, corner radius, kerf via range sliders with number readout"
- [ ] "User can type a custom seed or click randomize to generate one"
- [ ] "SVG preview updates instantly on every parameter change"
- [ ] "Previous SVG stays visible while new one generates (no blank flash)"
- [ ] "Piece count breakdown displays below SVG preview"
- [ ] "On initial load, a puzzle generates with defaults + random seed"

## Files

- `web/index.html`
- `web/src/style.css`
- `web/src/main.ts`
