# Phase 3: Connector Generation & SVG Export - Context

**Gathered:** 2026-03-02
**Status:** Ready for planning

<domain>
## Phase Boundary

The engine generates complete jigsaw puzzles with classic knob connectors and exports production-ready SVG files that work in laser cutter software. This phase implements the `ConnectorGenerator` trait (established in Phase 2) with real bezier connector shapes and builds the entire SVG export pipeline including kerf compensation. Requirements: CONN-01, CONN-02, EXPT-01, EXPT-02.

</domain>

<decisions>
## Implementation Decisions

### Knob shape & variation
- Traditional rounded knob shape (bell-curve bump/indent like Ravensburger puzzles)
- Visible neck (slight narrowing where knob meets edge baseline) for interlocking snap-fit — essential for laser-cut pieces to hold together
- Knob direction (in/out) assigned randomly per edge via seeded RNG — not alternating or patterned
- Jitter affects both control point positions AND knob center offset along the edge — each knob looks slightly different in shape and isn't perfectly centered
- Tab size percentage (from config) controls knob proportions relative to edge length

### SVG structure & laser compatibility
- Single combined `<path>` element with all cut lines — shared edges appear exactly once, no duplicate cuts
- Stroke: `stroke='#000000' stroke-width='0.001mm' fill='none'` — hairline black, industry standard for vector cutting
- Dimensions: explicit mm in width/height attributes with matching viewBox (`width='300mm' height='200mm' viewBox='0 0 300 200'`) — coordinates map 1:1 to millimeters
- Minimal SVG: just SVG namespace, dimensions, and path data — no title, desc, metadata, or editor-specific attributes
- Absolute coordinates throughout (no relative commands) per success criteria

### Kerf compensation
- Outward offset on all paths by half the kerf width — pieces end up slightly larger to compensate for laser-removed material
- Off by default (kerf = 0.0) — user opts in by setting their laser's kerf width
- Approximate offset for v1: flatten curves to polyline segments, offset each segment, re-smooth — sufficient for typical kerf values (0.05-0.2mm)
- Kerf modifies paths in-place in the single SVG export — no separate files, user just cuts the exported file

### Border & corner treatment
- Perfectly straight border edges — clean lines, no natural variation
- Circular arc (quarter-circle) at each of the 4 puzzle corners using configured `corner_radius` (default 2mm) — implemented with SVG arc commands
- Clean perpendicular T-junctions where internal edge connectors meet the border line — connectors start/end exactly at the border
- Corner pieces get no special treatment — same border + connector rules as other edge pieces, the rounded puzzle corner handles visual distinction

### Claude's Discretion
- Exact bezier control point math for the knob shape curves
- Polyline flattening resolution for kerf offset approximation
- SVG path command ordering and optimization for minimal file size
- How connector curves transition from the flat edge baseline into the knob shape
- Internal module structure for SVG generation code

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. The reference is traditional Ravensburger/Springbok-style jigsaw connectors, and LightBurn as the primary laser cutter software target for compatibility testing.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 03-connector-generation-svg-export*
*Context gathered: 2026-03-02*
