# Phase 1: Build Pipeline & WASM Foundation - Context

**Gathered:** 2026-03-01
**Status:** Ready for planning

<domain>
## Phase Boundary

A working Rust-to-WASM-to-Vite build pipeline where TypeScript can call Rust functions and receive results in the browser. Delivers requirement INFR-01. This phase creates the foundation all subsequent phases build on — no puzzle logic beyond a proof-of-life computation.

</domain>

<decisions>
## Implementation Decisions

### WASM binding approach
- Use **wasm-pack** for Rust-to-WASM compilation (auto-generates npm package, TypeScript types, handles wasm-opt)
- Data crosses the Rust/TS boundary as **serialized JSON** via serde — simple, debuggable, flexible
- WASM module loads **asynchronously** with a loading state in the UI
- **Thin facade API** — one main entry point (e.g., `generate_puzzle(config_json) -> result_json`), not many granular exports. TypeScript wrapper handles ergonomics

### Project structure & monorepo layout
- **Separate top-level directories**: `/crates/` for Rust, `/web/` for Vite+TS
- **Cargo workspace from day one** with at least two crates:
  - `puzzle-core` — library crate with domain logic (pure Rust, no WASM dependencies)
  - `puzzle-wasm` — thin WASM bindings crate that wraps puzzle-core for the browser
- **Vanilla TypeScript + Vite** for the web side — no framework. Phase 4 builds GUI controls on this foundation
- wasm-pack output goes to **default location** `/crates/puzzle-wasm/pkg/` — web code imports via relative path or Vite alias

### Round-trip proof of life
- Demo does a **puzzle-relevant computation**: TS sends grid dimensions (e.g., 3x4), Rust computes piece count breakdown (total, corners, edges, interior), result displays in browser
- **Minimal but presentable** styling — simple centered layout, basic typography, not ugly but not designed
- **Simple input fields** for rows and columns with a compute button — interactive, not static
- **Basic validation** included — Rust returns errors for invalid inputs (0, negatives), TS displays them in the page. Proves error flow across the boundary

### Dev experience & workflow
- **npm scripts orchestrate both builds** — e.g., `npm run build` calls wasm-pack then Vite. Single entry point from `/web/`
- **Manual rebuild for Rust** during development — Vite dev server runs with HMR for TS changes; Rust changes require manually running wasm-pack, then Vite picks up new WASM
- **Separate build profiles**: `npm run dev:wasm` for fast debug builds, `npm run build` for optimized release. Clear dev/prod distinction
- **Rust unit tests now** (`cargo test` in puzzle-core), web/integration tests deferred to Phase 2

### Claude's Discretion
- Exact Vite plugin configuration for WASM loading
- wasm-opt optimization level for release builds
- Loading indicator design (spinner, text, skeleton)
- Exact npm script naming conventions beyond the ones specified

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. The key constraint is the 500KB gzipped WASM bundle size limit from the success criteria.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-build-pipeline-wasm-foundation*
*Context gathered: 2026-03-01*
