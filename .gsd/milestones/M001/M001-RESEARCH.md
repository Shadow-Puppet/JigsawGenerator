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

# Architecture Research

**Domain:** Procedural jigsaw puzzle pattern generation (Rust/WASM + Web GUI)
**Researched:** 2026-03-01
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
Browser Tab
===========================================================================
|  +--------------------------+        +-------------------------------+   |
|  |     Web GUI (JS/TS)      |        |     WASM Module (Rust)        |   |
|  |                          |        |                               |   |
|  |  +--------------------+  |        |  +-------------------------+  |   |
|  |  | Parameter Controls |--+------->|  | PuzzleGenerator         |  |   |
|  |  | (dimensions, seed, |  | config |  | - grid layout           |  |   |
|  |  |  connector style)  |  |        |  | - edge assignment       |  |   |
|  |  +--------------------+  |        |  | - connector generation  |  |   |
|  |                          |        |  | - SVG path assembly     |  |   |
|  |  +--------------------+  |        |  +------------+------------+  |   |
|  |  | SVG Preview Canvas |<-+--------+               |               |   |
|  |  | (live render)      |  | svg str|  +------------v------------+  |   |
|  |  +--------------------+  |        |  | Geometry Engine          |  |   |
|  |                          |        |  | - bezier curves          |  |   |
|  |  +--------------------+  |        |  | - path validation        |  |   |
|  |  | Export Controls    |--+------->|  | - coordinate transforms  |  |   |
|  |  | (download SVG)     |  | export |  +-------------------------+  |   |
|  |  +--------------------+  |        |                               |   |
|  +--------------------------+        +-------------------------------+   |
|                                                                          |
|  +--------------------------------------------------------------------+  |
|  |                    WASM Boundary (wasm-bindgen)                     |  |
|  |  - Config structs passed via serde-wasm-bindgen                    |  |
|  |  - SVG string returned as String (single copy across boundary)     |  |
|  |  - Opaque handle to PuzzleGenerator lives in WASM linear memory    |  |
|  +--------------------------------------------------------------------+  |
===========================================================================
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| **PuzzleGenerator** (Rust) | Owns all puzzle state. Accepts config, produces SVG string. Orchestrates grid layout, edge assignment, and path generation. | `#[wasm_bindgen] pub struct PuzzleGenerator` with methods `new(config)`, `generate() -> String`, `update_config(config)` |
| **GridLayout** (Rust) | Computes piece boundaries on the grid. Maps (row, col) to physical coordinates. Handles piece sizing within puzzle dimensions. | Pure Rust module. Takes puzzle dimensions + grid size, returns cell boundary coordinates. |
| **EdgeAssignment** (Rust) | Decides which edges get connectors (tab/blank) vs flat edges (borders). Uses seeded RNG for deterministic assignment. Ensures mating: if edge A has a tab, adjacent edge B has a blank. | Seeded `rand::rngs::StdRng` or `rand_chacha::ChaCha8Rng`. Iterates grid edges, assigns in/out/flat. |
| **ConnectorGenerator** (Rust) | Generates the actual bezier curve control points for each connector shape (knob, etc). Applies per-piece randomized variation within constraints. | Pure geometry module. Takes edge endpoints + connector type + RNG, returns sequence of cubic bezier control points. |
| **SVGPathAssembler** (Rust) | Converts geometry primitives (lines, beziers) into SVG path `d` attribute strings. Assembles complete SVG document with proper viewBox, units. | String builder. Outputs `M`, `C`, `L`, `Z` commands. Handles coordinate precision for laser cutter compatibility. |
| **GeometryValidator** (Rust) | Validates paths don't self-intersect, connectors mate properly, no gaps exist between adjacent pieces. | Runs during generation. Can be debug-only for performance. |
| **Web GUI** (JS/TS) | User-facing controls for puzzle configuration. Renders SVG preview. Handles export/download. | Lightweight framework or vanilla JS. Inline SVG rendering for preview. Debounced parameter changes trigger regeneration. |
| **WASM Bridge** | Serializes config from JS to Rust, returns SVG string from Rust to JS. Manages WASM module lifecycle. | `wasm-bindgen` + `serde-wasm-bindgen` for config structs. SVG returned as `String` (auto-converted to JS string). |

## Recommended Project Structure

```
puzzle-generator/
├── crates/
│   └── puzzle-core/           # Rust library crate (compiles to WASM)
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs          # WASM entry point, #[wasm_bindgen] exports
│       │   ├── config.rs       # PuzzleConfig, ConnectorStyle enums (serde)
│       │   ├── grid.rs         # GridLayout: piece boundary computation
│       │   ├── edge.rs         # EdgeAssignment: tab/blank/flat assignment
│       │   ├── connector/
│       │   │   ├── mod.rs      # ConnectorGenerator trait + dispatch
│       │   │   ├── knob.rs     # Classic knob connector bezier generation
│       │   │   └── types.rs    # Connector type definitions
│       │   ├── geometry/
│       │   │   ├── mod.rs      # Geometry primitives
│       │   │   ├── point.rs    # Point2D, Vec2D
│       │   │   ├── bezier.rs   # CubicBezier, bezier utilities
│       │   │   ├── path.rs     # Path: sequence of path commands
│       │   │   └── validate.rs # Geometric validation
│       │   ├── svg/
│       │   │   ├── mod.rs      # SVG document assembly
│       │   │   ├── path.rs     # SVG path `d` attribute builder
│       │   │   └── document.rs # Full SVG document with viewBox, units
│       │   ├── puzzle.rs       # PuzzleGenerator: top-level orchestrator
│       │   └── rng.rs          # Seeded RNG wrapper, seed management
│       └── tests/
│           ├── grid_tests.rs
│           ├── edge_tests.rs
│           ├── connector_tests.rs
│           └── integration.rs  # Full puzzle generation tests
├── web/                        # Web frontend
│   ├── index.html
│   ├── src/
│   │   ├── main.ts             # App entry, WASM init
│   │   ├── controls.ts         # Parameter UI controls
│   │   ├── preview.ts          # SVG preview rendering
│   │   └── export.ts           # SVG download/export
│   ├── styles/
│   │   └── main.css
│   ├── package.json
│   └── vite.config.ts          # Or similar bundler config
├── Cargo.toml                  # Workspace root
└── .planning/
```

### Structure Rationale

- **`crates/puzzle-core/`:** Monorepo workspace pattern. The Rust library is its own crate that compiles to both native (for testing) and `wasm32-unknown-unknown`. Keeping it in `crates/` leaves room for future crates (e.g., a CLI tool).
- **`connector/` module:** Connectors are the most complex and extensible part. Each connector type (knob, wave, etc.) gets its own file implementing a common trait. This is the primary extension point.
- **`geometry/` module:** Pure math, no WASM dependencies. Custom `Point2D` and `CubicBezier` types avoid pulling in heavy geometry crates. These types are small and purpose-built.
- **`svg/` module:** Separate from geometry because SVG is an output format concern. The geometry engine doesn't know about SVG; it produces abstract path commands that the SVG module serializes.
- **`web/`:** Separate from Rust. Uses its own package.json and build tooling. Imports the WASM package as a dependency (built by `wasm-pack`).

## Architectural Patterns

### Pattern 1: Opaque Handle with Method Calls

**What:** The `PuzzleGenerator` struct lives in WASM linear memory. JavaScript holds an opaque handle to it and calls methods on it. This avoids copying the entire puzzle state across the WASM boundary on every interaction.
**When to use:** Always, for the core generator. This is the canonical Rust-WASM pattern from the official `wasm-bindgen` documentation.
**Trade-offs:** Excellent performance (no serialization of internal state), but JavaScript can't inspect internal state without explicit getter methods.

**Example:**
```rust
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct PuzzleConfig {
    pub rows: u32,
    pub cols: u32,
    pub width_mm: f64,
    pub height_mm: f64,
    pub seed: u64,
    pub connector_style: ConnectorStyle,
}

#[wasm_bindgen]
pub struct PuzzleGenerator {
    config: PuzzleConfig,
    // internal state lives here in WASM memory
}

#[wasm_bindgen]
impl PuzzleGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<PuzzleGenerator, JsError> {
        let config: PuzzleConfig = serde_wasm_bindgen::from_value(config)?;
        Ok(PuzzleGenerator { config })
    }

    pub fn generate(&self) -> String {
        // All heavy computation happens here in WASM
        // Returns SVG string - single copy across boundary
        todo!()
    }

    pub fn update_config(&mut self, config: JsValue) -> Result<(), JsError> {
        self.config = serde_wasm_bindgen::from_value(config)?;
        Ok(())
    }
}
```

### Pattern 2: Seeded Deterministic Pipeline

**What:** Every random decision in the generation pipeline flows from a single seed. The seed initializes an RNG, which is consumed in a deterministic order: grid layout perturbations, edge assignment, connector variation. Same seed = same puzzle, always.
**When to use:** Core to the product requirement of reproducible puzzles.
**Trade-offs:** Must be disciplined about RNG consumption order. Adding new randomized features later changes output for existing seeds unless carefully managed with sub-seeds.

**Example:**
```rust
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub struct SeededPipeline {
    master_rng: ChaCha8Rng,
}

impl SeededPipeline {
    pub fn new(seed: u64) -> Self {
        Self {
            master_rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn generate(&mut self) -> Puzzle {
        // Fork RNGs for each phase to isolate changes
        let mut grid_rng = ChaCha8Rng::from_rng(&mut self.master_rng);
        let mut edge_rng = ChaCha8Rng::from_rng(&mut self.master_rng);
        let mut connector_rng = ChaCha8Rng::from_rng(&mut self.master_rng);

        let grid = GridLayout::generate(&self.config, &mut grid_rng);
        let edges = EdgeAssignment::assign(&grid, &mut edge_rng);
        let paths = ConnectorGenerator::generate(&edges, &mut connector_rng);
        // ...
    }
}
```

### Pattern 3: SVG String as Single Return Value

**What:** The WASM module returns a complete SVG document as a `String`. JavaScript renders it by setting `innerHTML` of a container element (for preview) or creating a Blob for download. No incremental path data crosses the boundary.
**When to use:** For this project, always. SVG strings are compact (even large puzzles produce manageable strings), and the single-copy-across-boundary approach is the most performant WASM pattern.
**Trade-offs:** Cannot do partial/incremental updates. Every config change regenerates the full SVG. This is acceptable because generation should be fast (<100ms for typical puzzles).

**Example (JS side):**
```typescript
// Preview: inject SVG directly into DOM
const svgString = generator.generate();
previewContainer.innerHTML = svgString;

// Export: create downloadable file
const blob = new Blob([svgString], { type: 'image/svg+xml' });
const url = URL.createObjectURL(blob);
downloadLink.href = url;
```

### Pattern 4: Connector Trait for Extensibility

**What:** Define a `ConnectorShape` trait that each connector type implements. The generator dispatches to the correct implementation based on config. New connector types are added by implementing the trait.
**When to use:** From the start. Even with only one connector type (classic knob), the trait boundary ensures clean separation and makes adding new types trivial.
**Trade-offs:** Slight indirection. Use static dispatch (`enum` + match) rather than dynamic dispatch (`dyn Trait`) to avoid vtable overhead in WASM.

**Example:**
```rust
pub enum ConnectorStyle {
    ClassicKnob,
    // Future: Wave, Tab, Custom...
}

pub struct ConnectorParams {
    pub start: Point2D,
    pub end: Point2D,
    pub direction: TabDirection, // In or Out
    pub variation: f64,          // 0.0-1.0 from RNG
}

impl ConnectorStyle {
    pub fn generate_path(&self, params: &ConnectorParams) -> Vec<PathCommand> {
        match self {
            ConnectorStyle::ClassicKnob => knob::generate(params),
        }
    }
}
```

## Data Flow

### Generation Flow (Primary)

```
User adjusts parameters in Web GUI
    |
    v
JS: debounce(50ms) -> serialize config via serde-wasm-bindgen
    |
    v
WASM: PuzzleGenerator.update_config(config) + PuzzleGenerator.generate()
    |
    v
Rust: Seed RNG from config.seed
    |
    +---> GridLayout: compute cell boundaries from dimensions + grid size
    |         |
    |         v
    +---> EdgeAssignment: iterate all internal + border edges
    |     - Internal edges: randomly assign tab direction (in/out)
    |     - Border edges: assign flat (or connector for no-edge puzzles)
    |     - Ensure mating: adjacent edges get complementary directions
    |         |
    |         v
    +---> ConnectorGenerator: for each non-flat edge
    |     - Compute bezier control points for connector shape
    |     - Apply per-edge random variation (size, curvature, offset)
    |     - Return Vec<PathCommand> (MoveTo, CubicTo, LineTo, ClosePath)
    |         |
    |         v
    +---> SVGPathAssembler: for each piece
          - Walk piece boundary (4 edges for interior pieces)
          - Concatenate edge paths into closed piece outline
          - Build SVG path `d` attribute string
          - Assemble all pieces into SVG document
              |
              v
WASM -> JS: Return SVG String (single copy across boundary)
    |
    v
JS: Set innerHTML for preview / Create Blob for export
```

### Key Data Structures Flowing Through Pipeline

```
PuzzleConfig (JS -> WASM)
    |
    v
Grid<CellBounds>  (internal: row/col -> physical coordinates)
    |
    v
EdgeMap<EdgeId, EdgeInfo>  (internal: edge -> type + direction + endpoints)
    |
    v
Vec<PiecePath>  (internal: piece -> closed path of PathCommands)
    |
    v
String (SVG document)  (WASM -> JS)
```

### State Management

The Rust `PuzzleGenerator` is the single source of truth. There is no duplicated state in JavaScript. The GUI reads the current config to populate controls and sends updated configs to WASM. The flow is strictly unidirectional:

```
GUI Controls --[config]--> PuzzleGenerator --[SVG string]--> Preview/Export
```

No state synchronization needed. No pub/sub. No reactive stores. The WASM module is a pure function-like transform: config in, SVG out.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| Small puzzles (< 100 pieces) | No concerns. Generation is instantaneous. Direct SVG string return works perfectly. |
| Medium puzzles (100-1000 pieces) | Still fine. SVG strings may be 100KB-1MB. Ensure debouncing on parameter changes to avoid unnecessary regeneration during slider drags. |
| Large puzzles (1000-5000 pieces) | SVG string can reach several MB. Consider: (1) Web Worker for WASM execution to avoid blocking main thread, (2) generation progress callback, (3) SVG simplification for preview (fewer bezier segments) with full detail on export. |
| Extreme puzzles (5000+) | Likely needs: chunked generation, viewport-based partial rendering for preview, and background generation. This is out of scope for v1 but the architecture supports it - the WASM boundary doesn't need to change, only the internal pipeline adds chunking. |

### Scaling Priorities

1. **First bottleneck: Main thread blocking during generation.** For puzzles >500 pieces, generation may take >100ms, causing jank. **Fix:** Move WASM execution to a Web Worker. The worker holds the `PuzzleGenerator`, receives config messages, and posts SVG strings back. This is a straightforward refactor that doesn't change the Rust code at all.

2. **Second bottleneck: SVG rendering in browser.** Very complex SVGs (thousands of paths) can be slow to render in the DOM. **Fix:** For preview, reduce bezier segment count or use a canvas-based renderer. Keep full fidelity for export only.

## Anti-Patterns

### Anti-Pattern 1: Passing Geometry Data Across WASM Boundary

**What people do:** Return individual piece paths, vertex arrays, or geometry buffers to JavaScript for assembly/rendering.
**Why it's wrong:** Every cross-boundary call has overhead. Serializing thousands of points is far slower than assembling the SVG string in Rust and returning it as a single string.
**Do this instead:** Do ALL geometry computation and SVG assembly in Rust. Return only the final SVG string. JavaScript's job is rendering and user interaction, not geometry.

### Anti-Pattern 2: Using JS Math.random() from WASM

**What people do:** Import `Math.random` via `js-sys` for randomization in the Rust code.
**Why it's wrong:** `Math.random` is not seedable, making puzzles non-reproducible. It also requires crossing the WASM boundary for every random number, which is slow.
**Do this instead:** Use `rand` crate with `rand_chacha::ChaCha8Rng` (or similar seedable RNG) in pure Rust. Initialize from a user-provided seed. No boundary crossings, fully deterministic.

### Anti-Pattern 3: Monolithic generate() Function

**What people do:** Put all generation logic in a single massive function.
**Why it's wrong:** Untestable, unextensible, unmaintainable. Can't unit test edge assignment without also testing connector generation.
**Do this instead:** Pipeline of distinct, independently testable modules: `GridLayout -> EdgeAssignment -> ConnectorGenerator -> SVGAssembler`. Each module has its own tests with known inputs/outputs.

### Anti-Pattern 4: Floating Point Equality for Geometric Validation

**What people do:** Use `==` to check if path endpoints meet, connectors align, etc.
**Why it's wrong:** Floating point arithmetic introduces tiny errors. Two points that should be identical may differ by 1e-15.
**Do this instead:** Use epsilon-based comparison (`(a - b).abs() < EPSILON`) for all geometric equality checks. Define a project-wide `EPSILON` constant (e.g., `1e-9`).

### Anti-Pattern 5: SVG Namespace and Attribute Omission

**What people do:** Generate SVG paths without proper `xmlns`, `viewBox`, measurement units, or stroke attributes.
**Why it's wrong:** Laser cutter software (LightBurn, Glowforge) may fail to import or incorrectly scale SVGs missing these attributes.
**Do this instead:** Always emit complete SVG documents with: `xmlns="http://www.w3.org/2000/svg"`, correct `viewBox`, explicit `width`/`height` with units (mm or in), `fill="none"` and `stroke="black"` on cut paths, and appropriate `stroke-width`.

## Integration Points

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| JS GUI <-> WASM Module | `wasm-bindgen` + `serde-wasm-bindgen` for config; `String` return for SVG | Config is the only data going into WASM. SVG string is the only data coming out. Clean, minimal boundary. |
| GridLayout -> EdgeAssignment | Direct Rust function calls; `Grid<CellBounds>` struct passed by reference | No serialization. Pure internal Rust module boundary. |
| EdgeAssignment -> ConnectorGenerator | `EdgeMap` with edge endpoints, direction, type | Internal data structure. Each edge carries its two endpoint coordinates and assigned direction. |
| ConnectorGenerator -> SVGAssembler | `Vec<PathCommand>` per edge | Abstract path commands (MoveTo, CubicBezierTo, LineTo). No SVG-specific strings at this level. |

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| wasm-pack | Build tool, compiles Rust to WASM + JS bindings | Run as build step. Outputs to `pkg/` directory consumed by web bundler. |
| Vite/bundler | Imports WASM package, serves web app | Must be configured for WASM support. Vite has built-in WASM support via `vite-plugin-wasm` or top-level await. |
| Laser cutter software (Glowforge, LightBurn) | Consumes exported SVG files | SVG must follow specific conventions: vector paths only, no raster, proper units, single color for single-operation cutting. |

## Build Order Implications

The architecture has clear dependency ordering that should guide phase structure:

1. **Geometry primitives first** (`geometry/`): `Point2D`, `CubicBezier`, `PathCommand`. These are the foundation everything else builds on. Zero external dependencies. Fully testable in isolation with native Rust tests.

2. **Grid layout second** (`grid.rs`): Depends only on geometry primitives. Can be tested by verifying cell boundary coordinates for known inputs.

3. **Edge assignment third** (`edge.rs`): Depends on grid layout output. Can be tested by verifying correct tab/blank/flat assignment patterns and mating constraints.

4. **Connector generation fourth** (`connector/`): Depends on geometry primitives + edge info. The most mathematically complex component. Needs visual verification (generate SVG of individual connectors for manual inspection).

5. **SVG assembly fifth** (`svg/`): Depends on all of the above. Produces the output format.

6. **WASM bridge sixth** (`lib.rs`): Wires everything together with `wasm-bindgen`. Depends on all Rust modules being functional.

7. **Web GUI last** (`web/`): Depends on working WASM module. Can use mock/hardcoded SVG for early UI development in parallel with Rust work.

**Critical path:** Items 1-6 are sequential. Item 7 (Web GUI) can be developed in parallel from step 4 onward using hardcoded test SVGs.

## Sources

- Rust and WebAssembly Book - Implementing Conway's Game of Life (architecture patterns for Rust-WASM apps): https://rustwasm.github.io/docs/book/game-of-life/implementing.html [HIGH confidence - official documentation, though now archived]
- Rust and WebAssembly Book - Crates You Should Know: https://rustwasm.github.io/docs/book/reference/crates.html [HIGH confidence]
- Rust and WebAssembly Book - JavaScript Interoperation: https://rustwasm.github.io/docs/book/reference/js-ffi.html [HIGH confidence]
- Rust and WebAssembly Book - Shrinking .wasm Size: https://rustwasm.github.io/docs/book/reference/code-size.html [HIGH confidence]
- wasm-bindgen docs (v0.2.114): https://docs.rs/wasm-bindgen/latest/wasm_bindgen/ [HIGH confidence - official crate docs]
- serde-wasm-bindgen docs (v0.6.5): https://docs.rs/serde-wasm-bindgen/latest/serde_wasm_bindgen/ [HIGH confidence - official crate docs, recommended by wasm-bindgen]
- svg crate docs (v0.18.0): https://docs.rs/svg/latest/svg/ [MEDIUM confidence - viable but may be overkill; manual SVG string building may be simpler for this use case]
- rand crate docs (v0.10.0): https://docs.rs/rand/latest/rand/ [HIGH confidence - official crate docs]
- Bezier curve mathematics: https://en.wikipedia.org/wiki/B%C3%A9zier_curve [HIGH confidence - well-established mathematics]
- Connector shape generation approach: Cubic bezier curves with 4 control points per connector segment, randomized control point offsets for variation [MEDIUM confidence - based on standard computational geometry practice and domain analysis; specific implementation needs validation during development]

---
*Architecture research for: Procedural Jigsaw Puzzle Pattern Generator*
*Researched: 2026-03-01*

# Stack Research

**Domain:** Procedural jigsaw puzzle pattern generator (Rust + WASM + web GUI)
**Researched:** 2026-03-01
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (stable) | 1.93+ | Core puzzle generation engine | Already decided per project spec. Memory safety, zero-cost abstractions, and deterministic performance critical for real-time path computation. WASM compilation is a first-class target. |
| wasm-bindgen | 0.2.114 | Rust-to-JS FFI bindings | The standard for Rust/WASM interop. Generates TypeScript type declarations automatically. Updated Feb 2026 — very actively maintained. No real alternative exists. |
| wasm-pack | 0.14.0 | Build tooling for Rust WASM | Wraps `cargo build --target wasm32-unknown-unknown` with npm package generation, type generation, and optimization. Updated Jan 2026. The canonical build tool. |
| web-sys | 0.3.91 | DOM/Web API bindings from Rust | Provides typed access to browser APIs (Canvas, SVG DOM, etc.) from Rust. Versioned in lockstep with wasm-bindgen. Use only if Rust code needs direct DOM access (prefer JS side for DOM). |
| js-sys | 0.3.91 | JavaScript built-in bindings | Typed access to JS built-ins (Array, Date, JSON, etc.) from Rust. Same version cadence as wasm-bindgen. Needed for passing complex data across the WASM boundary. |
| TypeScript | 5.9.x | Frontend type safety | Type safety for the web GUI. wasm-bindgen auto-generates `.d.ts` files for the WASM module, giving end-to-end type safety from Rust through to the UI layer. |
| Vite | 7.3.x | Frontend build tooling and dev server | Fast HMR, native ESM, built-in WASM support. The standard frontend build tool in 2026. Better WASM integration than webpack/parcel. |

### Rust Geometry & Math Libraries

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| kurbo | 0.13.0 | 2D curve math (Bezier, paths, shapes) | **PRIMARY choice for this project.** From the Linebender project (Raph Levien). Provides `CubicBez`, `BezPath`, `Point`, `Affine` transforms, path simplification, offset curves, and SVG arc support. Purpose-built for 2D vector graphics. 15.6M downloads. Supports `serde` feature for serialization. Updated Nov 2025. |
| lyon | 1.0.16 | Path tessellation and algorithms | **Complementary to kurbo.** Provides path boolean operations, hit testing, and tessellation (if we ever need GPU preview). `lyon_algorithms` has path walking, simplification, and winding number utilities. However, **for SVG path output, kurbo is sufficient alone** — only bring in lyon if boolean operations on paths are needed (e.g., whimsy piece clipping). Updated Sep 2025. |

### Randomization & Procedural Generation

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| rand | 0.10.0 | Random number generation | The standard Rust RNG crate. v0.10 released Feb 2026 — breaking changes from 0.9 (new `rand_core` traits). Provides `SeedableRng` trait critical for reproducible puzzle generation from seeds. |
| rand_chacha | 0.10.0 | Deterministic seeded RNG | ChaCha20 algorithm. **Use this as the RNG engine** because it is deterministic across platforms — same seed produces identical puzzle on any machine. Critical for the "share seed" feature. Released Feb 2026. |
| getrandom | 0.4.1 | OS/WASM entropy source | Required for initial seed generation in browser (uses `crypto.getRandomValues`). Must configure with `wasm_js` feature for WASM target. Released Feb 2026. |
| noise | 0.9.0 | Perlin/Simplex noise | **Optional, defer to later phase.** Could add natural variation to connector shapes or grid distortion. Not needed for MVP — simple randomized Bezier control points are sufficient. Last updated Mar 2024, still functional. |

### SVG Output

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| svg (crate) | 0.18.0 | SVG document construction | Builder-pattern API for constructing SVG DOM trees in Rust. 4.8M downloads. Use this for the final SVG export — construct `<path>` elements with `d` attributes from kurbo `BezPath` data. Last updated Sep 2024; stable, works fine. |
| *(manual `String` formatting)* | N/A | SVG path `d` attribute generation | **Alternative to `svg` crate.** SVG path data (`M`, `C`, `L`, `Z` commands) is trivially simple to emit as strings. For maximum control and minimal WASM size, generate path `d` strings directly from kurbo geometry. Consider this if the `svg` crate adds unwanted WASM bloat. |

### Serialization & Data

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| serde | 1.0.228 | Serialization framework | Standard Rust serialization. Needed for passing puzzle configuration between JS and Rust (via JSON), and for seed/config export. Sep 2025. |
| serde_json | 1.0.149 | JSON serialization | JSON is the natural format for JS-WASM boundary data. Configuration structs serialize to JSON, pass to JS, render in UI. Jan 2026. |
| serde-wasm-bindgen | *(latest)* | Direct JS value serialization | **Use instead of `serde_json` for WASM boundary.** Converts Rust structs directly to/from `JsValue` without intermediate JSON string — faster and more ergonomic. |

### WASM Infrastructure

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| console_error_panic_hook | 0.1.7 | Debug panic messages in browser | Redirects Rust panics to `console.error` with stack traces. Essential for development. Tiny crate, no real alternatives. Oct 2021 but fully stable. |
| wasm-bindgen-test | 0.3.x | WASM unit testing | Run Rust tests in headless browser. Versioned with wasm-bindgen. Use for testing geometry correctness in actual WASM environment. |

### Frontend (Web GUI)

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| Vite | 7.3.x | Dev server + bundler | Native ESM, instant HMR, WASM support via plugin. |
| vite-plugin-wasm | 3.5.0 | WASM ESM integration for Vite | Adds WebAssembly ESM integration to Vite. Supports wasm-pack generated modules. Works with Vite 2-7. |
| Vanilla TypeScript (no framework) | — | UI layer | **Deliberately no React/Vue/Svelte.** The UI is a simple control panel + SVG preview. A framework adds bundle size and complexity for no benefit. Use vanilla TS with direct DOM manipulation. The SVG preview is rendered by injecting SVG markup generated by the Rust/WASM core. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| wasm-pack | Build Rust to WASM npm package | `wasm-pack build --target web` for ESM output compatible with Vite |
| wasm-opt (via wasm-pack) | WASM binary optimization | Automatically invoked by wasm-pack in release mode. Reduces binary size 10-30%. |
| cargo-watch | Auto-rebuild on Rust changes | `cargo watch -s 'wasm-pack build'` for rapid iteration |
| wasm-bindgen-cli | TypeScript declaration generation | Installed automatically by wasm-pack; generates `.d.ts` files |

## Installation

### Rust Dependencies (Cargo.toml)

```toml
[package]
name = "puzzle-generator"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
kurbo = { version = "0.13", features = ["serde"] }
rand = { version = "0.10", features = ["small_rng"] }
rand_chacha = "0.10"
getrandom = { version = "0.4", features = ["wasm_js"] }
svg = "0.18"
console_error_panic_hook = "0.1"

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "s"       # Optimize for size (WASM delivery)
lto = true            # Link-time optimization
```

### Frontend Dependencies (package.json)

```bash
# Core
npm install vite-plugin-wasm

# Dev dependencies
npm install -D vite typescript
```

### Build Commands

```bash
# Build WASM module
wasm-pack build --target web --out-dir web/pkg

# Dev server (from web/ directory)
npx vite

# Production build
wasm-pack build --target web --release --out-dir web/pkg
npx vite build
```

## Alternatives Considered

| Category | Recommended | Alternative | Why Not Alternative |
|----------|-------------|-------------|---------------------|
| 2D Geometry | kurbo | lyon_geom | lyon_geom is tied to the lyon ecosystem (f32 only, GPU-focused). kurbo is standalone, f64 precision (better for laser cutting), and has superior Bezier math (offset curves, simplification). |
| 2D Geometry | kurbo | geo | geo is for GIS/geospatial (lat/lng, projections, spatial indexing). Way too heavy for 2D curve math. Different problem domain. |
| SVG Output | svg crate / manual String | resvg/usvg | resvg is an SVG *renderer*, not a generator. We need to *produce* SVG, not render it. |
| SVG Output | svg crate / manual String | svg-fmt | svg-fmt is for *debugging* SVG output — dumps shapes for visualization. Not a production SVG builder. |
| Seeded RNG | rand_chacha (ChaCha20) | rand_pcg / rand_xoshiro | ChaCha20 is cryptographically secure and **guaranteed cross-platform deterministic**. PCG and xoshiro are faster but the performance difference is negligible for our puzzle-sized workloads, and ChaCha20's determinism guarantee is stronger. |
| Frontend Framework | Vanilla TypeScript | React / Vue / Svelte | This is a single-page tool with ~10 controls and an SVG preview pane. A framework would add 30-100KB gzipped to the bundle for no benefit. Vanilla TS with a few event listeners is simpler, faster, and has zero framework upgrade churn. |
| Frontend Framework | Vanilla TypeScript | Leptos / Yew / Dioxus (Rust WASM frameworks) | These compile Rust to WASM for the entire UI, which is impressive but wrong for this project. They produce much larger WASM bundles, have slower iteration cycles (full recompile for UI changes), and make it harder to use browser-native SVG rendering. The right boundary is: Rust/WASM for computation, TypeScript for UI. |
| WASM Build | wasm-pack | trunk | Trunk is designed for full Rust WASM apps (Yew/Leptos). Since we're using vanilla TS for the frontend and only need wasm-pack to compile the Rust library, wasm-pack is the right tool. |
| Frontend Bundler | Vite | webpack / parcel | Vite is faster, simpler, and has better native ESM/WASM support. webpack requires more configuration. Parcel has less ecosystem support. |
| Serialization boundary | serde-wasm-bindgen | serde_json (string) | serde-wasm-bindgen converts directly to/from JsValue without JSON string intermediary — faster and more idiomatic. serde_json requires stringify/parse round-trip across the boundary. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Leptos / Yew / Dioxus for UI | Massive WASM bundles (500KB+), slow recompile for UI tweaks, poor SVG DOM integration | Vanilla TypeScript for UI, Rust WASM only for computation |
| `geo` crate | GIS-focused, wrong abstraction level, pulls in heavy dependencies | `kurbo` for 2D curve math |
| `resvg` / `usvg` | SVG renderers, not generators. We produce SVG, not consume it. | `svg` crate or manual path string generation |
| `f32` for geometry | Laser cutters work in physical units (mm/inches). f32 gives ~7 decimal digits, which causes visible path artifacts at large puzzle sizes. | `f64` (kurbo uses f64 natively) |
| `noise` crate in MVP | Adds complexity and WASM size for minimal visual benefit in v1. Simple Bezier randomization is sufficient. | Direct randomized Bezier control point offsets via `rand` |
| `wasm-bindgen` `--target bundler` | Requires webpack-style bundler integration, more complex setup | `--target web` for native ESM, works directly with Vite |
| Canvas 2D for preview | Requires rasterization, loses SVG fidelity, can't inspect paths | Inject SVG markup directly into DOM — the preview IS the output |
| npm `svg.js` or `d3` for SVG | These are for manipulating SVG in the browser. Our SVG is generated by Rust. The browser just displays it via innerHTML. | Direct SVG markup injection from WASM output |

## Stack Patterns

**For live preview (primary pattern):**
- Rust generates SVG path data (string of `M`/`C`/`L`/`Z` commands)
- Passes string to JS via wasm-bindgen
- JS sets `innerHTML` on a container `<div>` to display the SVG
- This is simple, fast, and the preview exactly matches the export

**For configuration UI:**
- TypeScript reads control values (sliders, dropdowns)
- Serializes config to a Rust struct via serde-wasm-bindgen
- Calls WASM function with config, receives SVG string back
- Debounce rapid changes (e.g., slider drag) to avoid overwhelming WASM calls

**For SVG export:**
- Same Rust code that generates preview SVG
- JS triggers file download via `Blob` + `URL.createObjectURL`
- No separate export path — preview IS the export (single source of truth)

**For seed-based reproducibility:**
- User provides or generates a u64 seed
- Seed initializes `ChaCha20Rng` via `SeedableRng::seed_from_u64(seed)`
- All randomized operations draw from this single RNG
- Same seed + same config = identical puzzle, always, on any platform

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| wasm-bindgen 0.2.114 | web-sys 0.3.91, js-sys 0.3.91 | Must use matching versions — they're released in lockstep |
| rand 0.10.0 | rand_chacha 0.10.0, getrandom 0.4.x | rand 0.10 requires rand_chacha 0.10 (breaking change from 0.9) |
| kurbo 0.13.0 | serde 1.0.x (with `serde` feature) | Optional serde support, enable for config serialization |
| Vite 7.3.x | vite-plugin-wasm 3.5.0, TypeScript 5.9.x | vite-plugin-wasm supports Vite 2-7 |
| Rust edition 2024 | rustc 1.85+ | Rust 2024 edition stabilized in 1.85 (Nov 2024) |
| wasm-pack 0.14.0 | wasm-bindgen 0.2.x | wasm-pack invokes wasm-bindgen-cli; version must be compatible |
| getrandom 0.4.x | wasm32-unknown-unknown target | **Must enable `wasm_js` feature** for browser entropy via `crypto.getRandomValues` |

## Critical WASM-Specific Notes

1. **`getrandom` WASM configuration is mandatory.** Without `features = ["wasm_js"]`, `getrandom` (used by `rand`) will fail to compile for `wasm32-unknown-unknown`. This is the #1 stumbling block for Rust WASM projects using randomness.

2. **`crate-type = ["cdylib", "rlib"]`** is required in `Cargo.toml`. `cdylib` produces the `.wasm` file; `rlib` allows `cargo test` to work natively.

3. **WASM binary size matters.** Use `opt-level = "s"` and `lto = true` in release profile. Consider `wasm-opt -Oz` for additional 10-20% reduction. Target under 200KB gzipped for good load times.

4. **`--target web`** (not `--target bundler`) produces ESM-compatible output that works with Vite's native ESM support without additional configuration.

## Sources

- crates.io API — verified all crate versions and last-updated dates (fetched 2026-03-01) [HIGH confidence]
- docs.rs/kurbo/0.13.0 — verified API surface: BezPath, CubicBez, Point, Affine, SVG path support, offset curves [HIGH confidence]
- docs.rs/lyon/1.0.16 — verified API: tessellation, path algorithms, geom primitives [HIGH confidence]
- npmjs.org — verified Vite 7.3.1, TypeScript 5.9.3, vite-plugin-wasm 3.5.0 versions [HIGH confidence]
- wasm-bindgen docs — verified `--target web` ESM output, TypeScript generation [HIGH confidence]
- Rust 1.93.1 verified installed on build system [HIGH confidence]
- noise 0.9.0 — last updated Mar 2024, functional but not actively developed [MEDIUM confidence]
- serde-wasm-bindgen — recommended by wasm-bindgen docs for direct JsValue conversion [MEDIUM confidence — version not independently verified]

---
*Stack research for: Procedural Jigsaw Puzzle Pattern Generator*
*Researched: 2026-03-01*

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

# Pitfalls Research

**Domain:** Procedural jigsaw puzzle pattern generation (Rust/WASM + SVG for laser cutting)
**Researched:** 2026-03-01
**Confidence:** HIGH (computational geometry and SVG are well-understood domains; Rust-WASM patterns established)

## Critical Pitfalls

### Pitfall 1: Connector Geometry That Doesn't Physically Interlock

**What goes wrong:**
Procedurally generated connector shapes (knobs/tabs and blanks/sockets) that look correct on screen but fail physically. The male connector is wider than the female socket, curves self-intersect, or the neck of the knob is too narrow relative to the head — causing the piece to break during cutting or be impossible to assemble. This is the single most domain-specific failure mode.

**Why it happens:**
Developers treat connector generation as purely visual curve-drawing. They parameterize Bézier curves or arc segments without enforcing geometric constraints: minimum neck width, maximum knob diameter relative to socket, clearance tolerances for the laser kerf, and path non-self-intersection. Random variation pushes parameters outside physically valid bounds.

**How to avoid:**
- Define a **connector constraint system** before writing any curve generation code. Constraints include: minimum neck width (≥ 2× material thickness), knob head diameter < socket opening + kerf, no self-intersecting paths, smooth C1 continuity at connection points.
- Model connectors as parameterized templates with **clamped randomization ranges** — randomize within known-good bounds, not arbitrary ranges.
- Every connector is a **mating pair**: generate the male and female side from the same parameters, offset by kerf. Never generate them independently.
- Write property-based tests that verify: no path self-intersection, male fits within female with kerf tolerance, minimum feature width exceeds material/laser constraints.

**Warning signs:**
- Connector parameters lack explicit min/max bounds
- Male and female connectors generated by separate code paths
- No concept of "kerf" or "tolerance" in the data model
- Visual testing only (looks right on screen, never validated geometrically)

**Phase to address:**
Phase 1 (Core engine). This must be right from the start — every other feature depends on valid connectors.

---

### Pitfall 2: Floating-Point Precision Causing Gaps and Overlaps in Cut Paths

**What goes wrong:**
Adjacent puzzle piece edges don't share exactly the same path coordinates due to floating-point arithmetic. One piece's edge is at `y=100.00000000001` while the neighbor is at `y=99.99999999999`. The result is either a visible gap (pieces don't touch) or a double-cut line (laser cuts the same edge twice, ruining the material). At laser-cutter scale, even sub-micron differences cause perceptible artifacts.

**Why it happens:**
Floating-point arithmetic is inherently imprecise. When computing Bézier curves, arc segments, or grid intersections, accumulated rounding errors cause shared edges to diverge. The problem is amplified when connector curves are computed independently for adjacent pieces rather than being shared.

**How to avoid:**
- **Shared edge architecture**: Each internal edge exists once in memory. Both adjacent pieces reference the same edge data (one as-is, one reversed). Never recompute the same edge for two pieces.
- Use a consistent coordinate precision strategy: round final SVG output coordinates to a fixed decimal precision (4-5 decimal places is plenty for laser cutters — ~0.1 micron at typical scales).
- Build edge identity from grid coordinates `(row, col, direction)` so the same edge is always looked up, never duplicated.
- Avoid computing the same intersection point twice from different directions.

**Warning signs:**
- Each piece independently computes all four of its edges
- No shared data structure for edges between adjacent pieces
- SVG output uses full f64 precision (15+ decimal places)
- Visual gaps appear at high zoom levels in preview

**Phase to address:**
Phase 1 (Core engine / data model). The grid and edge data model must be designed with sharing from day one. Retrofitting shared edges is a rewrite.

---

### Pitfall 3: SVG Output Incompatible with Laser Cutter Software

**What goes wrong:**
Generated SVG files open correctly in web browsers but fail in laser cutter software (LightBurn, Glowforge UI, LaserGRBL). Common failures: paths interpreted as fills instead of cuts, units misinterpreted (px vs mm vs inches), grouped elements can't be selected individually, embedded CSS styles ignored, viewBox scaling causes wrong physical dimensions.

**Why it happens:**
SVG is a complex spec. Browsers render SVG very permissively, while laser cutter software has strict, inconsistent subset support. Developers test in browsers, not in actual laser software. The SVG spec allows multiple ways to express the same geometry, but laser tools only understand a subset.

**How to avoid:**
- Output only `<path>` elements with explicit `d` attributes using absolute coordinates (M, L, C, A, Z commands). No `<circle>`, `<rect>`, `<polyline>`, or other shape shortcuts — some laser tools can't handle them.
- Set explicit `width` and `height` attributes in physical units (`mm` or `in`) on the root `<svg>` element. Never rely on `viewBox` alone for sizing.
- Use inline style attributes (`stroke="red" fill="none" stroke-width="0.1"`) not CSS classes or `<style>` blocks.
- No transforms on path elements — pre-apply all transforms to coordinates.
- No `<g>` nesting deeper than one level.
- Set `fill="none"` and `stroke="black"` (or `stroke="red"` for Glowforge cut lines) explicitly on every path.
- Test SVG output in LightBurn (free trial available) and the Glowforge web UI early. Don't wait until the end.

**Warning signs:**
- SVG uses CSS stylesheets for stroke/fill
- Paths use relative coordinates (lowercase commands: m, l, c)
- No physical unit on `<svg>` width/height
- Transform attributes on `<path>` or `<g>` elements
- Testing only in Chrome/Firefox, never in laser software

**Phase to address:**
Phase 1 (SVG export). Define the SVG output format as a strict subset from the beginning. Create a validation function that rejects non-conforming output.

---

### Pitfall 4: WASM Bundle Size Explosion from Rust Dependencies

**What goes wrong:**
The compiled `.wasm` binary balloons to 2-10+ MB because Rust pulls in the standard library's formatting/panic infrastructure, or because a heavy dependency (like `regex`, `serde` with full features, or an RNG crate that pulls in `getrandom` with system backends) brings in unexpected code. Users experience slow page loads, especially on mobile.

**Why it happens:**
Rust's monomorphization and generous standard library mean that even simple code can produce large binaries. The `format!` macro alone pulls in significant formatting infrastructure. Panics (including implicit ones from `unwrap()`, array indexing, and division) add error formatting code. Crates designed for native Rust may include system-specific code that bloats or breaks WASM builds.

**How to avoid:**
- Configure `Cargo.toml` from day one with WASM-optimized release profile:
  ```toml
  [profile.release]
  opt-level = 's'
  lto = true
  codegen-units = 1
  strip = true
  ```
- Use `wasm-opt -Os` (from Binaryen) as a post-processing step.
- Use `wasm-pack` or `wasm-bindgen` with `--target web` for the leanest output.
- Prefer `Result` returns over `unwrap()`/`expect()` to avoid panic infrastructure.
- Audit every dependency for WASM compatibility before adding. Check that crates are `#![no_std]`-compatible or have a `wasm` feature flag.
- Use `twiggy` to profile binary size and identify bloat sources.
- For RNG, use `rand` with `SmallRng` and the `wasm-bindgen` feature — avoid `getrandom` defaults that try to access OS entropy sources.
- Budget: target < 500KB gzipped for the WASM module.

**Warning signs:**
- `.wasm` binary exceeds 1MB uncompressed without complex logic
- Build warnings about missing `getrandom` backends
- `twiggy top` shows `dlmalloc`, panic formatting, or `core::fmt` dominating binary size
- Adding a new crate increases bundle size by 100KB+

**Phase to address:**
Phase 1 (Project setup / build pipeline). Set up WASM-optimized build configuration and size monitoring from the very first build. Size debt compounds quickly.

---

### Pitfall 5: Grid-to-Connector Integration Breaks at Boundary Conditions

**What goes wrong:**
Edge pieces (flat border sides), corner pieces (two flat sides), and the transition between border edges and interior connectors produce invalid geometry. Connectors that work perfectly in the grid interior fail at borders — connector curves extend past the puzzle boundary, border edges don't close properly, or corner pieces have malformed paths.

**Why it happens:**
Developers build and test with interior pieces first (the common case), then bolt on border/corner handling as an afterthought. Border pieces need fundamentally different path construction: one or more sides are straight lines instead of connector curves, and the straight-to-curve transitions must be smooth. Corner pieces combine two straight sides with two connector sides. The combinatorial complexity of `{top, right, bottom, left} × {flat, connector_in, connector_out}` is easy to get wrong.

**How to avoid:**
- Model piece edge types as an enum: `Flat | ConnectorMale | ConnectorFemale`. Every piece has exactly 4 edges, each with a type. Border determination is a function of grid position.
- Build border and corner pieces FIRST, or simultaneously with interior pieces — not after. They're the constrained case.
- The piece path assembly function must handle all 16 possible edge-type combinations (4 sides × {flat, male, female} minus impossible combos). Write tests for every valid combination.
- Test with small grids (2×2, 3×3) where every piece is a border or corner piece. These expose boundary bugs immediately.

**Warning signs:**
- All early testing uses 10×10 or larger grids (border pieces are a tiny fraction)
- Border handling is a special-case `if` statement bolted onto interior logic
- No test for 2×2 grid (all corners, no interior pieces)
- No test for 1×N or N×1 grids

**Phase to address:**
Phase 1 (Core engine). Test with minimum-size grids from the first implementation.

---

### Pitfall 6: Seed-Based Reproducibility Broken by Algorithm Changes

**What goes wrong:**
Users save a seed value expecting to regenerate the exact same puzzle later. But any change to the generation algorithm — reordering operations, adding new randomized parameters, updating the RNG crate — silently changes the output for existing seeds. Saved/shared seeds become useless after updates.

**Why it happens:**
Pseudo-random number generators produce deterministic sequences from seeds, but the sequence depends on the exact order and number of calls to the RNG. Adding a single new `rng.next()` call anywhere in the generation pipeline shifts every subsequent random value. Developers don't realize that "deterministic" means "sequence-sensitive."

**How to avoid:**
- **Version the generation algorithm**. Store `(seed, algorithm_version)` as the reproducibility key. When the algorithm changes, increment the version. Support regenerating with older algorithm versions.
- Use **independent RNG instances** for independent concerns: one for connector shape randomization, one for piece wobble, one for whimsy placement. Derive child RNGs from the master seed using distinct stream IDs. This way, adding randomization to connectors doesn't affect whimsy placement.
- Document and test seed stability: maintain a set of reference seeds with known outputs (hash of SVG output). CI tests verify these don't change unexpectedly.
- Use a well-specified RNG algorithm (e.g., `ChaCha8Rng` from the `rand_chacha` crate) rather than `StdRng` which may change between Rust versions.

**Warning signs:**
- Single RNG instance passed through entire generation pipeline
- No tests that verify specific seed → specific output
- Using `rand::thread_rng()` or `StdRng` (implementation may change)
- Algorithm changes don't update any version marker

**Phase to address:**
Phase 1 (Core engine). The RNG architecture (sub-streams for independent concerns) must be designed into the generation pipeline from the start.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Single monolithic path per piece | Simpler generation code | Can't individually select/manipulate edges; can't highlight shared edges in preview; can't implement partial puzzle display | Never — use edge-based assembly from the start |
| Hardcoded connector shape | Get to MVP faster | Can't add new connector types without major refactoring; users stuck with one look | Only for initial prototype; refactor before Phase 2 |
| String concatenation for SVG | No SVG library dependency | Malformed XML, escaping bugs, no validation, hard to add attributes | MVP only — switch to structured SVG builder before adding features |
| `unwrap()` everywhere | Faster development | WASM panics are opaque; binary size bloat from panic infrastructure; bad user experience | Never in code that reaches WASM build — use `Result` |
| Pixel coordinates instead of physical units | Simpler math | Users can't specify puzzle size in inches/cm; export dimensions wrong for laser cutters | Never — use physical coordinates from day one |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| WASM ↔ JS data passing | Serializing complex structs to JSON strings on every call (slow) | Pass flat numeric arrays via shared WASM memory for geometry data; serialize only configuration |
| WASM ↔ JS string handling | Returning SVG as a string through wasm-bindgen on every parameter change | Generate SVG only on explicit export; use canvas/WebGL for live preview; or pass path data as typed arrays |
| SVG → Laser cutter | Assuming laser software handles SVG identically to browsers | Test with actual laser software early; different tools have different SVG subset support |
| Browser SVG rendering | Using `<img>` tag to render SVG (no interactivity) | Use inline SVG or `<object>` tag for interactive preview; or render to `<canvas>` for performance |
| RNG in WASM | Using `getrandom` crate default features (tries to access OS entropy) | Use `getrandom` with `js` feature flag for WASM, or avoid `getrandom` entirely by requiring explicit seeds |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Regenerating full SVG string on every parameter change | Preview freezes for 200ms+ during slider drag on large puzzles | Separate preview rendering (canvas-based, incremental) from SVG export (on-demand) | >100 pieces with real-time sliders |
| Allocating new Vec for every piece's path during generation | GC-like pauses in WASM, memory grows monotonically | Pre-allocate buffers sized to grid dimensions; reuse across regeneration cycles | >500 pieces, repeated regeneration |
| Bézier curve flattening with fixed step count | Either too many points (huge SVG, slow laser) or too few (visible faceting on curves) | Adaptive flattening based on curvature — more points on tight curves, fewer on gentle ones | Always — fixed steps are always wrong for some curves |
| DOM manipulation for SVG preview (one `<path>` per piece) | Browser layout/paint thrashes with 500+ DOM elements | Use `<canvas>` 2D rendering for preview; SVG DOM only for small puzzles or export preview | >200 pieces in live preview |
| Recomputing entire grid when one parameter changes | Whole-grid recomputation takes 100ms+ for large puzzles | Identify which parameters affect which pieces; cache unaffected results | >500 pieces with interactive parameter tuning |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Showing piece count as "rows × columns" only | Users think in total piece count ("I want a 500-piece puzzle"), not grid dimensions | Show both: grid dims AND total piece count; let users input either |
| No indication of physical output size | Users export SVG, send to laser cutter, discover puzzle is 2 inches wide | Always show physical dimensions prominently; warn if puzzle dimensions seem unreasonable for piece count |
| Preview doesn't match export | Preview uses anti-aliased curves, export has slightly different path precision | Use identical path generation for preview and export; just render differently |
| No kerf compensation option | Pieces fit too loosely (or too tightly depending on direction) after cutting | Provide a kerf offset parameter (default ~0.1-0.2mm for typical laser cutters); document its purpose |
| Randomization slider with no visual feedback | User moves "randomization" slider, can't see what changed | Show before/after or highlight regions affected by randomization; use debounced live preview |
| No way to preview individual pieces | User can't verify a specific piece looks correct before cutting entire sheet | Allow click-to-isolate a piece or small region in preview |

## "Looks Done But Isn't" Checklist

- [ ] **Connector generation:** Often missing kerf compensation — verify that male/female sides account for laser beam width
- [ ] **SVG export:** Often missing explicit physical units — verify `<svg>` root has `width="Xmm"` and `height="Ymm"` attributes
- [ ] **Edge sharing:** Often pieces silently duplicate edge paths — verify that the total number of unique path segments equals `(rows × (cols-1)) + ((rows-1) × cols) + 2×(rows+cols)` (internal horizontal + internal vertical + border edges)
- [ ] **Seed reproducibility:** Often works within a session but breaks across versions — verify with pinned reference seed tests in CI
- [ ] **Border pieces:** Often the 1×1, 1×N, and 2×2 cases crash or produce garbage — verify minimum grid sizes produce valid output
- [ ] **SVG path closure:** Often paths are not properly closed (`Z` command) — verify every piece path is a closed loop (starts and ends at same point)
- [ ] **Coordinate system:** Often Y-axis is flipped between screen (Y-down) and some laser software expectations — verify orientation matches target workflow
- [ ] **Large puzzles:** Often works at 10×10 but OOMs or freezes at 50×50 — verify performance at target max size (e.g., 100×100 = 10,000 pieces)

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Connector geometry invalid | MEDIUM | Add constraint validation layer; implement bounds checking on all connector parameters; regenerate with clamped values |
| Floating-point gaps | HIGH | Requires data model refactor to shared-edge architecture; can't patch with rounding alone |
| SVG incompatibility with laser software | LOW | Define strict SVG subset; write output sanitizer/validator; fix output format without changing generation logic |
| WASM bundle too large | MEDIUM | Profile with `twiggy`; eliminate dependencies one by one; switch to `abort` panic strategy; optimize release profile |
| Boundary condition bugs | MEDIUM | Add comprehensive test suite for all grid-size edge cases; systematic review of edge-type combinations |
| Seed reproducibility broken | HIGH | Implement algorithm versioning retroactively; maintain parallel generation paths for old versions; breaking change for existing users |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Connector geometry validity | Phase 1: Core engine | Property-based tests verify no self-intersection, valid mating, minimum feature width |
| Floating-point gaps | Phase 1: Data model | Shared-edge architecture verified by counting unique edges vs expected formula |
| SVG laser compatibility | Phase 1: SVG export | Output passes validation against strict SVG subset; tested in LightBurn import |
| WASM bundle size | Phase 1: Build setup | CI gate on `.wasm` file size (< 500KB gzipped); `twiggy` report in CI |
| Grid boundary conditions | Phase 1: Core engine | Test matrix covering all grid sizes 1×1 through 4×4; all edge-type combinations |
| Seed reproducibility | Phase 1: RNG architecture | Reference seed tests in CI; sub-stream RNG design documented |
| Performance at scale | Phase 2: Optimization | Benchmark suite for 10×10, 50×50, 100×100 grids; preview frame time budget (< 16ms) |
| Preview/export mismatch | Phase 2: GUI | Visual regression tests comparing preview render to exported SVG |
| Whimsy piece integration | Phase 3+: Advanced features | Whimsy pieces must satisfy same connector constraints as standard pieces |
| Multi-connector types | Phase 3+: Advanced features | Each connector type independently validated; interoperability tested |

## Sources

- Rust and WebAssembly Book — WASM code size optimization (HIGH confidence): https://rustwasm.github.io/docs/book/reference/code-size.html
- Rust and WebAssembly Book — JS FFI and data passing patterns (HIGH confidence): https://rustwasm.github.io/docs/book/reference/js-ffi.html
- Rust and WebAssembly Book — Crate compatibility with WASM (HIGH confidence): https://rustwasm.github.io/docs/book/reference/which-crates-work-with-wasm.html
- MDN SVG Path reference — path command syntax (HIGH confidence): https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorials/SVG_from_scratch/Paths
- Wikipedia Jigsaw Puzzle — Physical puzzle construction, piece terminology, whimsy pieces (HIGH confidence): https://en.wikipedia.org/wiki/Jigsaw_puzzle
- Computational geometry floating-point precision issues — domain knowledge (MEDIUM confidence, training data)
- Laser cutter SVG compatibility issues — domain knowledge from maker community patterns (MEDIUM confidence, training data corroborated by multiple community sources)
- Connector geometry constraints for physical puzzles — domain knowledge (MEDIUM confidence, training data)

---
*Pitfalls research for: Procedural jigsaw puzzle pattern generation (Rust/WASM + SVG for laser cutting)*
*Researched: 2026-03-01*