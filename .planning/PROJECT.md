# Puzzle Pattern Generator

## What This Is

A procedural jigsaw puzzle pattern generator that outputs SVG cut paths for laser cutting. Built as a Rust core compiled to WASM with a web-based GUI providing live preview. Users configure puzzle dimensions, piece counts, connector styles, and randomization, then export production-ready SVG files for laser cutting wood, acrylic, or other materials.

## Core Value

Generate geometrically valid, laser-cuttable jigsaw puzzle SVG patterns with configurable dimensions and procedural variation so no two puzzles are identical.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Configurable grid dimensions (rows x columns)
- [ ] Configurable puzzle size in inches or centimeters
- [ ] Classic knob connector shape with procedural randomization
- [ ] Seed-based randomization (reproducible puzzles via saved seeds)
- [ ] SVG export of cut paths
- [ ] Web GUI with live puzzle preview
- [ ] Rust core compiled to WASM for browser execution
- [ ] Custom border shapes (non-rectangular puzzle outlines)
- [ ] Whimsy/special pieces from preset shape library
- [ ] Whimsy pieces from user-imported SVG outlines
- [ ] Multi-piece whimsy (single shape spanning multiple pieces)
- [ ] Irregular edge puzzles (no straight border edges)
- [ ] No-edge puzzles (all connectors, no flat borders)
- [ ] All-edge puzzles (no connectors, all flat edges)
- [ ] Multiple connector types beyond classic knob

### Out of Scope

- DXF/PDF export — SVG sufficient for laser cutter workflows, revisit if needed
- Mobile app — web-based GUI covers all platforms
- Puzzle solving/simulation — this is a pattern generator, not a game
- Image overlay/printing — generates cut paths only, not puzzle artwork
- Commercial manufacturing die formats — focused on hobbyist laser cutting

## Context

- Target workflow: configure puzzle in browser -> preview live -> export SVG -> send to laser cutter (Glowforge, etc.)
- Rust chosen for performance (complex path generation) and memory safety, compiled to WASM for browser deployment
- Web frontend framework TBD — needs to pair well with WASM and canvas/SVG rendering for live preview
- Procedural generation uses controllable seeds so users can reproduce or share exact puzzle configurations
- Connector shapes need small per-piece variation while maintaining geometric validity (pieces must actually fit together)
- Whimsy pieces replace standard grid pieces with custom shapes — must integrate with surrounding connectors

## Constraints

- **Performance**: Live preview must update responsively as parameters change, even for larger puzzles
- **Geometric validity**: Every generated puzzle must be physically cuttable — no overlapping paths, no gaps, connectors must mate
- **SVG compatibility**: Output must work with common laser cutter software (Lightburn, Glowforge UI, etc.)
- **WASM size**: Compiled WASM bundle should be reasonable for web delivery

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + WASM | Performance for procedural generation + browser deployment | — Pending |
| Web GUI over native | Cross-platform, no install, pairs naturally with WASM | — Pending |
| SVG-only export | Laser cutters universally accept SVG, simplifies v1 | — Pending |
| Seed-based randomization | Reproducibility for users who want to share or recut puzzles | — Pending |
| Classic knob connector first | Most recognizable, prove the system before adding variants | — Pending |

---
*Last updated: 2026-03-01 after initialization*
