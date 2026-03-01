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
