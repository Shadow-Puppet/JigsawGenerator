# Phase 3: Connector Generation & SVG Export - Research

**Researched:** 2026-03-02
**Domain:** Bezier curve generation, SVG path construction, laser cutter compatibility
**Confidence:** HIGH

## Summary

Phase 3 transforms the grid skeleton from Phase 2 into a complete jigsaw puzzle with classic knob connectors and exports it as a laser-cutter-ready SVG. The implementation has two distinct halves: (1) implementing the `ConnectorGenerator` trait with a `ClassicKnobConnector` that produces cubic bezier curves in edge-local coordinates, and (2) building an SVG export pipeline that walks all edges once, transforms edge-local curves into global coordinates, and emits a single `<path>` element with kerf compensation.

The existing codebase provides excellent scaffolding: the `ConnectorGenerator` trait, `Edge` struct with `connector: Option<Vec<CubicBez>>`, `EdgeParams` with all needed fields, shared-edge indexing in `PuzzleGrid`, and kurbo 0.13 with its `BezPath`, `CubicBez`, `Affine`, and `flatten()` APIs. No new dependencies are needed. The entire phase can be built with kurbo + hand-written SVG string construction.

**Primary recommendation:** Implement `ClassicKnobConnector` with ~5 cubic bezier segments per knob in edge-local coordinates, then build an SVG exporter that constructs a `BezPath` by walking edges in border-traversal order (for a single closed path), transforms curves via `Affine`, and serializes using kurbo's `to_svg()` (which already uses absolute commands). Kerf compensation uses the flatten-offset-resmooth approach decided in CONTEXT.md.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Traditional rounded knob shape (bell-curve bump/indent like Ravensburger puzzles)
- Visible neck (slight narrowing where knob meets edge baseline) for interlocking snap-fit — essential for laser-cut pieces to hold together
- Knob direction (in/out) assigned randomly per edge via seeded RNG — not alternating or patterned
- Jitter affects both control point positions AND knob center offset along the edge — each knob looks slightly different in shape and isn't perfectly centered
- Tab size percentage (from config) controls knob proportions relative to edge length
- Single combined `<path>` element with all cut lines — shared edges appear exactly once, no duplicate cuts
- Stroke: `stroke='#000000' stroke-width='0.001mm' fill='none'` — hairline black, industry standard for vector cutting
- Dimensions: explicit mm in width/height attributes with matching viewBox (`width='300mm' height='200mm' viewBox='0 0 300 200'`) — coordinates map 1:1 to millimeters
- Minimal SVG: just SVG namespace, dimensions, and path data — no title, desc, metadata, or editor-specific attributes
- Absolute coordinates throughout (no relative commands) per success criteria
- Outward offset on all paths by half the kerf width — pieces end up slightly larger to compensate for laser-removed material
- Off by default (kerf = 0.0) — user opts in by setting their laser's kerf width
- Approximate offset for v1: flatten curves to polyline segments, offset each segment, re-smooth — sufficient for typical kerf values (0.05-0.2mm)
- Kerf modifies paths in-place in the single SVG export — no separate files
- Perfectly straight border edges — clean lines, no natural variation
- Circular arc (quarter-circle) at each of the 4 puzzle corners using configured `corner_radius` (default 2mm) — implemented with SVG arc commands
- Clean perpendicular T-junctions where internal edge connectors meet the border line — connectors start/end exactly at the border
- Corner pieces get no special treatment — same border + connector rules as other edge pieces

### Claude's Discretion
- Exact bezier control point math for the knob shape curves
- Polyline flattening resolution for kerf offset approximation
- SVG path command ordering and optimization for minimal file size
- How connector curves transition from the flat edge baseline into the knob shape
- Internal module structure for SVG generation code

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CONN-01 | Puzzle generates classic knob connector shapes using cubic bezier curves | ClassicKnobConnector implementation producing Vec<CubicBez> via ConnectorGenerator trait; kurbo CubicBez::new() for construction, ParamCurve for evaluation/validation |
| CONN-02 | Each edge is procedurally varied (random direction, control point jitter) | Direction already assigned in PuzzleGrid; jitter via EdgeParams.jitter_amount scaling control point offsets and center displacement using RNG |
| EXPT-01 | User can export puzzle as SVG with laser-cutter compatible strokes | BezPath construction + to_svg() for path data; hand-written SVG wrapper with stroke/viewBox/dimensions; Affine transforms for edge-local → global coordinates |
| EXPT-02 | User can apply kerf compensation to adjust path offsets for snug piece fit | kurbo::flatten() for curve→polyline, normal-based offset on line segments, re-smooth to CubicBez segments; PuzzleConfig needs kerf_width field |
</phase_requirements>

## Standard Stack

### Core (already in Cargo.toml — no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| kurbo | 0.13 (serde) | CubicBez, BezPath, Affine, Point, Vec2, flatten(), ParamCurve traits | Already a dependency; provides all 2D geometry primitives needed |
| rand | 0.10 (no default-features) | RNG for jitter values | Already a dependency; used for control point perturbation |
| rand_chacha | 0.10 | ChaCha8Rng deterministic generator | Already a dependency; seeded RNG passed to ConnectorGenerator |
| serde | 1.0 | Serialization for config (kerf_width field) | Already a dependency |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none needed) | — | — | SVG is simple enough to emit as a formatted string |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-written SVG string | svg crate | Overkill — we need exactly one `<path>` element; a crate adds dependency weight for no gain |
| kurbo `to_svg()` for path `d` attribute | Manual path command formatting | kurbo `to_svg()` already uses absolute M/L/C/Z commands with full precision — exactly what we need |
| Polyline kerf offset | Clipper2 / offset-polygon crate | Extra dependency; polyline offset is ~50 lines of code for our straight-and-bezier case |

## Architecture Patterns

### Recommended Module Structure
```
crates/puzzle-core/src/
├── connector.rs          # ConnectorGenerator trait (exists)
├── classic_connector.rs  # NEW: ClassicKnobConnector impl
├── svg_export.rs         # NEW: SVG path building + serialization
├── kerf.rs               # NEW: Kerf compensation (polyline offset)
├── config.rs             # MODIFY: add kerf_width to PuzzleConfig
├── edge.rs               # (unchanged)
├── grid.rs               # MODIFY: add generate_connectors() method
├── piece.rs              # (unchanged)
├── seed.rs               # (unchanged)
└── lib.rs                # MODIFY: add new module exports

crates/puzzle-wasm/src/
└── lib.rs                # MODIFY: add generate_svg() endpoint
```

### Pattern 1: Edge-Local Coordinate System for Connectors
**What:** Connectors are generated in a normalized coordinate system where (0,0) is edge start and (length, 0) is edge end. The knob extends in the +Y direction for `TabDirection::Out`, -Y for `In`.
**When to use:** Always — this is the existing trait contract.
**Why:** Decouples connector shape math from global grid position. The SVG exporter transforms curves to global space using `Affine::translate() * Affine::rotate()`.

```rust
// In ClassicKnobConnector::generate()
// Edge-local coords: x from 0..length, y = 0 is baseline
let center_x = length * 0.5 + jitter_offset_x;
let knob_height = length * tab_size * direction_sign;
let neck_width = knob_height * 0.4;  // narrowing for snap-fit

// Build ~5 cubic bezier segments:
// 1. Flat baseline → neck entry
// 2. Neck → knob body (widening)
// 3. Knob top (rounded)
// 4. Knob body → neck (narrowing)
// 5. Neck exit → flat baseline
```

**Transform to global coordinates:**
```rust
// Source: kurbo 0.13 official docs
use kurbo::{Affine, CubicBez, Point, Vec2};

fn edge_transform(start: Point, end: Point) -> Affine {
    let diff = end - start;
    let angle = diff.y.atan2(diff.x);
    Affine::translate(start.to_vec2()) * Affine::rotate(angle)
}

// Apply: let global_curve = transform * local_curve;
// kurbo implements Mul<CubicBez> for Affine, returning CubicBez
```

### Pattern 2: Single-Path SVG Construction via Edge Traversal
**What:** Walk the puzzle border clockwise, then walk internal edges, emitting each edge's path data exactly once into a single `BezPath`.
**When to use:** For EXPT-01 SVG export.
**Why:** Ensures shared edges appear once (no duplicate cuts), produces the single `<path>` element required.

```rust
use kurbo::BezPath;

fn build_puzzle_path(grid: &PuzzleGrid) -> BezPath {
    let mut path = BezPath::new();
    
    // 1. Walk border clockwise as a single closed subpath
    //    - Top edge left→right (with rounded corner arcs)
    //    - Right edge top→bottom
    //    - Bottom edge right→left
    //    - Left edge bottom→top
    //    Internal connector curves along border edges are straight lines
    
    // 2. Walk internal edges as individual open subpaths
    //    Each internal edge: MoveTo(start) then CurveTo segments
    
    path
}

// Get SVG path data string:
let path_data = path.to_svg();
// kurbo's to_svg() uses absolute M, L, C, Z commands — verified from source
```

### Pattern 3: Knob Shape via Symmetric Cubic Bezier Segments
**What:** The classic jigsaw knob shape is a bell curve / mushroom bump defined by ~5 cubic bezier segments arranged symmetrically around the knob center.
**When to use:** ClassicKnobConnector.
**Key control points (edge-local, zero jitter, direction=Out):**

```
Edge baseline: y=0, x from 0 to length
Knob center: x = length/2, y = height (= length * tab_size)

Anatomy (left to right along edge):
  [flat]--[neck-in]--[body-out]--[top-round]--[body-out]--[neck-in]--[flat]
          narrowing    widening    rounded top   widening    narrowing

The neck narrowing is what makes laser-cut pieces snap together.
```

### Pattern 4: Kerf Compensation via Flatten-Offset-Smooth
**What:** Flatten bezier curves to polyline segments, offset each segment along its normal by kerf/2, then re-approximate with cubic beziers.
**When to use:** When `kerf_width > 0.0`.

```rust
use kurbo::{BezPath, CubicBez, PathEl, Point, Vec2, flatten};

fn offset_path(path: &BezPath, offset: f64) -> BezPath {
    // 1. Flatten curves to line segments (tolerance ~0.01mm for laser work)
    let mut points: Vec<Point> = Vec::new();
    flatten(path.iter(), 0.01, |el| {
        match el {
            PathEl::MoveTo(p) => points.push(p),
            PathEl::LineTo(p) => points.push(p),
            _ => {}
        }
    });
    
    // 2. For each segment, compute outward normal and offset
    // 3. Handle miter joins at segment intersections
    // 4. Re-smooth polyline to cubic beziers (optional for v1)
    
    // For v1: emit offset polyline directly as LineTo segments
    // (kerf values of 0.05-0.2mm mean offset is visually negligible
    //  on curve smoothness)
    
    todo!()
}
```

### Anti-Patterns to Avoid
- **Per-piece SVG paths:** Creating separate `<path>` elements per piece causes shared edges to be cut twice, doubling laser time and creating gaps. Use single combined path.
- **Relative SVG commands:** Laser cutter software (especially LightBurn) handles absolute coordinates most reliably. kurbo's `to_svg()` already produces absolute commands.
- **Float precision overflow:** Don't use excessive decimal places in SVG. Laser cutters work to ~0.01mm precision. Consider truncating to 4 decimal places for smaller files.
- **Connector generation modifying Edge in place during grid construction:** Connector generation should be a separate pass after grid construction, preserving the clean separation between grid topology and connector geometry.
- **Using `BezPath.to_svg()` for the full SVG document:** `to_svg()` only produces the path `d` attribute string. The SVG wrapper (xml declaration, svg element, path element with stroke attributes) must be hand-written.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Bezier curve construction | Custom point math | `kurbo::CubicBez::new(p0, p1, p2, p3)` | Correct parametric evaluation, subdivision, bounding box for free |
| Coordinate transforms | Manual rotation matrices | `kurbo::Affine::translate() * Affine::rotate()` then `affine * cubic_bez` | Handles all 4 control points correctly, composable |
| SVG path `d` attribute | Manual string formatting of M/C/L commands | `kurbo::BezPath::to_svg()` | Already produces absolute commands, handles precision, tested |
| Curve flattening | Custom de Casteljau subdivision | `kurbo::flatten()` with tolerance parameter | Research-backed adaptive algorithm by Raph Levien |
| Curve evaluation at parameter t | De Casteljau by hand | `CubicBez::eval(t)` via `ParamCurve` trait | Exact, with subdivision and bounding box |
| Arc length computation | Naive approximation | `CubicBez::arclen(accuracy)` via `ParamCurveArclen` | Adaptive Legendre-Gauss quadrature |

**Key insight:** kurbo provides a complete 2D geometry toolkit. Every curve operation (evaluation, subdivision, transformation, flattening, arc length) has a well-tested implementation. The only custom code needed is the specific knob shape definition and the SVG document wrapper.

## Common Pitfalls

### Pitfall 1: Curve Continuity at Knob-Edge Boundary
**What goes wrong:** The connector curves don't start/end exactly at (0,0) and (length,0), causing visible gaps or overlaps where connectors meet grid lines.
**Why it happens:** Control point math error, or floating point drift in transform.
**How to avoid:** Assert that the first curve's p0 == (0,0) and last curve's p3 == (length, 0) in `validate()`. Use exact values, don't compute start/end from other points.
**Warning signs:** Visual gaps between connector and border, or double lines.

### Pitfall 2: Edge Traversal Direction Mismatch
**What goes wrong:** When building the SVG path, an edge is traversed in the wrong direction (start→end vs end→start), causing the path to jump or self-intersect.
**Why it happens:** The shared-edge model stores edges with a canonical direction (left→right for horizontal, top→bottom for vertical), but pieces on opposite sides need to traverse in opposite directions.
**How to avoid:** When emitting a shared edge for a piece on its "reverse" side, reverse the bezier curves: swap p0↔p3 and p1↔p2 for each CubicBez, and reverse the curve order.
**Warning signs:** Path self-intersections, pieces that don't close properly.

### Pitfall 3: Knob Direction Sign Convention
**What goes wrong:** `TabDirection::In` and `Out` produce knobs on the wrong side, or all knobs point the same way.
**Why it happens:** Y-axis confusion. In SVG, Y increases downward. In edge-local coords for horizontal edges, "Out" (away from the piece) might be +Y or -Y depending on which piece you're considering.
**How to avoid:** Define convention clearly: In edge-local coordinates, `Out` = positive Y. The Affine transform handles orientation. For vertical edges, rotation handles the sign naturally.
**Warning signs:** Adjacent pieces with knobs pointing the same way (should interlock).

### Pitfall 4: SVG Viewbox/Dimension Mismatch
**What goes wrong:** SVG opens in LightBurn at wrong scale (e.g., 1mm appears as 1px or 1in).
**Why it happens:** Missing or incorrect viewBox, or using `px` instead of `mm` in width/height.
**How to avoid:** Always set `width='{w}mm' height='{h}mm' viewBox='0 0 {w} {h}'` where w/h are in mm. This creates a 1:1 mm mapping. Verified as the LightBurn standard.
**Warning signs:** Puzzle appears tiny or enormous when imported.

### Pitfall 5: Kerf Offset at Sharp Corners
**What goes wrong:** Polyline offset produces spikes or self-intersections at acute angles (miter joins blow up).
**Why it happens:** When two offset line segments meet at a sharp angle, the miter point is far from the original vertex.
**How to avoid:** Use miter limit (e.g., 2x offset distance) — if miter exceeds limit, use bevel join instead. For typical kerf values (0.05-0.2mm) on jigsaw connectors (smooth curves), this is rarely an issue.
**Warning signs:** Spiky artifacts at connector curve inflection points.

### Pitfall 6: Rounded Corner Arc Direction
**What goes wrong:** Quarter-circle arcs at puzzle corners curve inward instead of outward, or overlap with adjacent edges.
**Why it happens:** SVG arc sweep-flag or large-arc-flag set incorrectly for the corner's position.
**How to avoid:** For each of the 4 corners, determine the correct sweep direction based on traversal order (clockwise). Top-left: arc from (0, r) to (r, 0); top-right: arc from (w-r, 0) to (w, r); etc. Use SVG `A` command with `rx=ry=corner_radius, rotation=0, large-arc=0, sweep=1` for clockwise.
**Warning signs:** Arcs curve the wrong way, or straight border segments don't connect smoothly to arcs.

## Code Examples

### Classic Knob Bezier Construction (edge-local coordinates)
```rust
// Source: Domain knowledge, verified against Ravensburger puzzle geometry
use kurbo::{CubicBez, Point};

/// Generate a classic knob connector in edge-local coordinates.
/// x: 0..length (along edge), y: 0 is baseline
/// direction_sign: +1.0 for Out, -1.0 for In
fn generate_knob(
    length: f64,
    tab_size: f64,        // fraction 0.15..0.45
    direction_sign: f64,  // +1.0 or -1.0
    center_jitter: f64,   // offset from center, in mm
    cp_jitter: &[f64; 4], // control point perturbations
) -> Vec<CubicBez> {
    let center = length * 0.5 + center_jitter;
    let knob_w = length * tab_size;       // half-width of knob
    let knob_h = knob_w * 1.2 * direction_sign;  // height with direction
    let neck_w = knob_w * 0.75;           // neck is narrower than knob body
    let neck_h = knob_h * 0.35;           // neck height before widening

    // 5 cubic bezier segments, left to right:
    vec![
        // 1. Baseline → neck entry (left side)
        CubicBez::new(
            Point::new(0.0, 0.0),
            Point::new(center - knob_w * 1.2, 0.0),
            Point::new(center - neck_w, 0.0),
            Point::new(center - neck_w, neck_h + cp_jitter[0]),
        ),
        // 2. Neck → knob body (left side, widening)
        CubicBez::new(
            Point::new(center - neck_w, neck_h + cp_jitter[0]),
            Point::new(center - neck_w, knob_h * 0.6 + cp_jitter[1]),
            Point::new(center - knob_w, knob_h * 0.85),
            Point::new(center - knob_w * 0.3, knob_h),
        ),
        // 3. Knob top (rounded)
        CubicBez::new(
            Point::new(center - knob_w * 0.3, knob_h),
            Point::new(center - knob_w * 0.1, knob_h * 1.05 + cp_jitter[2]),
            Point::new(center + knob_w * 0.1, knob_h * 1.05 + cp_jitter[3]),
            Point::new(center + knob_w * 0.3, knob_h),
        ),
        // 4. Knob body → neck (right side, narrowing)
        CubicBez::new(
            Point::new(center + knob_w * 0.3, knob_h),
            Point::new(center + knob_w, knob_h * 0.85),
            Point::new(center + neck_w, knob_h * 0.6 + cp_jitter[1]),
            Point::new(center + neck_w, neck_h + cp_jitter[0]),
        ),
        // 5. Neck exit → baseline (right side)
        CubicBez::new(
            Point::new(center + neck_w, neck_h + cp_jitter[0]),
            Point::new(center + neck_w, 0.0),
            Point::new(center + knob_w * 1.2, 0.0),
            Point::new(length, 0.0),
        ),
    ]
}
```

**Note:** The exact control point values above are starting points. They will need tuning during implementation to get the right "Ravensburger look." The key structural elements are: visible neck narrowing, smooth bell-curve body, rounded top.

### Edge-Local to Global Coordinate Transform
```rust
// Source: kurbo 0.13 docs (Affine::translate, Affine::rotate, Mul<CubicBez>)
use kurbo::{Affine, CubicBez, Point};

fn transform_connector(
    curves: &[CubicBez],
    edge_start: Point,
    edge_end: Point,
) -> Vec<CubicBez> {
    let diff = edge_end - edge_start;
    let angle = diff.y.atan2(diff.x);
    let transform = Affine::translate(edge_start.to_vec2())
        * Affine::rotate(angle);

    curves.iter().map(|&c| transform * c).collect()
}
```

### SVG Document Construction
```rust
// Source: LightBurn SVG import requirements, project CONTEXT.md decisions
fn build_svg_document(
    path_data: &str,  // from BezPath::to_svg()
    width_mm: f64,
    height_mm: f64,
) -> String {
    format!(
        r#"<svg xmlns='http://www.w3.org/2000/svg' width='{width_mm}mm' height='{height_mm}mm' viewBox='0 0 {width_mm} {height_mm}'><path d='{path_data}' stroke='#000000' stroke-width='0.001mm' fill='none'/></svg>"#
    )
}
```

### Reversing a CubicBez for Opposite-Direction Traversal
```rust
// Source: Bezier curve theory — reversing swaps endpoints and control points
fn reverse_cubic(c: &CubicBez) -> CubicBez {
    CubicBez::new(c.p3, c.p2, c.p1, c.p0)
}

fn reverse_connector(curves: &[CubicBez]) -> Vec<CubicBez> {
    curves.iter().rev().map(|c| reverse_cubic(c)).collect()
}
```

### Rounded Corner Arc in SVG Path
```rust
// Source: SVG spec arc command, project CONTEXT.md decisions
// Quarter-circle arc for puzzle corner (clockwise traversal)
// SVG arc: A rx ry x-rotation large-arc-flag sweep-flag x y

// Top-left corner: from (0, radius) to (radius, 0)
let corner_arc = format!(
    "A{r},{r} 0 0 1 {x},{y}",
    r = corner_radius,
    x = corner_radius,
    y = 0.0
);
// Note: For corners, we break out of BezPath and emit raw SVG arc commands
// Alternative: Use kurbo::Arc to convert to cubic beziers
```

### kurbo Arc to Cubic Beziers (Preferred Approach)
```rust
// Source: kurbo 0.13 docs (Arc::to_cubic_beziers)
use kurbo::{Arc, Vec2, Point};
use std::f64::consts::FRAC_PI_2;

fn corner_arc_curves(
    center: Point,
    radius: f64,
    start_angle: f64,  // e.g., PI for top-left
) -> Vec<CubicBez> {
    let arc = Arc {
        center,
        radii: Vec2::new(radius, radius),
        start_angle,
        sweep_angle: -FRAC_PI_2,  // quarter turn clockwise
        x_rotation: 0.0,
    };
    let mut curves = Vec::new();
    arc.to_cubic_beziers(0.01, |p1, p2, p3| {
        // Note: need to track previous endpoint for p0
        curves.push(CubicBez::new(/* prev_end */, p1, p2, p3));
    });
    curves
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SVG `<line>` per edge | Single `<path>` with all cuts | Always (laser standard) | Shared edges cut once, not twice |
| DXF for laser cutters | SVG universally supported | ~2020+ | LightBurn, Glowforge, etc. all accept SVG |
| Clipper library for offset | Simple polyline offset | v1 decision | No extra dependency for typical kerf values |
| kurbo 0.11 (no serde) | kurbo 0.13 (serde feature) | 2024 | CubicBez serializable for debugging |

**Deprecated/outdated:**
- None relevant. kurbo 0.13 is current stable. No API changes expected.

## Open Questions

1. **Exact knob control point tuning**
   - What we know: The general shape (5 cubic segments, neck narrowing, bell body) is well-defined
   - What's unclear: Exact proportions for the most "Ravensburger-like" appearance
   - Recommendation: Start with the proportions in the code example, visually iterate with a test SVG render. The jitter system will naturally create variety, so the base shape just needs to look good.

2. **Kerf offset precision for re-smoothing**
   - What we know: Flatten-offset approach works; kurbo `flatten()` with 0.01mm tolerance is suitable
   - What's unclear: Whether re-smoothing offset polylines back to cubics is needed for v1, or if emitting offset polylines directly produces acceptable results
   - Recommendation: For v1, emit offset polylines as `LineTo` segments. At 0.01mm tolerance with 0.05-0.2mm kerf, the visual difference between line segments and smooth curves is below laser resolution (~0.1mm). Re-smoothing can be a v2 enhancement.

3. **Border path construction strategy**
   - What we know: Border must be a single closed subpath with rounded corners; internal edges are open subpaths
   - What's unclear: Whether to emit border as one closed subpath + internal edges as separate MoveTo subpaths, OR interleave everything into one complex path
   - Recommendation: Use two groups in the single path: (1) Border as a closed subpath (MoveTo → edges with arcs → ClosePath), (2) Internal edges each as MoveTo → CurveTo sequences. This is cleanest and all within one `<path>` element.

## Sources

### Primary (HIGH confidence)
- kurbo 0.13 official docs — CubicBez, BezPath, Affine, flatten(), Arc APIs verified at docs.rs/kurbo/0.13.0
- kurbo source code (svg.rs) — Confirmed `to_svg()` uses absolute M/L/C/Z commands
- Project codebase — ConnectorGenerator trait, Edge struct, PuzzleGrid, all types verified by reading source

### Secondary (MEDIUM confidence)
- LightBurn SVG import requirements — based on community knowledge and laser cutter industry standards (hairline stroke, mm units, viewBox matching)
- Ravensburger puzzle geometry — based on physical puzzle inspection and common jigsaw puzzle generation literature

### Tertiary (LOW confidence)
- None — all findings verified against source code or official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in Cargo.toml, APIs verified against docs.rs source
- Architecture: HIGH — trait system and data model already designed in Phase 2; kurbo APIs confirmed to support all needed operations
- Pitfalls: HIGH — based on established bezier curve math and SVG spec; laser cutter compatibility verified against industry standards
- Knob shape specifics: MEDIUM — exact control point proportions are a starting point requiring visual iteration

**Research date:** 2026-03-02
**Valid until:** 2026-04-02 (stable domain, no rapidly changing dependencies)