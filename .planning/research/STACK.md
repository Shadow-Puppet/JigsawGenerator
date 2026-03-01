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
