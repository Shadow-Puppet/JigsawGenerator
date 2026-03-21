# Phase 2: Grid Engine & Data Model - Research

**Researched:** 2026-03-02
**Domain:** Computational geometry / grid layout engine in Rust (pure lib + WASM)
**Confidence:** HIGH

## Summary

This phase builds the core puzzle grid engine: a data model that represents an N×M grid of pieces with shared edges, deterministic seeded randomness, configurable physical dimensions in millimeters, and a pluggable connector trait. The primary output is abstract geometry (bezier control points via kurbo types), not SVG.

The standard stack is well-established: `kurbo` 0.13.0 for 2D geometry types (Point, CubicBez), `rand` 0.10.0 + `rand_chacha` 0.10.0 for deterministic seeded RNG, and `serde` 1.0 for serialization. All libraries are mature, well-documented, and WASM-compatible. The shared-edge data model is the primary architectural challenge — each internal edge must exist exactly once in memory, referenced by the two adjacent pieces.

**Primary recommendation:** Build the grid as a struct owning two flat `Vec`s of edges (horizontal and vertical), with pieces computed as views/indices into those edge arrays. Use `ChaCha8Rng::seed_from_u64()` for deterministic seeding — ChaCha8 is sufficient since we need reproducibility, not cryptographic security. Enable kurbo's `serde` feature for serialization of geometry types across the WASM boundary.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Millimeters as primary internal unit for all engine math
- API accepts both mm and inches via a unit enum; inches converted to mm immediately on input
- Default puzzle size: A4 landscape (297x210mm)
- Default grid: 6x8 (48 pieces)
- Output can convert back to inches for display, but all storage/computation in mm
- Grid size: minimum 2x2, maximum 100x100
- Tab size: 15-45% of edge length, default 25%
- Jitter amount: 0-100%, default 50%
- 0% jitter = all connectors identical; 100% = maximum variation while staying geometrically valid
- Tab direction (in/out) randomly assigned per edge from seed — no alternating pattern
- Single seed controls everything: tab directions AND jitter control point offsets
- Same seed = pixel-identical puzzle output across runs and platforms
- Seed input: string-based, hashed to u64 (e.g. user types "birthday" which deterministically hashes to a u64 for rand_chacha)
- Auto-generate a random seed by default; user can override with their own string
- Seed displayed in UI for copying/sharing
- Edge-level trait: takes edge parameters (length, direction, jitter params) and returns abstract bezier control points
- Trait handles internal edges only; border edges (straight lines + optional rounded corners) handled separately outside the trait
- Trait includes validation: generated paths checked for geometric validity (no overlapping adjacent pieces, stays within bounds)
- Output is Vec of cubic bezier control points (kurbo types), NOT SVG path data
- Only one implementation in v1 (classic knob), but trait designed to accept new types (wavy, angular, flat) in v2

### Claude's Discretion
- Exact hashing algorithm for string-to-u64 seed conversion
- Internal data structure choices for shared-edge storage
- Validation thresholds for geometric validity checks
- How piece count breakdown is computed and returned (struct shape)
- Rounded corner radius parameter range and default

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| GRID-01 | User can configure puzzle grid as rows x columns | GridConfig struct expansion with validation (min 2x2, max 100x100), existing `compute_piece_breakdown` already handles row/col |
| GRID-02 | User can set puzzle physical size in millimeters or inches | Unit enum + conversion at API boundary, all internal math in mm (f64), kurbo Point for coordinates |
| GRID-03 | User can control tab/knob size as percentage of edge length | TabConfig with percentage field (15-45%, default 25%), validated at construction, passed to connector trait |
| GRID-04 | User can control jitter/randomness amount per edge | JitterConfig with amount field (0-100%, default 50%), seeded RNG generates per-edge offsets scaled by jitter amount |
| GRID-05 | User can set rounded corner radius on puzzle border | BorderConfig with corner_radius field (mm), applied to border path generation (outside connector trait) |
| GRID-06 | User can see piece count breakdown (total, edge, corner, interior) | Already implemented in `compute_piece_breakdown()`, may need slight adjustment for min 2x2 validation |
| CONN-03 | User can set a seed value to reproduce exact puzzle configurations | String seed → u64 hash → `ChaCha8Rng::seed_from_u64()`, deterministic across platforms |
| INFR-02 | Connector generation uses pluggable trait/interface | `ConnectorGenerator` trait with `generate_edge()` method returning `Vec<CubicBez>` |

</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| kurbo | 0.13.0 | 2D geometry types (Point, CubicBez, BezPath, Rect) | Linebender ecosystem standard; used by Vello, xilem; well-maintained by Raph Levien. `CubicBez` has 4 `Point` fields (p0-p3), implements `Copy`, `Clone`, `Serialize`, `Deserialize`. |
| rand | 0.10.0 | RNG traits and distributions | Rust ecosystem standard for randomness; `Rng` trait, `SeedableRng` |
| rand_chacha | 0.10.0 | ChaCha-based deterministic PRNG | Portable, reproducible across platforms; `ChaCha8Rng` provides `seed_from_u64()` |
| serde | 1.0 | Serialization/deserialization | Already in use; needed for JSON boundary and kurbo type serialization |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde_json | 1.0 | JSON serialization | Already in puzzle-wasm for WASM boundary |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| kurbo CubicBez | Raw `[f64; 8]` arrays | kurbo provides rich geometry API (arc length, nearest point, bounding box) needed in Phase 3; types are Copy and cheap |
| ChaCha8Rng | ChaCha20Rng | ChaCha20 is overkill for non-crypto puzzle generation; ChaCha8 is faster with same reproducibility guarantees |
| std hash for string→u64 | Custom hash | std `DefaultHasher` is NOT portable across Rust versions; must use a fixed algorithm (see pitfalls) |

**Installation (puzzle-core Cargo.toml additions):**
```toml
[dependencies]
kurbo = { version = "0.13", features = ["serde"] }
rand = { version = "0.10", default-features = false }
rand_chacha = "0.10"
serde = { version = "1.0", features = ["derive"] }
```

**Key notes:**
- `kurbo` needs `serde` feature enabled for `CubicBez`/`Point` serialization
- `rand` with `default-features = false` avoids pulling in `getrandom`/OS entropy (not needed in WASM, and we only use deterministic seeding)
- `rand_chacha` 0.10.0 depends on `rand_core` 0.10.0 which aligns with `rand` 0.10.0

## Architecture Patterns

### Recommended Project Structure
```
crates/puzzle-core/src/
├── lib.rs              # Public API, re-exports
├── grid.rs             # Grid struct, cell/edge layout, shared-edge model
├── config.rs           # GridConfig, TabConfig, JitterConfig, BorderConfig, Unit enum
├── edge.rs             # Edge struct, EdgeId, EdgeDirection, edge parameters
├── piece.rs            # Piece struct (references into edge arrays), PieceBreakdown
├── seed.rs             # String-to-u64 hashing, RNG construction
├── connector.rs        # ConnectorGenerator trait definition
└── validation.rs       # Geometric validity checks for generated edges
```

### Pattern 1: Shared-Edge via Indexed Arrays
**What:** Store horizontal edges in one `Vec` and vertical edges in another. Edges are identified by `(row, col)` index. Pieces reference edges by index, not by owning them.
**When to use:** Any grid-based shared-edge model.
**Example:**
```rust
/// Horizontal edges: rows+1 rows of cols edges each
/// h_edges[(row, col)] = horizontal edge between piece (row-1, col) and (row, col)
/// Vertical edges: rows rows of cols+1 edges each  
/// v_edges[(row, col)] = vertical edge between piece (row, col-1) and (row, col)
pub struct PuzzleGrid {
    pub config: GridConfig,
    /// Horizontal edges: (rows+1) * cols total
    /// Index: row * cols + col, where row 0..=rows, col 0..cols
    pub h_edges: Vec<Edge>,
    /// Vertical edges: rows * (cols+1) total
    /// Index: row * (cols+1) + col, where row 0..rows, col 0..=cols
    pub v_edges: Vec<Edge>,
}

pub struct Edge {
    pub start: kurbo::Point,
    pub end: kurbo::Point,
    pub is_border: bool,
    pub direction: TabDirection, // In or Out (meaningless for borders)
    pub connector: Option<Vec<kurbo::CubicBez>>, // None for borders until generated
}

pub enum TabDirection {
    In,
    Out,
}
```
**Source:** Standard grid data structure pattern for puzzle/tile engines.

### Pattern 2: Deterministic Seeded RNG
**What:** Hash user string to u64, seed ChaCha8Rng, use sequentially for all random decisions in deterministic order.
**When to use:** Any time reproducible procedural generation is needed.
**Example:**
```rust
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn create_rng(seed_string: &str) -> ChaCha8Rng {
    let hash = hash_seed_string(seed_string);
    ChaCha8Rng::seed_from_u64(hash)
}

/// Hash a seed string to u64 using a portable, deterministic algorithm.
/// CRITICAL: Do NOT use std::hash::DefaultHasher — it is NOT stable across Rust versions.
fn hash_seed_string(s: &str) -> u64 {
    // Use a simple FNV-1a or SipHash with fixed keys
    // FNV-1a is simplest and perfectly adequate:
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for byte in s.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

fn assign_tab_directions(rng: &mut ChaCha8Rng, edge_count: usize) -> Vec<TabDirection> {
    (0..edge_count)
        .map(|_| if rng.random_bool(0.5) { TabDirection::In } else { TabDirection::Out })
        .collect()
}
```

### Pattern 3: Connector Trait (Strategy Pattern)
**What:** Define a trait for edge connector generation that can be swapped without modifying grid/edge logic.
**When to use:** When connector shape algorithms should be independently pluggable.
**Example:**
```rust
use kurbo::CubicBez;

/// Parameters describing an edge for connector generation
pub struct EdgeParams {
    pub length: f64,           // Edge length in mm
    pub direction: TabDirection, // In or Out
    pub tab_size: f64,         // Tab size as fraction (0.15..=0.45)
    pub jitter_amount: f64,    // Jitter as fraction (0.0..=1.0)
    pub jitter_seed: u64,      // Per-edge seed for jitter RNG
}

/// Trait for generating connector shapes on internal edges
pub trait ConnectorGenerator: Send + Sync {
    /// Generate bezier curves for an edge connector.
    /// Returns control points in edge-local coordinates
    /// (0,0 = edge start, (length, 0) = edge end).
    fn generate(&self, params: &EdgeParams) -> Vec<CubicBez>;
    
    /// Validate that generated curves stay within acceptable bounds.
    fn validate(&self, curves: &[CubicBez], params: &EdgeParams) -> Result<(), String>;
}
```

### Pattern 4: Unit Conversion at the Boundary
**What:** Accept user input in mm or inches, convert to mm immediately, work in mm internally, convert back at output.
**When to use:** Any system with mixed unit inputs.
**Example:**
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Unit {
    Millimeters,
    Inches,
}

impl Unit {
    pub fn to_mm(&self, value: f64) -> f64 {
        match self {
            Unit::Millimeters => value,
            Unit::Inches => value * 25.4,
        }
    }
    
    pub fn from_mm(&self, value_mm: f64) -> f64 {
        match self {
            Unit::Millimeters => value_mm,
            Unit::Inches => value_mm / 25.4,
        }
    }
}
```

### Anti-Patterns to Avoid
- **Piece owns its edges:** Each piece storing its own edge data duplicates edges between adjacent pieces. Use index-based references into shared edge arrays instead.
- **Using `std::collections::hash_map::DefaultHasher` for seed hashing:** Its algorithm is NOT guaranteed stable across Rust compiler versions. Always use a fixed hash algorithm (FNV-1a or SipHash with hardcoded keys).
- **Generating random values in arbitrary order:** Tab direction and jitter must be generated in a fixed, deterministic sequence (e.g., iterate edges row-major) to ensure seed reproducibility.
- **Storing geometry in integer coordinates:** Use f64 throughout for mm coordinates. Integer truncation causes visible artifacts in bezier curves.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| 2D point/vector math | Custom Point struct | `kurbo::Point`, `kurbo::Vec2` | Handles arithmetic operators, lerp, distance; Copy + Serialize |
| Cubic bezier representation | Raw `[f64; 8]` | `kurbo::CubicBez` | Has `eval()`, `arclen()`, `nearest()`, `bounding_box()`, `inflections()` — all needed for validation and Phase 3 |
| Bounding box computation | Manual min/max tracking | `kurbo::Rect`, `CubicBez::bounding_box()` | Handles bezier extrema correctly (not just endpoint min/max) |
| Deterministic cross-platform RNG | Custom PRNG | `rand_chacha::ChaCha8Rng` | Tested against reference vectors; portable; `seed_from_u64` handles low-quality seeds well |
| Random boolean with probability | Manual bit manipulation | `rng.random_bool(0.5)` | Correctly handles edge cases, uses full entropy |

**Key insight:** kurbo's `CubicBez` is not just a data container — it implements `ParamCurve`, `ParamCurveArclen`, `ParamCurveNearest`, `ParamCurveExtrema`, and `Shape`. These will be critical for validation (bounding box checks) and for Phase 3 (SVG path generation via `path_elements()`).

## Common Pitfalls

### Pitfall 1: Non-Portable String Hashing
**What goes wrong:** Using `std::collections::hash_map::DefaultHasher` or `#[derive(Hash)]` to convert seed strings to u64. The hash algorithm changed between Rust versions and is explicitly documented as non-portable.
**Why it happens:** It's the most obvious/ergonomic approach in Rust.
**How to avoid:** Implement FNV-1a hash manually (trivial ~5 lines) or use a fixed SipHash with hardcoded keys. The exact algorithm is Claude's discretion per CONTEXT.md.
**Warning signs:** Tests pass on one machine but produce different puzzles on another, or after a Rust toolchain update.

### Pitfall 2: Edge Indexing Off-By-One
**What goes wrong:** An N×M grid has (N+1)×M horizontal edges and N×(M+1) vertical edges. Getting the count wrong corrupts the shared-edge model.
**Why it happens:** Confusing "edges between rows" with "edges per row", or forgetting border edges.
**How to avoid:** Clear naming: `h_edges` has `(rows + 1) * cols` elements; `v_edges` has `rows * (cols + 1)` elements. Write explicit tests for 2x2, 2x3, and 3x3 grids verifying edge counts.
**Warning signs:** Index out of bounds panics; pieces at grid boundaries reference wrong edges.

### Pitfall 3: Non-Deterministic Iteration Order
**What goes wrong:** Using `HashMap` or `HashSet` iteration to generate random values, producing different results per run even with the same seed.
**Why it happens:** HashMap iteration order is randomized in Rust for security.
**How to avoid:** Always iterate edges/pieces in a fixed order (row-major index). Use `Vec` with index-based lookup, not `HashMap`.
**Warning signs:** Same seed produces different puzzles on different runs.

### Pitfall 4: getrandom Panic in WASM
**What goes wrong:** Importing `rand` with default features pulls in `getrandom`, which panics on `wasm32-unknown-unknown` without the `wasm_js` backend configured.
**Why it happens:** `rand` 0.10's default features include OS RNG support which requires `getrandom`.
**How to avoid:** Use `rand = { version = "0.10", default-features = false }` in puzzle-core. We only need `SeedableRng` and `Rng` traits, not OS entropy. The seed comes from JavaScript via the WASM boundary.
**Warning signs:** Runtime panic: "getrandom: this target is not supported" when loading WASM.

### Pitfall 5: f64 Precision Across Platforms
**What goes wrong:** Floating-point operations may produce slightly different results on different platforms (x86 vs ARM, different optimization levels), breaking "pixel-identical" seed guarantee.
**Why it happens:** IEEE 754 allows implementation-specific behavior for transcendental functions.
**How to avoid:** Our computations are primarily arithmetic (+, -, *, /) which ARE deterministic in IEEE 754. Avoid `sin()`, `cos()`, `sqrt()` in the grid layout phase. If needed later, use kurbo's `libm` feature for portable math. The RNG itself (ChaCha8) is fully portable.
**Warning signs:** Grid layouts differ between native tests and WASM execution.

### Pitfall 6: Connector Geometry Exceeds Bounds
**What goes wrong:** Generated bezier curves for connectors extend beyond the piece boundary, causing overlap with adjacent pieces.
**Why it happens:** Jitter offsets push control points too far; tab size percentage too large for short edges.
**How to avoid:** Use `CubicBez::bounding_box()` to verify generated curves stay within a tolerance zone. Clamp jitter offsets proportionally to remaining space. The `validate()` method on the connector trait enforces this.
**Warning signs:** Adjacent piece edges visually intersect when rendered.

## Code Examples

Verified patterns from official documentation:

### Creating a CubicBez (kurbo 0.13.0)
```rust
// Source: https://docs.rs/kurbo/0.13.0/kurbo/struct.CubicBez.html
use kurbo::{CubicBez, Point};

let cubic = CubicBez::new(
    Point::new(0.0, 0.0),   // p0: start
    Point::new(10.0, 20.0),  // p1: control point 1
    Point::new(40.0, 20.0),  // p2: control point 2
    Point::new(50.0, 0.0),   // p3: end
);

// Evaluate point at parameter t
let midpoint = cubic.eval(0.5);

// Get bounding box (handles bezier extrema correctly)
let bbox = cubic.bounding_box();

// Check if curve is geometrically valid
assert!(cubic.is_finite());
```

### Seeding ChaCha8Rng (rand_chacha 0.10.0)
```rust
// Source: https://docs.rs/rand_chacha/0.10.0/rand_chacha/
// Source: https://docs.rs/rand_core/0.10.0/rand_core/trait.SeedableRng.html
use rand::SeedableRng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

// Seed from u64 — deterministic, portable
let mut rng = ChaCha8Rng::seed_from_u64(12345u64);

// Generate random bool (for tab direction)
let tab_out: bool = rng.random_bool(0.5);

// Generate random f64 in range (for jitter offsets)
let jitter: f64 = rng.random_range(-1.0..=1.0);
```

### kurbo Serde Serialization
```rust
// kurbo types with serde feature implement Serialize/Deserialize
// Source: https://docs.rs/kurbo/0.13.0/kurbo/ (feature flags section)
use kurbo::{CubicBez, Point};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct EdgeData {
    curves: Vec<CubicBez>,
    start: Point,
    end: Point,
}

// Serializes to JSON for WASM boundary
let json = serde_json::to_string(&edge_data)?;
```

### Piece Edge Indexing
```rust
/// Get the four edge indices for a piece at (row, col) in a grid
fn piece_edges(row: usize, col: usize, cols: usize) -> PieceEdges {
    PieceEdges {
        top:    (row, col),         // h_edges index: row * cols + col
        bottom: (row + 1, col),     // h_edges index: (row+1) * cols + col
        left:   (row, col),         // v_edges index: row * (cols+1) + col
        right:  (row, col + 1),     // v_edges index: row * (cols+1) + (col+1)
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| rand 0.8 + rand_chacha 0.3 | rand 0.10 + rand_chacha 0.10 | 2025 | `gen_bool` → `random_bool`, `gen_range` → `random_range`; `SeedableRng` moved to `rand_core` but re-exported |
| getrandom 0.2 | getrandom 0.3 | 2025 | WASM backend selection now via `--cfg getrandom_backend` instead of feature flags |
| kurbo 0.11 | kurbo 0.13 | 2025 | Added `schemars` support; stable API for CubicBez, Point |

**Deprecated/outdated:**
- `rand::thread_rng()` → now `rand::rng()` in 0.10
- `rng.gen::<bool>()` → now `rng.random::<bool>()` or `rng.random_bool(p)` in 0.10
- `rng.gen_range(a..b)` → now `rng.random_range(a..b)` in 0.10

## Open Questions

1. **Rounded corner radius: what range and default?**
   - What we know: User decision says "rounded corner radius on puzzle border" (GRID-05). CONTEXT.md lists this as Claude's discretion.
   - What's unclear: Appropriate range for laser-cut puzzles.
   - Recommendation: Default 2.0mm, range 0.0-10.0mm. 0.0 = sharp corners. 2mm is a common laser-cutting-friendly radius. kurbo has `RoundedRect` with `RoundedRectRadii` but we may not use it directly since borders are separate from the connector trait.

2. **Per-edge jitter seed derivation**
   - What we know: Single master seed, but each edge needs independent jitter variation.
   - What's unclear: Whether to use a single RNG stream (generate all jitter in fixed order) or derive per-edge sub-seeds.
   - Recommendation: Single RNG stream, generate values in deterministic row-major order. Simpler and equally deterministic. Per-edge sub-seeds add complexity without benefit since we always generate the full grid.

3. **Validation thresholds for geometric validity**
   - What we know: Connector trait includes validation. Need to check bounding boxes.
   - What's unclear: How much tolerance to allow beyond the theoretical piece boundary.
   - Recommendation: Allow connector curves to extend up to 5% beyond the nominal piece boundary in each direction (provides visual margin). Reject if bounding box exceeds this. Exact threshold is Claude's discretion per CONTEXT.md.

## Sources

### Primary (HIGH confidence)
- kurbo 0.13.0 docs: https://docs.rs/kurbo/0.13.0/kurbo/ — CubicBez struct, Point struct, feature flags, serde support verified
- rand_chacha 0.10.0 docs: https://docs.rs/rand_chacha/0.10.0/rand_chacha/ — ChaCha8Rng, seeding, portability guarantees verified
- rand 0.10.0 docs: https://docs.rs/rand/0.10.0/rand/ — Rng trait, SeedableRng, API changes from 0.8
- rand_core 0.10.0 SeedableRng: https://docs.rs/rand_core/0.10.0/rand_core/trait.SeedableRng.html — `seed_from_u64` documented behavior and portability
- getrandom 0.3.3 docs: https://docs.rs/getrandom/0.3.3/getrandom/ — WASM backend configuration, `wasm_js` feature requirement

### Secondary (MEDIUM confidence)
- Existing puzzle-core codebase: `crates/puzzle-core/src/lib.rs` — GridConfig, PieceBreakdown, compute_piece_breakdown() verified
- Existing puzzle-wasm codebase: `crates/puzzle-wasm/src/lib.rs` — JSON boundary pattern verified
- rand 0.10 migration: API rename patterns (gen → random, gen_range → random_range) verified against official docs

### Tertiary (LOW confidence)
- FNV-1a hash constants: Well-known constants from FNV specification, but implementation should be verified with test vectors

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified on docs.rs with current versions, WASM compatibility confirmed
- Architecture: HIGH — shared-edge indexed array is a well-established pattern for grid-based puzzle engines; connector trait is standard Strategy pattern
- Pitfalls: HIGH — getrandom WASM issue, DefaultHasher portability, and f64 determinism are well-documented Rust ecosystem issues
- Code examples: HIGH — all examples use verified API from official docs (rand 0.10, kurbo 0.13)

**Research date:** 2026-03-02
**Valid until:** 2026-04-02 (stable libraries, 30-day validity)