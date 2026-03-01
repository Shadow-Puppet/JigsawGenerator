# Feature Research

**Domain:** Procedural jigsaw puzzle pattern generation (SVG cut paths for laser cutting)
**Researched:** 2026-03-01
**Confidence:** HIGH (core features verified against 4+ existing open-source implementations)

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete. Derived from analyzing every existing open-source jigsaw puzzle SVG generator (Draradech/jigsaw, MB-Deen/jigsaw-svg-generator, astbis/laser-jigsaw-generator, zvikabh/jigsaw-puzzle-svg).

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Configurable grid (rows x columns) | Every generator has this. First thing users look for. | LOW | Two integer inputs controlling piece count. All competitors have this. |
| Configurable puzzle dimensions (mm/inches) | Physical output needs physical units. Laser cutters work in real-world units. | LOW | Must support both metric (mm) and imperial (inches). Draradech uses mm only. |
| Classic knob connector (tab/blank) shape | The universally recognized jigsaw piece shape. Cubic bezier curves forming the interlocking "knob". | MEDIUM | Draradech implementation uses 3 cubic bezier segments per edge with 10 control points. This is the proven approach — each edge is defined by control points with parametric variation. |
| Procedural per-edge randomization | No two edges should look identical. All existing generators randomize each edge. | MEDIUM | Random flip direction (in/out), jitter on control points. Must maintain geometric validity (no self-intersections). |
| Seed-based reproducibility | Users need to reproduce exact puzzles (re-cut, share configurations). Every generator has a seed parameter. | LOW | Single integer seed driving a deterministic PRNG. Display it prominently so users can save/share. |
| Tab size control | Controls how large/prominent the interlocking knobs are. Present in all generators. | LOW | Single parameter as percentage of edge length. Draradech default: 20%, range 10-30%. |
| Jitter/randomness amount control | Controls how much variation per piece. Present in all generators. | LOW | Single parameter as percentage. Draradech default: 4%, range 0-13%. |
| SVG export (laser-cutter compatible) | The entire point of the tool. Must produce clean vector paths. | MEDIUM | Paths must be: no-fill, stroke-only, thin stroke widths (0.01-0.1mm for laser software). Must be valid SVG with real-world units (mm) in viewBox. Color-code border vs interior paths (Draradech uses DarkBlue/DarkRed/Black). |
| Live preview in browser | Users need to see what they're generating before downloading. Every web generator has this. | MEDIUM | Must update responsively as parameters change. SVG rendering in browser canvas. |
| Rounded corner radius on border | Rectangular puzzles need configurable corner rounding. Prevents sharp corners that can break on thin materials. | LOW | Single float parameter in mm. Draradech default: 2.0mm. Uses SVG arc commands. |
| Web GUI with parameter controls | Browser-based interface with sliders/inputs for all parameters. No install required. | MEDIUM | Sliders for seed, tab size, jitter. Text inputs for dimensions, grid size, corner radius. Download button. |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valued. **No existing open-source generator has any of these features** — this is where we create value.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Whimsy/figural pieces (preset library) | Premium artisan jigsaw puzzles feature recognizable shapes (animals, stars, keys, hearts) cut into the puzzle grid. Currently only available from hand-cut $200+ wooden puzzle makers. **No software generator supports this.** | HIGH | Requires: shape library as SVG paths, algorithm to place shapes on grid replacing standard pieces, re-routing connectors around whimsy outlines. This is the single biggest differentiator. |
| Whimsy pieces from user-imported SVG | Let users import their own SVG outlines as whimsy shapes. Personalization that no competitor offers. | HIGH | Must validate SVG paths are closed, simple (non-self-intersecting), and fit within grid bounds. Requires path simplification and validation pipeline. Depends on preset whimsy working first. |
| Multi-piece whimsy (spanning multiple grid cells) | A single whimsy shape that replaces 2-4 adjacent standard pieces. Creates dramatic "reveal" moments during assembly. | VERY HIGH | Most complex whimsy variant. Must handle partial grid cell replacement, connector re-routing across cell boundaries, and ensure remaining cells still interlock. Defer to v2+. |
| Multiple connector types | Beyond classic knob: flat tabs, wavy connectors, angular/geometric connectors, rounded bumps. Adds visual variety and difficulty tuning. | MEDIUM | Each type is a different set of bezier curves defining the edge shape. Core architecture must abstract "edge generator" so types are pluggable. |
| Custom border shapes (non-rectangular) | Circular, hexagonal, heart-shaped, or arbitrary outline puzzles. Draradech has hexagonal as a separate tool but no arbitrary shapes. | HIGH | Requires: define outer boundary as path, clip grid to boundary, handle partial edge pieces at border, re-route edges that cross the boundary. |
| Irregular edge puzzles (no straight borders) | All edges have connectors, no flat border pieces. Increases difficulty. | MEDIUM | Replace flat border edges with connector edges. Conceptually simple but requires border-edge generation logic change. |
| No-edge puzzles (all connectors) | Every edge including borders has connectors. Puzzle "floats" with no frame reference. | LOW | Variant of irregular edges — just enable connectors on all borders. |
| All-edge puzzles (no connectors) | Every edge is flat — pieces are differentiated only by shape/size variation in the grid. Extremely hard to solve. | LOW | Disable connector generation entirely. Jitter on grid line positions creates unique shapes. |
| Laser-cutter stroke presets | One-click presets for Glowforge, LightBurn, Epilog — auto-set stroke widths and colors to match each software's cut/engrave conventions. | LOW | astbis/laser-jigsaw-generator pioneered this with 0.01mm red strokes. We can go further with named presets. |
| Configuration sharing via URL | Encode all parameters in URL hash/query params so users can share exact puzzle configs. | LOW | Serialize parameters to URL. No existing generator does this (they use seed sliders). |
| Piece count display | Show total piece count and highlight how many are edge/corner/interior. Users care about count for difficulty. | LOW | Pure UI feature. count = rows*cols, edges = 2*(rows+cols)-4, corners = 4. |
| Material thickness kerf compensation | Adjust path offsets to account for laser kerf width so pieces fit snugly. Critical for actual manufacturing. | MEDIUM | Requires path offset algorithm (inset/outset by half kerf width). Parameter in mm (typical: 0.1-0.2mm for wood). |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems. Deliberately NOT building these.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Image overlay/printing | Users want to see a picture on the puzzle pieces | Fundamentally different product — we generate CUT PATHS, not printed puzzles. Image overlay requires raster processing, print alignment, color management. Massive scope expansion for a different audience. | Users overlay images in their laser cutter software (LightBurn, Glowforge) which already handles this well. Our SVG imports cleanly as a cut layer. |
| DXF/PDF export | Some laser software prefers DXF | SVG is universally supported by every laser cutter software. DXF adds a complex format with many dialects (R12, R14, 2000, 2004+) and coordinate system headaches. PDF adds print-oriented complexity. | SVG-only for v1. All major laser software (LightBurn, Glowforge, Epilog) imports SVG natively. Revisit only if multiple users report SVG import failures. |
| Puzzle solving simulation | Users want to "play" the puzzle in browser | Completely different product (game vs tool). Requires piece physics, drag-and-drop, snap detection, image rendering. 10x scope for tangential value. | Stay focused on generation. If users want to test-solve, they can import SVG into any online jigsaw app. |
| Mobile-native app | Mobile users exist | Web GUI works on mobile browsers already. Native app adds App Store overhead, review cycles, platform maintenance (iOS + Android), and WASM works in mobile browsers. | Responsive web design. Test on mobile browsers. PWA if offline use requested. |
| Real-time collaboration | Multiple users editing puzzle params simultaneously | Adds WebSocket/server infrastructure for a tool that's inherently single-user (one person configures, exports, laser-cuts). No competitor has this because nobody needs it. | Share configurations via URL instead. |
| Undo/redo system | UI convention | For a parametric generator, every state is fully determined by parameters. "Undo" is just "change the number back." Browser back button + URL params handle this naturally. | Parameter persistence in URL hash. Browser history acts as natural undo. |
| Per-piece editing | Users want to manually adjust individual pieces | Violates the parametric generation model. If you hand-edit one piece, you break reproducibility (seed no longer defines the output). Also requires a full vector editor which is a massive product. | Expose enough parameters (jitter, tab size, connector type) that manual editing is unnecessary. Users can post-process SVG in Inkscape for one-off tweaks. |
| 3D puzzle support | Cool-sounding feature | Completely different geometric domain. 3D interlocking requires volumetric modeling, not 2D path generation. Would require a CAD kernel, not an SVG generator. | Stay 2D. 3D puzzles are a different product category. |

## Feature Dependencies

```
[Grid generation (rows x cols)]
    |
    +---> [Classic knob connector]
    |         |
    |         +---> [Multiple connector types] (requires abstract edge interface)
    |         |
    |         +---> [Per-edge randomization + seed]
    |
    +---> [SVG export]
    |         |
    |         +---> [Laser-cutter stroke presets]
    |         |
    |         +---> [Kerf compensation] (path offset on exported SVG)
    |
    +---> [Live preview]
    |         |
    |         +---> [Piece count display]
    |
    +---> [Border generation (rectangular)]
              |
              +---> [Irregular edges / no-edge / all-edge variants]
              |
              +---> [Custom border shapes] (requires boundary clipping)
              |
              +---> [Whimsy pieces - preset library]
                        |
                        +---> [Whimsy pieces - user SVG import]
                        |
                        +---> [Multi-piece whimsy] (requires multi-cell replacement)

[Configuration sharing via URL] --independent--> (no dependencies)
```

### Dependency Notes

- **Multiple connector types requires abstract edge interface:** The edge generation must be pluggable from day one. Don't hardcode the classic knob — define an edge generator trait/interface and implement classic knob as the first implementation.
- **Whimsy pieces require working grid + connectors:** Can't place figural shapes until the base grid and connector routing works correctly. Whimsy shapes must integrate with the connector system (re-route edges around the shape outline).
- **Custom border shapes require boundary clipping:** Must first have rectangular borders working, then generalize to arbitrary paths with grid clipping.
- **Kerf compensation requires path offset:** This is a post-processing step on the final SVG paths. Doesn't affect generation logic, but needs robust path offset algorithms.
- **Irregular/no-edge/all-edge are border variants:** Simple flags on the border generation step. Low complexity once borders work.

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the concept and be immediately useful for laser cutting.

- [ ] Configurable grid (rows x columns) — core generation parameter
- [ ] Configurable puzzle dimensions (mm, with inch conversion) — physical output requires physical units
- [ ] Classic knob connector with procedural randomization — the fundamental puzzle shape
- [ ] Tab size and jitter controls — users need to tune piece aesthetics
- [ ] Seed-based reproducibility — essential for sharing and re-cutting
- [ ] SVG export with laser-cutter compatible strokes — the deliverable
- [ ] Rounded corner radius on border — prevents fragile corners
- [ ] Web GUI with live preview — the interface
- [ ] Piece count display — low-effort high-value UI element

### Add After Validation (v1.x)

Features to add once core is working and users confirm value.

- [ ] Irregular edge / no-edge / all-edge border variants — low complexity, adds variety. Trigger: users asking for harder puzzles.
- [ ] Multiple connector types beyond classic knob — medium complexity, requires edge abstraction to already be clean. Trigger: users wanting visual variety.
- [ ] Laser-cutter stroke presets (Glowforge, LightBurn) — low complexity, high polish. Trigger: users manually adjusting stroke settings.
- [ ] Configuration sharing via URL — low complexity, high sharing value. Trigger: users asking "how do I share this puzzle?"
- [ ] Kerf compensation — medium complexity, high manufacturing value. Trigger: users reporting loose-fitting pieces.

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] Whimsy pieces from preset shape library — high complexity, highest differentiator. Defer because: requires robust grid generation first, complex connector re-routing algorithm, shape library curation.
- [ ] Whimsy pieces from user SVG import — high complexity. Defer because: depends on preset whimsy, adds SVG parsing/validation pipeline.
- [ ] Custom border shapes (non-rectangular outlines) — high complexity. Defer because: requires grid-boundary clipping, partial piece handling.
- [ ] Multi-piece whimsy — very high complexity. Defer because: most complex feature, depends on single-cell whimsy working first.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Grid dimensions (rows x cols) | HIGH | LOW | P1 |
| Puzzle size (mm/inches) | HIGH | LOW | P1 |
| Classic knob connector | HIGH | MEDIUM | P1 |
| Per-edge randomization | HIGH | MEDIUM | P1 |
| Seed-based reproducibility | HIGH | LOW | P1 |
| Tab size control | MEDIUM | LOW | P1 |
| Jitter control | MEDIUM | LOW | P1 |
| SVG export (laser-compatible) | HIGH | MEDIUM | P1 |
| Live preview | HIGH | MEDIUM | P1 |
| Corner radius | MEDIUM | LOW | P1 |
| Web GUI | HIGH | MEDIUM | P1 |
| Piece count display | LOW | LOW | P1 |
| Irregular/no-edge/all-edge variants | MEDIUM | LOW | P2 |
| Multiple connector types | MEDIUM | MEDIUM | P2 |
| Laser-cutter presets | MEDIUM | LOW | P2 |
| URL config sharing | MEDIUM | LOW | P2 |
| Kerf compensation | MEDIUM | MEDIUM | P2 |
| Whimsy pieces (preset) | HIGH | HIGH | P3 |
| Whimsy pieces (user SVG) | MEDIUM | HIGH | P3 |
| Custom border shapes | MEDIUM | HIGH | P3 |
| Multi-piece whimsy | LOW | VERY HIGH | P3 |

**Priority key:**
- P1: Must have for launch — the baseline that makes this a usable puzzle generator
- P2: Should have, add when possible — polish and variety features
- P3: Nice to have, future consideration — the differentiators that make this unique (but require solid foundation first)

## Competitor Feature Analysis

| Feature | Draradech/jigsaw (269 stars) | astbis/laser-jigsaw-generator | MB-Deen/jigsaw-svg-generator | Our Approach |
|---------|-----|-----|-----|-----|
| Grid config | Yes (text inputs) | Yes (text inputs) | Yes (text inputs) | Yes, with sliders + text inputs |
| Puzzle size | mm only | mm only | mm only | mm + inches toggle |
| Connector shape | Classic knob only | Classic knob only | Classic knob only | Classic knob first, pluggable for more types |
| Randomization | Seed + jitter | Seed + jitter | Seed + jitter | Seed + jitter + per-connector-type params |
| SVG export | Basic (0.1mm strokes) | Laser-optimized (0.01mm, color-coded) | Basic (0.1mm strokes) | Laser-optimized with named presets |
| Live preview | Yes (inline SVG) | Yes (inline SVG) | Yes (inline SVG) | Yes (WASM-rendered SVG) |
| Hexagonal grid | Yes (separate page) | No | Yes (separate page) | Future consideration |
| Text labels on pieces | No | Yes (educational use) | No | Not planned (anti-feature: different product) |
| Whimsy pieces | No | No | No | **Planned (P3) — no competitor has this** |
| Custom borders | No | No | No | **Planned (P3) — no competitor has this** |
| Multiple connectors | No | No | No | **Planned (P2) — no competitor has this** |
| Edge variants | No | No | No | **Planned (P2) — no competitor has this** |
| URL sharing | No | No | No | **Planned (P2)** |
| Kerf compensation | No | No | No | **Planned (P2) — high manufacturing value** |
| Technology | Vanilla JS (~300 LOC) | Vanilla JS (fork) | Vanilla JS (fork) | Rust + WASM (performance for complex generation) |
| Performance | Fine for small grids | Fine for small grids | Fine for small grids | Handles large grids (50x50+) via WASM |

### Competitive Summary

The existing landscape is remarkably homogeneous: every open-source jigsaw generator is a fork or spiritual descendant of Draradech's ~300-line vanilla JavaScript implementation. They all share the same limitations:
- Only classic knob connectors
- Only rectangular grids (hex in some)
- No whimsy/figural pieces
- No custom borders
- No edge variants
- No kerf compensation
- No connector variety
- Limited to small grids (performance)

**Our biggest differentiators** are whimsy pieces (no competitor has this at all) and the Rust/WASM architecture enabling complex generation at scale. Even P2 features like multiple connector types and edge variants would be unique in the open-source space.

## Sources

- **Draradech/jigsaw** (269 stars, 107 forks) — https://github.com/Draradech/jigsaw — Most popular open-source implementation, CC0 license. Source code analyzed for connector algorithm (cubic bezier with 10 control points per edge). [HIGH confidence]
- **astbis/laser-jigsaw-generator** — https://github.com/astbis/laser-jigsaw-generator — Laser-cutting optimized fork with educational labels, stroke width presets. [HIGH confidence]
- **MB-Deen/jigsaw-svg-generator** — https://github.com/MB-Deen/jigsaw-svg-generator — UI-improved fork with hex support. [HIGH confidence]
- **zvikabh/jigsaw-puzzle-svg** — https://github.com/zvikabh/jigsaw-puzzle-svg — Python + JS implementation. [MEDIUM confidence — minimal documentation]
- **Wikipedia: Jigsaw puzzle** — https://en.wikipedia.org/wiki/Jigsaw_puzzle — Whimsy piece terminology ("whimsies", "silhouettes", "figurals"), modern construction methods, laser cutting mention. [HIGH confidence]
- **GitHub search: "jigsaw puzzle SVG generator"** — Only 4 public repos total. Niche with very few competitors. [HIGH confidence]

---
*Feature research for: Procedural jigsaw puzzle pattern generation*
*Researched: 2026-03-01*
