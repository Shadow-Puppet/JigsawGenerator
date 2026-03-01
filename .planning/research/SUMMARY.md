# Project Research Summary

**Project:** Puzzle Pattern Generator
**Domain:** Procedural jigsaw puzzle SVG generation (Rust/WASM + web GUI for laser cutting)
**Researched:** 2026-03-01
**Confidence:** HIGH

## Executive Summary

This is a procedural geometry tool that generates laser-cuttable jigsaw puzzle SVG patterns. The existing open-source landscape is tiny and homogeneous — every competitor is a ~300-line vanilla JavaScript fork of the same codebase (Draradech/jigsaw), limited to basic knob connectors on rectangular grids with no whimsy pieces, no connector variety, and no kerf compensation. The Rust/WASM architecture gives us a genuine performance edge for complex generation (large grids, whimsy piece routing) while the web GUI keeps it zero-install. The recommended approach is: Rust core with `kurbo` for 2D Bezier math, `rand_chacha` for deterministic seeded RNG, vanilla TypeScript frontend with Vite, and SVG string return across the WASM boundary as the single data exchange pattern.

The architecture is a clean pipeline: Grid Layout → Edge Assignment → Connector Generation → SVG Assembly, with the `PuzzleGenerator` struct living in WASM linear memory and JavaScript holding an opaque handle. The critical design decisions that must be locked in from day one are: (1) shared-edge data model so adjacent pieces reference the same path data (prevents floating-point gaps), (2) sub-stream RNG forking from a master seed (prevents seed breakage when features are added), (3) connector trait abstraction even with only one connector type (enables future extensibility), and (4) strict SVG output subset targeting laser cutter compatibility (absolute coordinates, explicit physical units, inline stroke attributes, no CSS/transforms).

The top risks are connector geometry that doesn't physically interlock (randomized Bezier parameters exceeding valid bounds), floating-point precision causing gaps between adjacent pieces, and SVG output that renders in browsers but fails in laser cutter software. All three are preventable with upfront architectural decisions rather than post-hoc fixes. The WASM bundle size is a secondary risk managed by build configuration (`opt-level = 's'`, LTO, size auditing with `twiggy`). The project is well-scoped for iterative delivery: a solid MVP with grid generation, classic knob connectors, and SVG export is achievable as a first phase, with differentiating features (whimsy pieces, multiple connector types) layered on top of a proven foundation.

## Key Findings

### Recommended Stack

The stack is Rust-native for computation with a thin TypeScript GUI shell. Every library choice has been verified for WASM compatibility and current version availability. See [STACK.md](STACK.md) for full details.

**Core technologies:**
- **Rust 1.93+ → WASM**: Core puzzle generation engine. Memory safety, zero-cost abstractions, first-class `wasm32-unknown-unknown` target.
- **kurbo 0.13.0**: Primary 2D geometry library — `CubicBez`, `BezPath`, `Affine` transforms, f64 precision. From the Linebender project. Purpose-built for 2D vector graphics.
- **rand_chacha 0.10.0 (ChaCha8Rng)**: Deterministic cross-platform seeded RNG. Same seed = same puzzle on any machine. Critical for reproducibility.
- **wasm-bindgen 0.2.114 + serde-wasm-bindgen**: Rust↔JS FFI with automatic TypeScript declarations. Direct JsValue conversion without JSON string intermediary.
- **Vanilla TypeScript + Vite 7.3.x**: No framework. The UI is ~10 controls + SVG preview pane. A framework would add bundle bloat for zero benefit.
- **svg 0.18.0 (or manual string generation)**: SVG document construction. Manual string generation is a viable alternative for minimal WASM size.

**Critical version constraint:** `rand 0.10` + `rand_chacha 0.10` + `getrandom 0.4` with `wasm_js` feature — these must be aligned or WASM compilation fails.

### Expected Features

The competitive landscape is sparse and uniform. All 4 open-source competitors share identical limitations. See [FEATURES.md](FEATURES.md) for full analysis.

**Must have (table stakes — every competitor has these):**
- Configurable grid (rows × columns) with puzzle dimensions in mm/inches
- Classic knob connector shape with procedural per-edge randomization
- Seed-based reproducibility for sharing/re-cutting
- Tab size and jitter controls
- SVG export with laser-cutter compatible strokes (no-fill, thin stroke, physical units)
- Web GUI with live preview and rounded corner radius on border

**Should have (differentiators — no competitor has any of these):**
- Multiple connector types beyond classic knob (pluggable edge generator architecture)
- Irregular/no-edge/all-edge border variants (simple flags, low complexity)
- Laser-cutter stroke presets (Glowforge, LightBurn one-click configs)
- Configuration sharing via URL encoding
- Kerf compensation (path offset for laser beam width)

**Defer to v2+ (high complexity, requires solid foundation):**
- Whimsy/figural pieces from preset library — highest differentiator but requires connector re-routing around arbitrary shapes
- Whimsy from user-imported SVG — depends on preset whimsy + SVG validation pipeline
- Custom border shapes (non-rectangular) — requires grid-boundary clipping
- Multi-piece whimsy spanning grid cells — most complex feature in the entire product

### Architecture Approach

The system follows a pipeline architecture with a clear WASM boundary: config goes in, SVG string comes out. All computation happens in Rust; JavaScript handles only UI and rendering. The `PuzzleGenerator` struct lives in WASM linear memory as an opaque handle. See [ARCHITECTURE.md](ARCHITECTURE.md) for full details.

**Major components:**
1. **PuzzleGenerator** — Top-level orchestrator. Owns config, seeds RNG, drives the pipeline.
2. **GridLayout** — Computes cell boundaries from puzzle dimensions + grid size.
3. **EdgeAssignment** — Assigns tab/blank/flat to each edge using seeded RNG. Enforces mating constraints.
4. **ConnectorGenerator** — Generates Bezier control points for connector shapes. Trait-based for extensibility.
5. **SVGPathAssembler** — Converts abstract path commands to SVG `d` attribute strings. Builds complete SVG document.
6. **Web GUI** — Vanilla TS controls + SVG preview via `innerHTML` injection.

**Project structure:** Cargo workspace with `crates/puzzle-core/` (Rust library) and `web/` (TypeScript frontend). Connector types in dedicated `connector/` module with trait dispatch. Geometry in pure-math `geometry/` module independent of WASM.

### Critical Pitfalls

See [PITFALLS.md](PITFALLS.md) for all 6 critical pitfalls with detailed prevention strategies.

1. **Connector geometry that doesn't physically interlock** — Clamped randomization ranges within validated bounds. Model connectors as mating pairs from shared parameters. Property-based tests for non-self-intersection and minimum feature width.
2. **Floating-point gaps between adjacent pieces** — Shared-edge architecture where each internal edge exists once in memory, referenced (normal/reversed) by both adjacent pieces. Must be designed into the data model from day one; retrofitting is a rewrite.
3. **SVG incompatible with laser cutter software** — Output strict SVG subset: only `<path>` elements, absolute coordinates, inline attributes, explicit physical units, no CSS/transforms. Test in LightBurn early.
4. **WASM bundle size explosion** — `opt-level = 's'`, LTO, `codegen-units = 1`, prefer `Result` over `unwrap()`. Budget: <500KB gzipped. Monitor with `twiggy` from first build.
5. **Grid boundary condition failures** — Build and test border/corner pieces simultaneously with interior pieces, not after. Test with 1×1, 2×2, 1×N grids that are all-boundary.
6. **Seed reproducibility broken by algorithm changes** — Fork independent sub-RNGs from master seed for each pipeline stage. Version the generation algorithm. Pin reference seed tests in CI.

## Implications for Roadmap

Based on combined research, the architecture has a clear sequential dependency chain that dictates phase ordering: geometry primitives → grid → edges → connectors → SVG → WASM bridge → web GUI. The feature research shows MVP features are all P1 (table stakes), with differentiators cleanly layered as P2/P3 on top.

### Phase 1: Project Scaffolding & Build Pipeline
**Rationale:** WASM build configuration is where most Rust-WASM projects fail first. `getrandom` feature flags, `crate-type`, release profile optimization, and Vite WASM integration must work before any logic is written. PITFALLS.md explicitly flags bundle size as a Phase 1 concern.
**Delivers:** Working Rust→WASM→Vite build pipeline with "hello world" round-trip. TypeScript can call a Rust function and get a result. Optimized release profile. Size monitoring.
**Addresses features:** None directly (infrastructure).
**Avoids pitfalls:** WASM bundle size explosion (#4), `getrandom` compilation failure.

### Phase 2: Core Geometry Engine & Grid Layout
**Rationale:** Everything builds on geometry primitives and grid layout. ARCHITECTURE.md identifies these as the first two pipeline stages. The shared-edge data model MUST be designed here — PITFALLS.md rates retrofitting as HIGH recovery cost.
**Delivers:** `Point2D`, `CubicBezier`, `PathCommand` types. `GridLayout` computing cell boundaries. `EdgeAssignment` with shared-edge architecture. Seeded RNG pipeline with sub-stream forking.
**Addresses features:** Configurable grid (rows × cols), configurable puzzle dimensions (mm/inches), seed-based reproducibility.
**Avoids pitfalls:** Floating-point gaps (#2), boundary condition bugs (#5), seed reproducibility (#6).

### Phase 3: Connector Generation & SVG Output
**Rationale:** With grid + edges in place, connector generation produces the actual puzzle geometry. SVG assembly converts it to output. These are tightly coupled — you can't validate connectors without seeing SVG output. PITFALLS.md identifies connector validity as the #1 domain-specific failure.
**Delivers:** Classic knob connector via trait-based `ConnectorGenerator`. Full SVG document assembly with laser-cutter-compatible strict subset. Complete generation pipeline: config → SVG string.
**Addresses features:** Classic knob connector with randomization, tab size/jitter controls, SVG export (laser-compatible), rounded corner radius.
**Avoids pitfalls:** Connector geometry invalidity (#1), SVG laser incompatibility (#3).

### Phase 4: Web GUI & Live Preview
**Rationale:** With the Rust pipeline producing valid SVG, the GUI wires it up. The WASM bridge (`PuzzleGenerator` opaque handle) connects config controls to generation. Live preview is the primary user interaction.
**Delivers:** Parameter controls (sliders + inputs), live SVG preview via `innerHTML`, SVG file download, seed display/input, piece count display.
**Addresses features:** Web GUI, live preview, piece count display, all remaining P1 table-stakes features.
**Avoids pitfalls:** Preview/export mismatch (same SVG string for both).

### Phase 5: Polish & Variant Features
**Rationale:** With working MVP, add low-complexity differentiators. Border variants are simple flags. URL sharing is independent. Laser presets are output-format-only changes. These features are low-risk and don't require architectural changes.
**Delivers:** Irregular/no-edge/all-edge border variants, laser-cutter stroke presets, configuration sharing via URL, kerf compensation.
**Addresses features:** All P2 differentiators from FEATURES.md.
**Avoids pitfalls:** None new — validates existing architecture handles variants.

### Phase 6: Advanced Features (Whimsy & Custom Borders)
**Rationale:** Highest complexity, highest differentiation. Requires solid foundation. Whimsy pieces need connector re-routing around arbitrary shapes — the most algorithmically complex feature. Custom borders need grid-boundary clipping. Both depend on every prior phase being robust.
**Delivers:** Whimsy pieces from preset shape library, multiple connector types, custom border shapes. (User SVG import and multi-piece whimsy deferred further.)
**Addresses features:** All P3 features from FEATURES.md.
**Avoids pitfalls:** Whimsy integration pitfall (#9 from PITFALLS.md) — connector constraints must apply to whimsy boundaries too.

### Phase Ordering Rationale

- **Sequential dependency chain is firm:** Geometry → Grid → Edges → Connectors → SVG → GUI. ARCHITECTURE.md's build order analysis confirms this. Phases 1-4 cannot be reordered.
- **Shared-edge architecture is a day-one decision:** PITFALLS.md rates the recovery cost as HIGH if retrofitted. Phase 2 must get this right.
- **Connector trait abstraction pays for itself in Phase 5+:** Even though only classic knob ships in Phase 3, the `ConnectorStyle` enum pattern enables Phase 5's multiple connector types without refactoring.
- **Phase 5 features are independent of each other:** Border variants, URL sharing, laser presets, and kerf compensation can ship incrementally in any order.
- **Whimsy (Phase 6) is deliberately last:** It's the most complex feature with the highest risk, but also the highest differentiator. It should only be attempted on a proven, well-tested foundation.

### Research Flags

**Phases likely needing deeper research during planning:**
- **Phase 3 (Connectors):** The classic knob connector algorithm (cubic Bezier with 10 control points per edge, from Draradech) needs implementation validation. The constraint system for physically valid connectors has no reference implementation — it's domain-specific and needs experimentation.
- **Phase 6 (Whimsy):** No existing open-source implementation exists. The connector re-routing algorithm around arbitrary shapes is novel. Will likely need iterative prototyping and visual validation.

**Phases with standard patterns (skip deep research):**
- **Phase 1 (Build Pipeline):** wasm-pack + Vite is well-documented with official tutorials. The `getrandom` `wasm_js` feature flag is the only gotcha, and it's documented.
- **Phase 2 (Grid/Edges):** Standard computational geometry. Grid layout and edge assignment are straightforward algorithms.
- **Phase 4 (Web GUI):** Vanilla TS + DOM manipulation. No framework complexity. The WASM bridge pattern is canonical from wasm-bindgen docs.
- **Phase 5 (Polish):** All features in this phase are low-complexity extensions of existing patterns.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crate versions verified on crates.io. WASM compatibility confirmed. Vite + wasm-pack integration well-documented. |
| Features | HIGH | Feature landscape analyzed against 4 open-source competitors. Table stakes verified against all. Differentiators confirmed as absent in all competitors. |
| Architecture | HIGH | Pipeline pattern is canonical for Rust-WASM apps (official Rust-WASM book). Data flow is straightforward. Component boundaries are clean. |
| Pitfalls | HIGH | Floating-point, SVG compatibility, and WASM size are well-documented domain issues. Connector geometry pitfalls confirmed by analyzing Draradech implementation. |

**Overall confidence:** HIGH

### Gaps to Address

- **Connector constraint bounds:** The exact numeric ranges for "physically valid" knob connectors (min neck width, max head diameter, etc.) are not documented anywhere. Need empirical testing with actual laser-cut materials to establish good defaults. **Handle during Phase 3:** Start with Draradech's proven control points, then iterate with test cuts.
- **`serde-wasm-bindgen` version pinning:** Recommended by wasm-bindgen docs but exact latest version was not independently verified. **Handle during Phase 1:** Pin during initial `Cargo.toml` setup, verify compiles.
- **Laser cutter software SVG subset differences:** LightBurn, Glowforge, and Epilog may have subtly different SVG parser behaviors. Research covered general patterns but not per-tool quirks. **Handle during Phase 3:** Test exported SVG in at least LightBurn (free trial) before shipping.
- **Performance at extreme scale (5000+ pieces):** Architecture supports Web Worker offloading but no benchmarks exist. **Handle during Phase 5:** Add benchmarks; implement Web Worker only if measured latency exceeds 100ms for target puzzle sizes.
- **`kurbo` vs manual geometry types:** ARCHITECTURE.md suggests custom `Point2D`/`CubicBezier` types while STACK.md recommends `kurbo`. **Resolution: Use `kurbo`'s types directly.** They're purpose-built, f64, serde-compatible, and avoid reimplementing Bezier math. The `geometry/` module wraps `kurbo` types with project-specific utilities, not replacements.

## Sources

### Primary (HIGH confidence)
- crates.io API — verified all Rust crate versions and last-updated dates (2026-03-01)
- docs.rs/kurbo/0.13.0 — API surface: BezPath, CubicBez, Point, Affine, SVG path support
- docs.rs/rand/0.10.0, docs.rs/rand_chacha/0.10.0 — RNG traits, SeedableRng, ChaCha8Rng
- wasm-bindgen 0.2.114 docs — `--target web` ESM output, TypeScript generation, opaque handle pattern
- Rust and WebAssembly Book — architecture patterns, code size optimization, JS FFI, crate compatibility
- npmjs.org — Vite 7.3.1, TypeScript 5.9.3, vite-plugin-wasm 3.5.0
- Draradech/jigsaw (269 stars) — connector algorithm analysis (cubic Bezier, 10 control points)
- astbis/laser-jigsaw-generator — laser-optimized SVG output patterns
- MDN SVG Path reference — path command syntax for laser compatibility

### Secondary (MEDIUM confidence)
- serde-wasm-bindgen docs — recommended by wasm-bindgen for direct JsValue conversion
- Wikipedia: Jigsaw puzzle — whimsy piece terminology, physical construction methods
- Connector geometry constraints — computational geometry domain knowledge, needs empirical validation
- Laser cutter SVG compatibility — maker community patterns, corroborated by multiple sources

### Tertiary (LOW confidence)
- noise 0.9.0 — deferred to later phases; last updated Mar 2024, functional but not actively developed
- Multi-piece whimsy algorithmic approach — no reference implementation exists; needs prototyping

---
*Research completed: 2026-03-01*
*Ready for roadmap: yes*
