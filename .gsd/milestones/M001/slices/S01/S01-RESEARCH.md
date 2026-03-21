# Phase 1: Build Pipeline & WASM Foundation - Research

**Researched:** 2026-03-02
**Domain:** Rust-to-WASM build pipeline with Vite integration
**Confidence:** HIGH

## Summary

This phase establishes the Rust-to-WASM-to-browser build pipeline that all subsequent phases build on. The stack is well-established: wasm-pack compiles Rust to WASM and generates JS/TS bindings, vite-plugin-wasm provides seamless ESM integration with Vite, and serde_json handles data serialization across the WASM boundary. The project uses a Cargo workspace with two crates (puzzle-core for domain logic, puzzle-wasm for bindings) and a separate /web/ directory for Vite+TypeScript.

The primary risk area is toolchain setup — the development machine uses Arch Linux system Rust (not rustup), so the `rust-wasm` package must be installed for the `wasm32-unknown-unknown` target, and `wasm-pack` must be installed from Arch repos. The proof-of-life demo sends grid dimensions from TypeScript to Rust, computes piece count breakdown, and displays results in the browser — proving the full round-trip works including error handling.

**Primary recommendation:** Use `wasm-pack build --target web` with manual async init for the simplest, most transparent setup. The `--target web` output produces native ESM that Vite can serve directly without requiring vite-plugin-wasm during development. For production builds, vite-plugin-wasm handles bundling the .wasm file properly.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Use **wasm-pack** for Rust-to-WASM compilation (auto-generates npm package, TypeScript types, handles wasm-opt)
- Data crosses the Rust/TS boundary as **serialized JSON** via serde — simple, debuggable, flexible
- WASM module loads **asynchronously** with a loading state in the UI
- **Thin facade API** — one main entry point (e.g., `generate_puzzle(config_json) -> result_json`), not many granular exports. TypeScript wrapper handles ergonomics
- **Separate top-level directories**: `/crates/` for Rust, `/web/` for Vite+TS
- **Cargo workspace from day one** with at least two crates: `puzzle-core` (library, pure Rust) and `puzzle-wasm` (thin WASM bindings)
- **Vanilla TypeScript + Vite** for the web side — no framework
- wasm-pack output goes to **default location** `/crates/puzzle-wasm/pkg/`
- Demo does **puzzle-relevant computation**: TS sends grid dimensions (e.g., 3x4), Rust computes piece count breakdown (total, corners, edges, interior)
- **Minimal but presentable** styling — simple centered layout, basic typography
- **Simple input fields** for rows and columns with a compute button
- **Basic validation** — Rust returns errors for invalid inputs, TS displays them
- **npm scripts orchestrate both builds** — `npm run build` calls wasm-pack then Vite
- **Manual rebuild for Rust** during development — Vite HMR for TS; Rust needs manual wasm-pack rebuild
- **Separate build profiles**: `npm run dev:wasm` for fast debug, `npm run build` for optimized release
- **Rust unit tests now** (`cargo test` in puzzle-core), web/integration tests deferred to Phase 2

### Claude's Discretion
- Exact Vite plugin configuration for WASM loading
- wasm-opt optimization level for release builds
- Loading indicator design (spinner, text, skeleton)
- Exact npm script naming conventions beyond the ones specified

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INFR-01 | Puzzle generation runs in Rust compiled to WASM in the browser | Full stack identified: wasm-pack + wasm-bindgen for compilation, Vite + vite-plugin-wasm for serving/bundling, serde_json for data boundary. Cargo workspace structure with puzzle-core and puzzle-wasm crates. Proof-of-life demo proves the pipeline works end-to-end. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| wasm-pack | 0.14.0 | Rust-to-WASM build tool | Canonical build tool for Rust WASM. Wraps cargo build, generates JS/TS bindings, runs wasm-opt, produces npm-compatible package. Available as Arch package. |
| wasm-bindgen | 0.2.114 | Rust/JS FFI bindings | The standard for Rust-WASM interop. Auto-generates TypeScript declarations. No real alternative. |
| serde | 1.0.x | Serialization framework | Standard Rust serialization. Required for JSON data boundary per user decision. |
| serde_json | 1.0.x | JSON serialization | Handles Rust struct <-> JSON string conversion for the WASM boundary. Per user decision: serialized JSON, not direct JsValue (serde-wasm-bindgen). |
| console_error_panic_hook | 0.1.7 | Browser panic debugging | Redirects Rust panics to console.error with stack traces. Essential for dev. Tiny, stable. |
| Vite | 7.3.x | Frontend build + dev server | Fast HMR, native ESM, WASM support. Standard frontend tooling. |
| TypeScript | 5.9.x | Frontend type safety | wasm-pack generates .d.ts files for end-to-end type safety. |
| vite-plugin-wasm | 3.5.0 | WASM ESM integration for Vite | Enables importing wasm-pack output as ES modules in Vite. Supports Vite 2-7. By Menci, 405 stars, actively maintained. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| vite-plugin-top-level-await | latest | Top-level await support | Only if targeting older browsers (not needed with `build.target: 'esnext'`). vite-plugin-wasm README recommends it for non-modern targets. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `--target web` (wasm-pack) | `--target bundler` | `bundler` output assumes ES module WASM integration (needs vite-plugin-wasm). `web` output works natively but requires manual `init()` call. Both work with Vite — `web` is simpler to understand and debug. |
| serde_json (JSON strings) | serde-wasm-bindgen (direct JsValue) | serde-wasm-bindgen avoids JSON string intermediary and is faster for complex structures. But user decided on JSON for simplicity and debuggability. JSON is fine for the simple config objects this project uses. |
| vite-plugin-wasm | Vite native `?init` import | Vite natively supports `.wasm?init` but requires manual instantiation. vite-plugin-wasm provides cleaner import ergonomics matching wasm-pack output. |

**Installation:**

Arch Linux system packages:
```bash
sudo pacman -S wasm-pack rust-wasm
```

Web dependencies (from /web/ directory):
```bash
npm install -D vite typescript vite-plugin-wasm
```

## Architecture Patterns

### Recommended Project Structure
```
puzzle-generator/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── puzzle-core/              # Pure Rust library (no WASM deps)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs            # Grid math, piece count logic
│   └── puzzle-wasm/              # Thin WASM bindings
│       ├── Cargo.toml
│       ├── src/
│       │   └── lib.rs            # #[wasm_bindgen] exports, JSON facade
│       └── pkg/                  # wasm-pack output (gitignored)
├── web/                          # Vite + TypeScript frontend
│   ├── index.html                # Entry point
│   ├── package.json              # npm scripts orchestrate everything
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── src/
│       ├── main.ts               # WASM init, event handlers, UI logic
│       └── style.css             # Minimal styling
└── .gitignore
```

### Pattern 1: JSON Facade API
**What:** A single `#[wasm_bindgen]` function accepts a JSON string config and returns a JSON string result. TypeScript has a thin wrapper that does JSON.stringify/JSON.parse and provides typed interfaces.
**When to use:** Always for this project (per user decision — thin facade, serialized JSON).
**Example:**

```rust
// crates/puzzle-wasm/src/lib.rs
use wasm_bindgen::prelude::*;
use puzzle_core::compute_piece_breakdown;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn compute_pieces(config_json: &str) -> String {
    match serde_json::from_str::<puzzle_core::GridConfig>(config_json) {
        Ok(config) => {
            match compute_piece_breakdown(&config) {
                Ok(result) => serde_json::to_string(&result).unwrap(),
                Err(e) => serde_json::to_string(&ErrorResponse { error: e.to_string() }).unwrap(),
            }
        }
        Err(e) => serde_json::to_string(&ErrorResponse { error: e.to_string() }).unwrap(),
    }
}
```

```typescript
// web/src/main.ts
import init, { compute_pieces, init_panic_hook } from '../crates/puzzle-wasm/pkg';

interface GridConfig {
  rows: number;
  cols: number;
}

interface PieceBreakdown {
  total: number;
  corners: number;
  edges: number;
  interior: number;
}

interface ErrorResponse {
  error: string;
}

async function loadWasm() {
  await init();
  init_panic_hook();
}

function computePieces(config: GridConfig): PieceBreakdown | ErrorResponse {
  const resultJson = compute_pieces(JSON.stringify(config));
  return JSON.parse(resultJson);
}
```

### Pattern 2: Async WASM Loading with UI State
**What:** WASM module loads asynchronously. UI shows loading state, then enables interaction once WASM is ready.
**When to use:** Always (per user decision — async loading with loading state).
**Example:**

```typescript
// web/src/main.ts
const loadingEl = document.getElementById('loading')!;
const appEl = document.getElementById('app')!;

async function main() {
  try {
    await loadWasm();
    loadingEl.style.display = 'none';
    appEl.style.display = 'block';
    setupEventHandlers();
  } catch (err) {
    loadingEl.textContent = 'Failed to load WASM module';
    console.error(err);
  }
}

main();
```

### Pattern 3: Cargo Workspace with Core + WASM Split
**What:** `puzzle-core` is a pure Rust library with zero WASM dependencies. `puzzle-wasm` is a thin wrapper that depends on puzzle-core and wasm-bindgen. This means puzzle-core can be tested with `cargo test` natively (fast), while puzzle-wasm only handles serialization and FFI.
**When to use:** Always (per user decision — Cargo workspace from day one).
**Example:**

```toml
# Root Cargo.toml
[workspace]
members = ["crates/puzzle-core", "crates/puzzle-wasm"]
resolver = "2"

# crates/puzzle-core/Cargo.toml
[package]
name = "puzzle-core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }

# crates/puzzle-wasm/Cargo.toml
[package]
name = "puzzle-wasm"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
serde_json = "1.0"
console_error_panic_hook = "0.1"
puzzle-core = { path = "../puzzle-core" }

[profile.release]
opt-level = "s"
lto = true
```

### Anti-Patterns to Avoid
- **Putting wasm-bindgen in puzzle-core:** Core library should be pure Rust. WASM bindings belong in puzzle-wasm only. This keeps puzzle-core testable with native `cargo test`.
- **Using `--target bundler` without vite-plugin-wasm:** The `bundler` target produces output expecting WASM ESM integration, which browsers don't natively support. You need either vite-plugin-wasm OR use `--target web` with manual init.
- **Forgetting `crate-type = ["cdylib", "rlib"]`:** `cdylib` produces the .wasm file; `rlib` allows `cargo test` to work. Without both, either build or test will fail.
- **Importing WASM synchronously:** WASM must load asynchronously. Never assume it's available at module load time.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WASM JS bindings | Manual `WebAssembly.instantiate` wrapper | wasm-bindgen + wasm-pack | wasm-pack generates correct JS glue, TypeScript types, handles memory management, string passing, error propagation. Doing this manually is hundreds of lines of error-prone code. |
| JSON serialization across boundary | Manual string building in Rust | serde + serde_json | serde handles all Rust types correctly, generates proper JSON, handles escaping. Manual string building misses edge cases. |
| Build orchestration | Complex shell scripts | npm scripts calling wasm-pack + vite | npm scripts are the standard orchestration point. Keep it simple: one script calls wasm-pack, another calls vite. |
| WASM binary optimization | Manual wasm-opt invocation | wasm-pack release profile | wasm-pack automatically invokes wasm-opt in release mode (configurable via Cargo.toml metadata). |

**Key insight:** The Rust WASM toolchain (wasm-pack + wasm-bindgen) handles enormous complexity: memory layout, string encoding/decoding, error propagation, TypeScript types, binary optimization. Every piece of this is battle-tested. Custom solutions break in subtle ways.

## Common Pitfalls

### Pitfall 1: Missing wasm32-unknown-unknown Target
**What goes wrong:** `wasm-pack build` fails with "target not installed" or compilation errors.
**Why it happens:** On non-rustup setups (like Arch Linux system Rust), the wasm32 target must be installed separately. The development machine has Rust 1.93.1 from the `rust` Arch package but does NOT have `rust-wasm` installed.
**How to avoid:** Install `rust-wasm` from Arch repos: `sudo pacman -S rust-wasm`. This provides the `wasm32-unknown-unknown` target matching the system Rust version.
**Warning signs:** `wasm-pack build` fails immediately with target-related errors.

### Pitfall 2: wasm-pack Not Installed
**What goes wrong:** `wasm-pack` command not found.
**Why it happens:** wasm-pack is not part of the base Rust installation. On Arch, it's a separate package.
**How to avoid:** Install from Arch repos: `sudo pacman -S wasm-pack`. Version 0.14.0 is in the extra repo.
**Warning signs:** First build attempt fails.

### Pitfall 3: Wrong wasm-pack Target Flag
**What goes wrong:** Generated JS doesn't work with Vite, or requires manual init that isn't accounted for.
**Why it happens:** wasm-pack has multiple `--target` options (`web`, `bundler`, `nodejs`, `no-modules`), each producing different JS output:
- `--target web`: Produces ESM with explicit `init()` function. Works directly in browsers. Requires calling `init()` before using exports.
- `--target bundler` (default): Produces output assuming bundler WASM ESM integration. Needs vite-plugin-wasm.
**How to avoid:** Use `--target web` for transparency and simplicity. The generated code is self-contained ESM. TypeScript imports the init function and named exports. This works in dev (Vite serves it as-is) and production (Vite bundles it).
**Warning signs:** Imports fail at runtime, WASM not loaded, "WebAssembly module not yet instantiated" errors.

### Pitfall 4: wasm-pack Output Path with Cargo Workspace
**What goes wrong:** wasm-pack can't find the correct crate, or outputs to unexpected location.
**Why it happens:** In a workspace, you must specify the crate path when running wasm-pack: `wasm-pack build crates/puzzle-wasm`. Without the path, it looks in the current directory.
**How to avoid:** Always specify the crate path: `wasm-pack build crates/puzzle-wasm --target web`. Output goes to `crates/puzzle-wasm/pkg/` by default. Do NOT use `--out-dir` unless necessary.
**Warning signs:** "Could not find `Cargo.toml`" errors, or pkg/ appears in wrong location.

### Pitfall 5: Vite Config Missing WASM Support
**What goes wrong:** Vite dev server fails to serve .wasm files with wrong MIME type, or import fails.
**Why it happens:** Vite's native WASM support uses `?init` query syntax. wasm-pack's generated JS imports .wasm files without query params.
**How to avoid:** Install vite-plugin-wasm and add it to vite.config.ts. Set `build.target: 'esnext'` to enable top-level await (avoiding need for vite-plugin-top-level-await). Configure `optimizeDeps.exclude` for the wasm package if prebundling issues arise.
**Warning signs:** 404 errors for .wasm files, MIME type errors, "No loader configured for .wasm files".

### Pitfall 6: pkg/ Directory in Version Control
**What goes wrong:** Merge conflicts in generated files, stale bindings.
**Why it happens:** wasm-pack generates the pkg/ directory with JS, TS, and WASM files. These are build artifacts.
**How to avoid:** Add `crates/puzzle-wasm/pkg/` to .gitignore. wasm-pack automatically creates a .gitignore inside pkg/ but explicitly ignoring the directory is clearer.
**Warning signs:** Large binary diffs in git, merge conflicts in generated .js/.ts files.

### Pitfall 7: Rust Edition 2024 Syntax Issues
**What goes wrong:** Compilation fails with unexpected syntax errors.
**Why it happens:** Rust edition 2024 (stabilized in 1.85) has some differences from 2021, notably around `unsafe` blocks in unsafe functions, reserved syntax, and gen keyword. Most wasm-bindgen examples use edition 2021.
**How to avoid:** Use edition 2024 as planned (Rust 1.93 supports it), but be aware that some copied examples may need minor adjustments. If issues arise, the differences are minor and well-documented.
**Warning signs:** Unexpected compilation errors when copying code from tutorials.

## Code Examples

### Workspace Root Cargo.toml
```toml
# Cargo.toml (project root)
[workspace]
members = ["crates/puzzle-core", "crates/puzzle-wasm"]
resolver = "2"
```

### puzzle-core Cargo.toml and lib.rs
```toml
# crates/puzzle-core/Cargo.toml
[package]
name = "puzzle-core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
```

```rust
// crates/puzzle-core/src/lib.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GridConfig {
    pub rows: u32,
    pub cols: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PieceBreakdown {
    pub total: u32,
    pub corners: u32,
    pub edges: u32,
    pub interior: u32,
}

pub fn compute_piece_breakdown(config: &GridConfig) -> Result<PieceBreakdown, String> {
    if config.rows == 0 || config.cols == 0 {
        return Err("Rows and columns must be greater than 0".to_string());
    }
    if config.rows == 1 && config.cols == 1 {
        return Ok(PieceBreakdown {
            total: 1,
            corners: 1,
            edges: 0,
            interior: 0,
        });
    }

    let total = config.rows * config.cols;
    let corners = 4.min(total); // 4 corners for any grid >= 2x2
    let edges = if config.rows >= 2 && config.cols >= 2 {
        2 * (config.rows - 2) + 2 * (config.cols - 2)
    } else {
        // 1xN or Nx1 case
        total.saturating_sub(2)
    };
    let interior = total - corners - edges;

    Ok(PieceBreakdown {
        total,
        corners,
        edges,
        interior,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_3x4_grid() {
        let config = GridConfig { rows: 3, cols: 4 };
        let result = compute_piece_breakdown(&config).unwrap();
        assert_eq!(result.total, 12);
        assert_eq!(result.corners, 4);
        assert_eq!(result.edges, 10 - 4); // perimeter minus corners
        assert_eq!(result.interior, 2);
    }

    #[test]
    fn test_invalid_zero() {
        let config = GridConfig { rows: 0, cols: 5 };
        assert!(compute_piece_breakdown(&config).is_err());
    }
}
```

### puzzle-wasm Cargo.toml and lib.rs
```toml
# crates/puzzle-wasm/Cargo.toml
[package]
name = "puzzle-wasm"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
serde_json = "1.0"
console_error_panic_hook = "0.1"
puzzle-core = { path = "../puzzle-core" }

[package.metadata.wasm-pack.profile.release]
wasm-opt = ['-Os']
```

```rust
// crates/puzzle-wasm/src/lib.rs
use wasm_bindgen::prelude::*;
use serde_json;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn compute_pieces(config_json: &str) -> String {
    let config: puzzle_core::GridConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"Invalid config: {}"}}"#, e),
    };

    match puzzle_core::compute_piece_breakdown(&config) {
        Ok(result) => serde_json::to_string(&result).unwrap(),
        Err(e) => format!(r#"{{"error":"{}"}}"#, e),
    }
}
```

### Vite Configuration
```typescript
// web/vite.config.ts
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  plugins: [wasm()],
  build: {
    target: 'esnext',
  },
});
```

### TypeScript WASM Integration
```typescript
// web/src/main.ts
import init, { compute_pieces, init_panic_hook } from '../../crates/puzzle-wasm/pkg';

interface PieceBreakdown {
  total: number;
  corners: number;
  edges: number;
  interior: number;
}

interface ErrorResponse {
  error: string;
}

type ComputeResult = PieceBreakdown | ErrorResponse;

function isError(result: ComputeResult): result is ErrorResponse {
  return 'error' in result;
}

async function main() {
  const loading = document.getElementById('loading')!;
  const app = document.getElementById('app')!;

  try {
    await init();
    init_panic_hook();
    loading.style.display = 'none';
    app.style.display = 'block';

    const rowsInput = document.getElementById('rows') as HTMLInputElement;
    const colsInput = document.getElementById('cols') as HTMLInputElement;
    const computeBtn = document.getElementById('compute')!;
    const resultEl = document.getElementById('result')!;

    computeBtn.addEventListener('click', () => {
      const config = {
        rows: parseInt(rowsInput.value, 10),
        cols: parseInt(colsInput.value, 10),
      };
      const resultJson = compute_pieces(JSON.stringify(config));
      const result: ComputeResult = JSON.parse(resultJson);

      if (isError(result)) {
        resultEl.textContent = `Error: ${result.error}`;
        resultEl.classList.add('error');
      } else {
        resultEl.innerHTML = `
          <strong>Piece Breakdown</strong><br>
          Total: ${result.total}<br>
          Corners: ${result.corners}<br>
          Edges: ${result.edges}<br>
          Interior: ${result.interior}
        `;
        resultEl.classList.remove('error');
      }
    });
  } catch (err) {
    loading.textContent = 'Failed to load WASM module';
    console.error(err);
  }
}

main();
```

### npm Scripts (package.json)
```json
{
  "name": "puzzle-generator-web",
  "private": true,
  "type": "module",
  "scripts": {
    "dev:wasm": "wasm-pack build ../crates/puzzle-wasm --target web --dev",
    "build:wasm": "wasm-pack build ../crates/puzzle-wasm --target web --release",
    "dev": "vite",
    "build": "npm run build:wasm && vite build",
    "preview": "vite preview"
  },
  "devDependencies": {
    "typescript": "^5.9.0",
    "vite": "^7.3.0",
    "vite-plugin-wasm": "^3.5.0"
  }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `wasm-pack --target bundler` + webpack | `--target web` + Vite native ESM | Vite 2+ (2021), mainstream 2023+ | Simpler config, faster dev server, no webpack overhead |
| `wee_alloc` custom allocator for size | Standard allocator + `opt-level = "s"` + `lto = true` | wee_alloc deprecated 2023 | wee_alloc is unmaintained and saved only ~1KB. Standard allocator with LTO is sufficient. |
| `serde_json` strings across boundary | `serde-wasm-bindgen` for direct JsValue | 2022+ | serde-wasm-bindgen is now recommended by wasm-bindgen docs. But user chose JSON strings for simplicity — both work fine. |
| rustwasm.github.io docs | drager.github.io/wasm-pack + wasm-bindgen.github.io | 2025 | Original rustwasm docs are archived. Content mostly still accurate but links may rot. |

**Deprecated/outdated:**
- `wee_alloc`: Was commonly recommended in old tutorials. Unmaintained, negligible size benefit with modern toolchain.
- `#[wasm_bindgen(module = "...")]` for JS imports: Still works but rarely needed for this project type.
- `wasm-pack init`: Deprecated command, replaced by `wasm-pack build`.

## Open Questions

1. **`--target web` vs `--target bundler` with vite-plugin-wasm**
   - What we know: `--target web` produces self-contained ESM with `init()`. `--target bundler` produces ESM assuming WASM module integration. vite-plugin-wasm README says it supports wasm-pack modules.
   - What's unclear: Whether vite-plugin-wasm works better with `bundler` target. The plugin's description explicitly says "support wasm-pack generated modules" suggesting `bundler` target.
   - Recommendation: Start with `--target web` (simplest, most transparent). If import ergonomics are awkward, switch to `--target bundler` + vite-plugin-wasm. Both require vite-plugin-wasm for production builds anyway. LOW effort to switch later.

2. **Relative import path from web/ to crates/puzzle-wasm/pkg/**
   - What we know: TypeScript in web/src/ needs to import from crates/puzzle-wasm/pkg/. This is a `../../crates/puzzle-wasm/pkg` relative path.
   - What's unclear: Whether Vite resolves this cleanly or if a path alias is needed.
   - Recommendation: Try relative import first. If problematic, add a Vite alias in vite.config.ts: `resolve: { alias: { 'puzzle-wasm': path.resolve(__dirname, '../crates/puzzle-wasm/pkg') } }`.

3. **WASM bundle size at this phase**
   - What we know: Success criteria is <500KB gzipped. Phase 1 has minimal Rust code (just serde_json + piece counting). Should be well under 100KB gzipped.
   - What's unclear: Exact size with serde_json included. serde_json can add significant binary size.
   - Recommendation: Measure actual size after first build. If concerning, `opt-level = "s"` + `lto = true` should bring it well under target. Full optimization can be deferred since Phase 1 code is minimal.

## Sources

### Primary (HIGH confidence)
- wasm-pack official docs (build command, targets, Cargo.toml config) — https://rustwasm.github.io/docs/wasm-pack/commands/build.html — verified 2026-03-02. Note: docs being migrated to drager.github.io/wasm-pack
- wasm-bindgen official docs (deployment targets, serde integration, size optimization) — https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html — verified 2026-03-02. Note: docs being migrated to wasm-bindgen.github.io
- Vite official docs (WebAssembly features, v7.3.x) — https://vite.dev/guide/features.html#webassembly — verified 2026-03-02
- vite-plugin-wasm GitHub repo (Menci/vite-plugin-wasm, v3.5.0, 405 stars) — https://github.com/Menci/vite-plugin-wasm — verified 2026-03-02
- Local toolchain verification: rustc 1.93.1, Node 25.6.1, npm 11.10.0 — verified on development machine

### Secondary (MEDIUM confidence)
- wasm-pack non-rustup setup docs — verified for Arch Linux: `rust-wasm` package provides wasm32-unknown-unknown target matching system Rust version
- wasm-pack Cargo.toml configuration (wasm-opt profiles) — https://rustwasm.github.io/docs/wasm-pack/cargo-toml-configuration.html

### Tertiary (LOW confidence)
- `--target web` vs `--target bundler` with vite-plugin-wasm: Based on reading both docs but not empirically tested. The plugin says it supports wasm-pack modules; the exact target preference is unclear.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all tools verified via official docs, versions confirmed, packages available on system
- Architecture: HIGH - Cargo workspace + wasm-pack + Vite is the canonical pattern, well-documented
- Pitfalls: HIGH - toolchain state verified on actual development machine (missing wasm-pack, missing rust-wasm target)
- Code examples: MEDIUM - patterns from official docs adapted for this project's specific structure; not yet build-tested

**Research date:** 2026-03-02
**Valid until:** 2026-04-02 (stable toolchain, 30-day validity)