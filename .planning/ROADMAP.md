# Roadmap: Puzzle Pattern Generator

**Created:** 2026-03-01
**Depth:** Standard
**Phases:** 4
**Coverage:** 16/16 v1 requirements mapped

## Phases

- [x] **Phase 1: Build Pipeline & WASM Foundation** - Rust→WASM→browser build pipeline with round-trip proof
- [x] **Phase 2: Grid Engine & Data Model** - Grid layout, shared-edge architecture, seeded RNG, pluggable connector trait
- [x] **Phase 3: Connector Generation & SVG Export** - Classic knob connectors, procedural variation, laser-cutter-compatible SVG output
- [x] **Phase 4: Web GUI & Live Preview** - Parameter controls, live SVG preview, SVG download, URL sharing

## Phase Details

### Phase 1: Build Pipeline & WASM Foundation
**Goal**: A working Rust→WASM→Vite build pipeline where TypeScript can call Rust functions and receive results in the browser
**Depends on**: Nothing (first phase)
**Requirements**: INFR-01
**Success Criteria** (what must be TRUE):
  1. User can run a single build command that compiles Rust to WASM and bundles the web app
  2. A TypeScript function can call a Rust/WASM function and display the returned value in the browser
  3. WASM bundle size is under 500KB gzipped with optimized release profile
**Plans:** 1 plan
Plans:
- [x] 01-01-PLAN.md — Rust workspace + WASM bindings + Vite web app + round-trip demo

### Phase 2: Grid Engine & Data Model
**Goal**: The engine computes geometrically valid grid layouts with shared-edge architecture, deterministic seeding, and configurable dimensions — the complete data model foundation
**Depends on**: Phase 1
**Requirements**: GRID-01, GRID-02, GRID-03, GRID-04, GRID-05, GRID-06, CONN-03, INFR-02
**Success Criteria** (what must be TRUE):
  1. Engine generates a grid of N×M cells with correct physical dimensions in mm or inches, where each internal edge exists exactly once in memory (shared-edge)
  2. Given the same seed value, the engine produces identical grid layouts and edge assignments across runs and platforms
  3. Changing tab size percentage or jitter amount produces visibly different edge parameters while maintaining geometric validity
  4. Engine reports accurate piece count breakdown (total, edge, corner, interior) for any grid configuration
  5. Connector generation uses a trait/interface that can be swapped without modifying grid or edge logic
**Plans:** 3 plans
Plans:
- [x] 02-01-PLAN.md — Foundation types, config, seed, edge types, connector trait
- [x] 02-02-PLAN.md — Grid engine with shared-edge model (TDD)
- [x] 02-03-PLAN.md — WASM boundary integration + end-to-end verification

### Phase 3: Connector Generation & SVG Export
**Goal**: The engine generates complete jigsaw puzzles with classic knob connectors and exports production-ready SVG files that work in laser cutter software
**Depends on**: Phase 2
**Requirements**: CONN-01, CONN-02, EXPT-01, EXPT-02
**Success Criteria** (what must be TRUE):
  1. Generated puzzle SVG shows classic knob connectors where each edge has visible procedural variation (different knob direction, control point jitter) while pieces still interlock
  2. Exported SVG opens correctly in LightBurn or equivalent laser cutter software with no parse errors, using absolute coordinates, inline stroke attributes, and explicit physical units
  3. User can apply kerf compensation and the resulting SVG paths are offset inward/outward by the specified amount, producing snug-fitting pieces when laser cut
  4. Generated SVG contains no overlapping paths, no gaps between adjacent pieces, and border pieces have rounded corners matching the configured radius
**Plans:** 2 plans
Plans:
- [x] 03-01-PLAN.md — ClassicKnobConnector implementation with TDD (knob shape, jitter, grid wiring)
- [x] 03-02-PLAN.md — SVG export pipeline with kerf compensation and WASM endpoint

### Phase 4: Web GUI & Live Preview
**Goal**: Users can configure, preview, and export puzzles entirely in the browser through an intuitive web interface
**Depends on**: Phase 3
**Requirements**: GUI-01, GUI-02, GUI-03
**Success Criteria** (what must be TRUE):
  1. User can adjust all puzzle parameters (grid size, dimensions, tab size, jitter, corner radius, seed, kerf) via sliders and input fields in the browser
  2. SVG preview updates live as the user changes any parameter, with no perceptible lag for puzzles up to 20×20
  3. User can download the generated SVG as a file ready for laser cutter import
  4. User can share a puzzle configuration via URL — opening the shared URL reproduces the exact same puzzle
**Plans:** 2 plans
Plans:
- [x] 04-01-PLAN.md — Controls panel + live SVG preview with instant regeneration
- [x] 04-02-PLAN.md — URL sharing, SVG download, copy link + visual verification

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Build Pipeline & WASM Foundation | 1/1 | Complete | 2026-03-02 |
| 2. Grid Engine & Data Model | 3/3 | Complete | 2026-03-03 |
| 3. Connector Generation & SVG Export | 2/2 | Complete | 2026-03-03 |
| 4. Web GUI & Live Preview | 2/2 | Complete | 2026-03-03 |

## Coverage Map

```
INFR-01 → Phase 1
GRID-01 → Phase 2
GRID-02 → Phase 2
GRID-03 → Phase 2
GRID-04 → Phase 2
GRID-05 → Phase 2
GRID-06 → Phase 2
CONN-03 → Phase 2
INFR-02 → Phase 2
CONN-01 → Phase 3
CONN-02 → Phase 3
EXPT-01 → Phase 3
EXPT-02 → Phase 3
GUI-01  → Phase 4
GUI-02  → Phase 4
GUI-03  → Phase 4

Mapped: 16/16 ✓
Orphaned: 0
```

---
*Roadmap created: 2026-03-01*
*Last updated: 2026-03-03T22:55:47Z*
