# Puzzle Pattern Generator

## What This Is

A procedural jigsaw puzzle pattern generator that outputs SVG cut paths for laser cutting. Built as a Rust core compiled to WASM with a web-based GUI providing live preview. Users configure puzzle dimensions, piece counts, connector styles, and randomization, then export production-ready SVG files for laser cutting wood, acrylic, or other materials. The generator currently produces standard rectangular puzzles with classic knob connectors and is being extended to support non-rectangular borders and whimsy pieces.

## Core Value

Generate geometrically valid, laser-cuttable jigsaw puzzle SVG patterns with configurable dimensions, procedural variation, custom border shapes, and whimsy pieces — so no two puzzles are identical and every cut path is physically correct.

## Current State

M001 complete. The generator produces rectangular puzzles with:
- Configurable grid (rows × cols), physical dimensions (mm/in), seed-based RNG
- Classic knob connectors with tab size, taper, jitter, and offset randomization
- Canvas 2D live preview with zoom/pan, viewport culling, touch support
- SVG export with laser-cutter compatible strokes and kerf compensation
- URL-based configuration sharing
- Smart constraint enforcement (piece size warnings, aspect ratio limits)
- Binary edge data pipeline (Rust → WASM → Float64Array → Canvas)

## Architecture / Key Patterns

- **Rust core** (`crates/puzzle-core/`): grid model, edge/piece types, connector generation via pluggable `ConnectorGenerator` trait, SVG export, binary export
- **WASM bridge** (`crates/puzzle-wasm/`): JSON config → Rust structs, binary edge data + cached SVG output
- **Web frontend** (`web/`): vanilla TypeScript, Canvas 2D rendering, Vite build with `vite-plugin-wasm`
- **Shared-edge model**: pieces reference edges by index into `h_edges`/`v_edges` arrays; edges stored once, shared between adjacent pieces
- **Deterministic RNG**: ChaCha8 seeded from user string via FNV-1a hash; separate RNG streams for grid construction and connector generation
- **kurbo** for 2D geometry (points, bezier curves, affine transforms, BezPath)

## Capability Contract

See `.gsd/REQUIREMENTS.md` for the explicit capability contract, requirement status, and coverage mapping.

## Milestone Sequence

- [x] M001: Migration — Core puzzle generator with rectangular grid, connectors, Canvas preview, SVG export
- [ ] M002: Whimsy & Custom Borders — Mask/reverse-mask system for non-rectangular borders and whimsy piece placement with sub-puzzle splitting (planned, not started)
