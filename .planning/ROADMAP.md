# Roadmap

Procedural jigsaw puzzle pattern generator that emits SVG cut paths for laser cutting. Rust core (`kurbo` geometry + `rand_chacha` deterministic RNG + `voronoice` Delaunay/Voronoi) → WASM bridge → vanilla TypeScript frontend with Canvas 2D live preview.

## Capability set

**Shipped:**
- **Piece-first CVT layout**: pieces come from a centroidal Voronoi tessellation with Lloyd relaxation over a closed boundary. Rectangle, heart, and star boundaries all go through the same path. The rectangular-grid-with-clipping architecture has been retired.
- Classic Ravensburger-style knob connectors, scaled proportionally per edge, deterministic from a seed string.
- Proportional small-knob pruning: edges shorter than 35 % of median edge length lose their connector, subject to the constraint that every piece keeps at least 2 knobbed edges.
- Canvas live preview with zoom/pan/touch, adaptive-tick rulers on both axes, URL-based config sharing, SVG download for laser cutting.
- Advisory dimension sizing: warns when current `W × H` can't host 3 mm neck openings for the requested piece count; never mutates user input.

**Next up (no active plan yet):**
- Whimsy pieces: figural shapes placed inside the CVT layout, with surrounding pieces re-relaxed to hug the whimsy's contour as a cut line. The original whimsy plans (`.planning/plans/pending/005-*`, `006-*`) were written against the retired rectangular-grid architecture and need to be re-scoped for the CVT world before any of them becomes active.
- Export/URL/filename polish (`pending/007-*`).

**Explicitly out of scope for v1 (deferred):**
- Multiple whimsies per puzzle, whimsy–whimsy intersections
- User-imported SVG outlines
- DXF/PDF export
- Per-piece randomization of knob shape parameters (knob size/neck taper/offset are all hardcoded constants now)

**Future directions (not scheduled):**
- Alternative tessellations: hex, triangle, Penrose, Truchet, custom periodic/aperiodic tilings. The `PuzzleLayout` abstraction is algorithm-agnostic — new layouts plug in as `build_*_layout` functions.
- Additional connector styles beyond classic knob
- Irregular / no-edge / all-edge border variants

## Milestone sequence

1. **Rectangular foundation** (complete): build pipeline, classic knob connector, web GUI with live preview, URL sharing, SVG download. See `plans/completed/001-rectangular-foundation.md`.
2. **Custom borders via CVT** (complete): centroidal Voronoi with Lloyd relaxation inside an arbitrary boundary shape, classic knob connectors on each shared Voronoi edge, proportional small-knob pruning, dimension-sizing warnings, adaptive rulers. The old mask/reverse-mask + rectangular-grid-clip approach was retired in favor of CVT for every boundary type.
3. **Whimsy & polish**: whimsy piece placement, export polish. Plans in `pending/` need re-scoping against the CVT architecture.
4. **Beyond M002**: deferred items above become candidate milestones once there's demand.

## Architecture invariants

See `CLAUDE.md` for the detail. In short:
- `PuzzleLayout` is the algorithm-agnostic contract between layout builders and everything downstream (connector generation, SVG/binary export, WASM bridge). Each builder is one function: `build_rectangular_grid` (legacy, deleted), `build_cvt_layout` (current), `build_hex_layout` (future), …
- Deterministic RNG: ChaCha8 seeded by FNV-1a of a seed string. Seed-derived streams split independent subsystems — `"{seed}-positions"` for initial seed scatter, `"{seed}-directions"` for per-edge knob direction — so changes to one don't perturb the other.
- Lloyd relaxation is deterministic (no RNG); same seed string reproduces the same puzzle exactly on any platform.
- Connector geometry is fixed by constants in `classic_connector.rs` (`KNOB_WIDTH_RATIO = 0.25`, `NECK_RATIO = 0.25`); knob scale follows edge length.
- Knob shape decisions are post-hoc on the finished layout (proportional pruning), never layered into the CVT pipeline itself — keeps the tessellation stable even if knob rules change.
