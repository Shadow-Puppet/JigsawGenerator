# Phase 2: Grid Engine & Data Model - Context

**Gathered:** 2026-03-02
**Status:** Ready for planning

<domain>
## Phase Boundary

The computational engine that produces geometrically valid puzzle grid layouts with shared-edge architecture, deterministic seeded RNG, and configurable dimensions. Outputs abstract geometry (bezier control points), not SVG. Includes a pluggable connector trait for edge generation. Connector shape implementation (classic knobs) and SVG rendering are Phase 3.

</domain>

<decisions>
## Implementation Decisions

### Unit system & defaults
- Millimeters as primary internal unit for all engine math
- API accepts both mm and inches via a unit enum; inches converted to mm immediately on input
- Default puzzle size: A4 landscape (297x210mm)
- Default grid: 6x8 (48 pieces)
- Output can convert back to inches for display, but all storage/computation in mm

### Parameter bounds & feel
- Grid size: minimum 2x2, maximum 100x100
- Tab size: 15-45% of edge length, default 25%
- Jitter amount: 0-100%, default 50%
- 0% jitter = all connectors identical; 100% = maximum variation while staying geometrically valid

### Edge randomness model
- Tab direction (in/out) randomly assigned per edge from seed — no alternating pattern
- Single seed controls everything: tab directions AND jitter control point offsets
- Same seed = pixel-identical puzzle output across runs and platforms
- Seed input: string-based, hashed to u64 (e.g. user types "birthday" which deterministically hashes to a u64 for rand_chacha)
- Auto-generate a random seed by default; user can override with their own string
- Seed displayed in UI for copying/sharing

### Connector trait design
- Edge-level trait: takes edge parameters (length, direction, jitter params) and returns abstract bezier control points
- Trait handles internal edges only; border edges (straight lines + optional rounded corners) handled separately outside the trait
- Trait includes validation: generated paths checked for geometric validity (no overlapping adjacent pieces, stays within bounds)
- Output is Vec of cubic bezier control points (kurbo types), NOT SVG path data
- Only one implementation in v1 (classic knob), but trait designed to accept new types (wavy, angular, flat) in v2

### Claude's Discretion
- Exact hashing algorithm for string-to-u64 seed conversion
- Internal data structure choices for shared-edge storage
- Validation thresholds for geometric validity checks
- How piece count breakdown is computed and returned (struct shape)
- Rounded corner radius parameter range and default

</decisions>

<specifics>
## Specific Ideas

- String seeds for sharing puzzles by memorable names ("birthday", "christmas-2026") — makes URL sharing in Phase 4 more human-friendly
- rand_chacha already in dependencies from Phase 1 planning; use it for deterministic seeded RNG
- kurbo already planned for 2D geometry — bezier control points should use kurbo types directly
- Puzzle engine is pure Rust in puzzle-core crate; WASM boundary in puzzle-wasm crate follows existing JSON serialization pattern from Phase 1

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-grid-engine-data-model*
*Context gathered: 2026-03-02*
