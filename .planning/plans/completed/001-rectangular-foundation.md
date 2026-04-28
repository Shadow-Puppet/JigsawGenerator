# 001 — Rectangular foundation

**What:** Everything needed to generate and export a rectangular jigsaw puzzle from a web GUI.
**Why:** Prove the Rust-core → WASM-bridge → Canvas-frontend pipeline end-to-end before layering on boundary/whimsy features.

## What shipped

Four slices, all merged to `main` on 2026-03-03:

- **S01 — Build pipeline & WASM foundation.** Cargo workspace (`puzzle-core` pure Rust + `puzzle-wasm` thin facade), Vite + `vite-plugin-wasm` frontend, JSON-in/JSON-out WASM boundary with typed discriminated unions in TypeScript. 48 KB gzipped WASM after round-trip proof-of-life.
- **S02 — Grid engine & data model.** `PuzzleConfig` with full validation, `Unit` enum (mm/inches convert at boundary), FNV-1a `hash_seed` → `ChaCha8Rng`, `Edge`/`EdgeParams`/`TabDirection` types, `ConnectorGenerator` trait, `PuzzleGrid` with shared-edge h_edges/v_edges model, piece indexing + type classification, `generate_grid` WASM endpoint.
- **S03 — Classic knob connector & SVG export.** `ClassicKnobConnector` producing 5 cubic bezier segments per knob (baseline → neck → body → top → body → neck → baseline) with visible neck narrowing for snap-fit. Separate `"{seed}-connectors"` RNG stream. `svg_export.rs` builds a single-`<path>` SVG with rounded border corners, edge-local → global bezier transforms via `Affine::translate * Affine::rotate`, and polyline kerf compensation with miter/bevel joins. `generate_svg` WASM endpoint.
- **S04 — Web GUI & live preview.** Two-column responsive layout, all parameter controls, live regeneration without debounce (WASM is fast enough), URL param sync via `history.replaceState`, compact URL encoding (`w`/`h`, `mm`/`in`, integer percentages), SVG download with descriptive filenames, copy-link-to-clipboard, and CSS stroke-width override so laser-hairline paths are visible on screen but preserved in the download.

## Key patterns established (referenced by M002)

- **Shared-edge indexing:** `top=row*cols+col`, `bottom=(row+1)*cols+col`, `left=row*(cols+1)+col`, `right=row*(cols+1)+(col+1)`. Pieces reference edges by index; edges are never cloned.
- **Fixed RNG consumption order:** all h_edges row-major, then all v_edges row-major. Changing this would break determinism across versions.
- **Border edges always `TabDirection::In`** (unused but consistent default); internal edges randomised.
- **WASM boundary pattern:** JSON in → deserialize to puzzle-core types → call engine → serialize WASM-specific response types → JSON out. Error shape: `{"error":"message"}`.
- **CSS display override vs export:** laser-cutter SVG uses 0.001 mm hairlines; frontend applies `stroke-width: 0.5px !important` on screen, download preserves original attributes.
- **URL abbreviations:** `w`/`h` for dimensions, unit as `mm`/`in`, tab/jitter/etc. as integer percentages for compact shareable links.

Final test count after M001: 113 tests (98 puzzle-core + 15 puzzle-wasm). WASM binary ~93 KB gzipped.
