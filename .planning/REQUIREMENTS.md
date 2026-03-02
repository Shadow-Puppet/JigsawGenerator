# Requirements: Puzzle Pattern Generator

**Defined:** 2026-03-01
**Core Value:** Generate geometrically valid, laser-cuttable jigsaw puzzle SVG patterns with configurable dimensions and procedural variation so no two puzzles are identical.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Grid & Dimensions

- [ ] **GRID-01**: User can configure puzzle grid as rows x columns
- [ ] **GRID-02**: User can set puzzle physical size in millimeters or inches
- [ ] **GRID-03**: User can control tab/knob size as percentage of edge length
- [ ] **GRID-04**: User can control jitter/randomness amount per edge
- [ ] **GRID-05**: User can set rounded corner radius on puzzle border
- [ ] **GRID-06**: User can see piece count breakdown (total, edge, corner, interior)

### Connectors

- [ ] **CONN-01**: Puzzle generates classic knob connector shapes using cubic bezier curves
- [ ] **CONN-02**: Each edge is procedurally varied (random direction, control point jitter)
- [ ] **CONN-03**: User can set a seed value to reproduce exact puzzle configurations

### Export

- [ ] **EXPT-01**: User can export puzzle as SVG with laser-cutter compatible strokes
- [ ] **EXPT-02**: User can apply kerf compensation to adjust path offsets for snug piece fit

### GUI & Preview

- [ ] **GUI-01**: User can configure all parameters via web-based controls (sliders, inputs)
- [ ] **GUI-02**: User sees live SVG preview that updates as parameters change
- [ ] **GUI-03**: User can share puzzle configuration via URL

### Infrastructure

- [ ] **INFR-01**: Puzzle generation runs in Rust compiled to WASM in the browser
- [ ] **INFR-02**: Connector generation uses pluggable trait/interface (extensible for future types)

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Border Variants

- **BORD-01**: User can generate irregular edge puzzles (no straight borders)
- **BORD-02**: User can generate no-edge puzzles (all connectors, no flat borders)
- **BORD-03**: User can generate all-edge puzzles (no connectors, all flat edges)
- **BORD-04**: User can define custom border shapes (non-rectangular outlines)

### Whimsy Pieces

- **WHIM-01**: User can place whimsy/figural pieces from a preset shape library
- **WHIM-02**: User can import custom SVG outlines as whimsy pieces
- **WHIM-03**: User can place multi-piece whimsy shapes spanning multiple grid cells

### Additional Connectors

- **CONN-04**: User can select from multiple connector types (flat tabs, wavy, angular)

### Polish

- **POLH-01**: User can select laser-cutter presets (Glowforge, LightBurn, Epilog)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Image overlay/printing | Different product — we generate cut paths, not printed puzzles. Users overlay images in laser cutter software. |
| DXF/PDF export | SVG universally supported by all laser cutter software. DXF has format dialect complexity. |
| Puzzle solving simulation | Different product (game vs tool). Massive scope for tangential value. |
| Mobile native app | Web GUI works on mobile browsers. No App Store overhead needed. |
| Real-time collaboration | Single-user tool. Share configurations via URL instead. |
| Per-piece manual editing | Violates parametric model, breaks seed reproducibility. Post-process in Inkscape. |
| 3D puzzle support | Different geometric domain requiring volumetric modeling. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| GRID-01 | — | Pending |
| GRID-02 | — | Pending |
| GRID-03 | — | Pending |
| GRID-04 | — | Pending |
| GRID-05 | — | Pending |
| GRID-06 | — | Pending |
| CONN-01 | — | Pending |
| CONN-02 | — | Pending |
| CONN-03 | — | Pending |
| EXPT-01 | — | Pending |
| EXPT-02 | — | Pending |
| GUI-01 | — | Pending |
| GUI-02 | — | Pending |
| GUI-03 | — | Pending |
| INFR-01 | — | Pending |
| INFR-02 | — | Pending |

**Coverage:**
- v1 requirements: 16 total
- Mapped to phases: 0
- Unmapped: 16 ⚠️

---
*Requirements defined: 2026-03-01*
*Last updated: 2026-03-01 after initial definition*
