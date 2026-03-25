use std::cell::RefCell;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use puzzle_core::{
    border_to_binary, compute_piece_breakdown, edges_to_binary, heart_path, star_path,
    BoundaryPuzzle, ClassicKnobConnector, GridConfig, PieceType, PuzzleConfig, PuzzleGrid,
};

thread_local! {
    static CACHED_SVG: RefCell<String> = RefCell::new(String::new());
}

/// Resolve a border shape name to a BezPath at the given dimensions.
///
/// Returns `Ok(BezPath)` for known shapes ("heart", "star"), or
/// `Err(error_message)` for unknown shape names.
fn resolve_border_shape(
    name: &str,
    width: f64,
    height: f64,
) -> Result<kurbo::BezPath, String> {
    match name {
        "heart" => Ok(heart_path(width, height)),
        "star" => Ok(star_path(width, height, 5)),
        other => Err(format!("Unknown border shape: {}", other)),
    }
}

/// Initialize the panic hook for better error messages in the browser console.
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Compute puzzle piece breakdown from a JSON configuration string.
///
/// Expects JSON: `{"rows": N, "cols": M}`
/// Returns JSON: `{"total": N, "corners": N, "edges": N, "interior": N}`
/// Or on error: `{"error": "message"}`
#[wasm_bindgen]
pub fn compute_pieces(config_json: &str) -> String {
    let config: GridConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"Invalid JSON: {}"}}"#, e),
    };

    match compute_piece_breakdown(&config) {
        Ok(breakdown) => serde_json::to_string(&breakdown)
            .unwrap_or_else(|e| format!(r#"{{"error":"Serialization error: {}"}}"#, e)),
        Err(msg) => format!(r#"{{"error":"{}"}}"#, msg),
    }
}

// ─── WASM Response Types ──────────────────────────────────────────

/// Simplified piece info for JSON output (no internal edge indices).
#[derive(Debug, Serialize)]
struct PieceInfo {
    row: usize,
    col: usize,
    piece_type: String,
    is_top_border: bool,
    is_bottom_border: bool,
    is_left_border: bool,
    is_right_border: bool,
}

/// Piece count breakdown in the grid response.
#[derive(Debug, Serialize)]
struct PieceBreakdownInfo {
    total: usize,
    corners: usize,
    edges: usize,
    interior: usize,
}

/// Edge count summary in the grid response.
#[derive(Debug, Serialize)]
struct EdgeSummary {
    h_edge_count: usize,
    v_edge_count: usize,
    border_count: usize,
    internal_count: usize,
}

/// Full grid response returned by the generate_grid WASM endpoint.
///
/// This is a WASM-layer concern — it selects what data to expose to JS,
/// not the full internal Edge/Piece structs. Full edge geometry (bezier
/// control points) will be added in Phase 3 when connectors are generated.
#[derive(Debug, Serialize)]
struct GridResponse {
    rows: u32,
    cols: u32,
    width_mm: f64,
    height_mm: f64,
    seed: String,
    piece_breakdown: PieceBreakdownInfo,
    edge_summary: EdgeSummary,
    pieces: Vec<PieceInfo>,
}

/// Generate a puzzle grid from a JSON configuration string.
///
/// Accepts full PuzzleConfig JSON:
/// ```json
/// {
///   "rows": 6, "cols": 8,
///   "width": 297.0, "height": 210.0,
///   "unit": "Millimeters",
///   "tab": { "size_pct": 0.25, "taper": 0.5 },
///   "seed": "my-puzzle-seed"
/// }
/// ```
///
/// Returns JSON GridResponse with grid summary, piece breakdown,
/// edge summary, and per-piece info.
///
/// On error: `{"error": "message"}`
///
/// **Seed handling:** If `seed` is empty, a fixed default seed "default" is used.
/// WASM cannot access OS entropy (no getrandom), so true random seeds must be
/// generated in JavaScript and passed in. Phase 4 will pass a JS-generated
/// random seed from the browser.
#[wasm_bindgen]
pub fn generate_grid(config_json: &str) -> String {
    // 1. Deserialize PuzzleConfig from JSON
    let mut config: PuzzleConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"Invalid JSON: {}"}}"#, e),
    };

    // Handle empty seed: use fixed default since WASM has no OS entropy.
    // Phase 4 will pass JS-generated random seeds from the browser.
    if config.seed.is_empty() {
        config.seed = "default".to_string();
    }

    let seed_used = config.seed.clone();

    // 2. Create PuzzleGrid (validates config internally)
    let grid = match PuzzleGrid::new(config) {
        Ok(g) => g,
        Err(e) => return format!(r#"{{"error":"{}"}}"#, e),
    };

    // 3. Build response
    let rows = grid.config.rows;
    let cols = grid.config.cols;
    let rows_usize = rows as usize;
    let cols_usize = cols as usize;
    let border_shape = grid.config.border_shape.clone();
    let grid_width = grid.config.width;
    let grid_height = grid.config.height;

    // Unify grid access: when boundary puzzle is active, grid lives inside it
    enum GridAccess {
        Owned(PuzzleGrid),
        InBoundary(BoundaryPuzzle),
    }

    let access = if let Some(ref shape_name) = border_shape {
        let boundary = match resolve_border_shape(shape_name, grid_width, grid_height) {
            Ok(b) => b,
            Err(e) => return format!(r#"{{"error":"{}"}}"#, e),
        };
        GridAccess::InBoundary(BoundaryPuzzle::new(grid, boundary))
    } else {
        GridAccess::Owned(grid)
    };

    let (grid_ref, bp_opt): (&PuzzleGrid, Option<&BoundaryPuzzle>) = match &access {
        GridAccess::Owned(g) => (g, None),
        GridAccess::InBoundary(bp) => (&bp.grid, Some(bp)),
    };

    // Piece breakdown
    let all_pieces = grid_ref.pieces();

    // Filter pieces to only included cells when boundary is active
    let filtered_pieces: Vec<_> = if let Some(bp) = bp_opt {
        all_pieces
            .iter()
            .filter(|p| bp.cell_inside[p.row][p.col])
            .collect()
    } else {
        all_pieces.iter().collect()
    };

    let corners = filtered_pieces
        .iter()
        .filter(|p| p.piece_type == PieceType::Corner)
        .count();
    let edges = filtered_pieces
        .iter()
        .filter(|p| p.piece_type == PieceType::Edge)
        .count();
    let interior = filtered_pieces
        .iter()
        .filter(|p| p.piece_type == PieceType::Interior)
        .count();

    // Edge summary
    let border_count = grid_ref
        .h_edges
        .iter()
        .chain(grid_ref.v_edges.iter())
        .filter(|e| e.is_border)
        .count();
    let total_edges = grid_ref.h_edges.len() + grid_ref.v_edges.len();
    let internal_count = if let Some(bp) = bp_opt {
        bp.included_edge_count()
    } else {
        total_edges - border_count
    };

    // Per-piece info (only included pieces when boundary is active)
    let pieces: Vec<PieceInfo> = filtered_pieces
        .iter()
        .map(|p| {
            let piece_type_str = match p.piece_type {
                PieceType::Corner => "corner",
                PieceType::Edge => "edge",
                PieceType::Interior => "interior",
            };
            PieceInfo {
                row: p.row,
                col: p.col,
                piece_type: piece_type_str.to_string(),
                is_top_border: p.row == 0,
                is_bottom_border: p.row == rows_usize - 1,
                is_left_border: p.col == 0,
                is_right_border: p.col == cols_usize - 1,
            }
        })
        .collect();

    let response = GridResponse {
        rows,
        cols,
        width_mm: grid_ref.config.width,
        height_mm: grid_ref.config.height,
        seed: seed_used,
        piece_breakdown: PieceBreakdownInfo {
            total: filtered_pieces.len(),
            corners,
            edges,
            interior,
        },
        edge_summary: EdgeSummary {
            h_edge_count: grid_ref.h_edges.len(),
            v_edge_count: grid_ref.v_edges.len(),
            border_count,
            internal_count,
        },
        pieces,
    };

    // 4. Serialize to JSON
    serde_json::to_string(&response)
        .unwrap_or_else(|e| format!(r#"{{"error":"Serialization error: {}"}}"#, e))
}

/// Generate a laser-cutter-ready SVG from a JSON configuration string.
///
/// Accepts full PuzzleConfig JSON (same as `generate_grid`).
///
/// Returns a complete SVG string with:
/// - Physical mm dimensions and viewBox
/// - Single `<path>` with all cut lines (border + connectors)
/// - Hairline black stroke, no fill
///
/// On error: `{"error": "message"}`
#[wasm_bindgen]
pub fn generate_svg(config_json: &str) -> String {
    // 1. Deserialize PuzzleConfig from JSON
    let mut config: PuzzleConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"Invalid JSON: {}"}}"#, e),
    };

    // Handle empty seed: use fixed default since WASM has no OS entropy.
    if config.seed.is_empty() {
        config.seed = "default".to_string();
    }

    let border_shape = config.border_shape.clone();

    // 2. Create PuzzleGrid (validates config internally)
    let mut grid = match PuzzleGrid::new(config) {
        Ok(g) => g,
        Err(e) => return format!(r#"{{"error":"{}"}}"#, e),
    };

    // 3. Generate connectors on all internal edges
    grid.generate_connectors(&ClassicKnobConnector);

    // 4. Generate and return SVG
    if let Some(ref shape_name) = border_shape {
        let boundary = match resolve_border_shape(shape_name, grid.config.width, grid.config.height) {
            Ok(b) => b,
            Err(e) => return format!(r#"{{"error":"{}"}}"#, e),
        };
        let bp = BoundaryPuzzle::new(grid, boundary);
        bp.generate_boundary_svg()
    } else {
        puzzle_core::generate_svg(&grid)
    }
}

/// Generate binary edge data for Canvas 2D rendering.
///
/// Returns a JS object with:
/// - `edges`: Float64Array of internal edge connector curves (36 floats per edge)
/// - `border`: Float64Array of border path drawing commands
/// - `width`: puzzle width in mm
/// - `height`: puzzle height in mm
///
/// Also caches the SVG string internally for retrieval via `get_cached_svg()`.
///
/// On error: returns a JS object with `error` property.
#[wasm_bindgen]
pub fn generate_edges_binary(config_json: &str) -> JsValue {
    // 1. Deserialize PuzzleConfig from JSON
    let mut config: PuzzleConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(e) => {
            let obj = js_sys::Object::new();
            let _ = js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("error"),
                &JsValue::from_str(&format!("Invalid JSON: {}", e)),
            );
            return obj.into();
        }
    };

    // Handle empty seed
    if config.seed.is_empty() {
        config.seed = "default".to_string();
    }

    let border_shape = config.border_shape.clone();

    // 2. Create PuzzleGrid
    let mut grid = match PuzzleGrid::new(config) {
        Ok(g) => g,
        Err(e) => {
            let obj = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("error"), &JsValue::from_str(&e));
            return obj.into();
        }
    };

    let width = grid.config.width;
    let height = grid.config.height;

    // 3. Generate connectors
    grid.generate_connectors(&ClassicKnobConnector);

    // 4-5. Generate SVG + binary data (boundary-aware when shape is set)
    let (svg, edges_data, border_data, piece_count) = if let Some(ref shape_name) = border_shape {
        let boundary = match resolve_border_shape(shape_name, width, height) {
            Ok(b) => b,
            Err(e) => {
                let obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("error"),
                    &JsValue::from_str(&e),
                );
                return obj.into();
            }
        };
        let bp = BoundaryPuzzle::new(grid, boundary);
        let count = bp.included_cells().len();
        let svg = bp.generate_boundary_svg();
        let edges = bp.boundary_edges_to_binary();
        let border = bp.boundary_border_to_binary();
        (svg, edges, border, count)
    } else {
        let rows = grid.config.rows as usize;
        let cols = grid.config.cols as usize;
        let count = rows * cols;
        let svg = puzzle_core::generate_svg(&grid);
        let edges = edges_to_binary(&grid);
        let border = border_to_binary(&grid);
        (svg, edges, border, count)
    };

    // Cache SVG for retrieval via get_cached_svg()
    CACHED_SVG.with(|c| {
        *c.borrow_mut() = svg;
    });

    // 6. Create Float64Arrays
    let edges_arr = js_sys::Float64Array::new_with_length(edges_data.len() as u32);
    edges_arr.copy_from(&edges_data);

    let border_arr = js_sys::Float64Array::new_with_length(border_data.len() as u32);
    border_arr.copy_from(&border_data);

    // 7. Build result object
    let obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("edges"), &edges_arr);
    let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("border"), &border_arr);
    let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("width"), &JsValue::from_f64(width));
    let _ = js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("height"),
        &JsValue::from_f64(height),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("piece_count"),
        &JsValue::from_f64(piece_count as f64),
    );

    obj.into()
}

/// Retrieve the cached SVG string from the last `generate_edges_binary()` call.
///
/// Returns the full SVG string with physical mm dimensions, suitable for
/// laser-cutter download. Returns empty string if no SVG has been generated yet.
#[wasm_bindgen]
pub fn get_cached_svg() -> String {
    CACHED_SVG.with(|c| c.borrow().clone())
}

/// Compute the safe maximum tab size for a given grid configuration.
///
/// Accepts JSON: `{"rows": N, "cols": M, "width": W, "height": H}`
/// Returns JSON: `{"max": 0.25}` (the safe maximum tab size_pct)
/// Or on error: `{"error": "message"}`
///
/// Note: Clamps tab/taper to valid ranges before creating the grid so this
/// function works even when the current slider value is out of range (e.g.
/// during initial load from a stale URL).
#[wasm_bindgen]
pub fn safe_tab_max(config_json: &str) -> String {
    let mut config: PuzzleConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"Invalid JSON: {}"}}"#, e),
    };

    // Clamp tab params to valid ranges so PuzzleGrid::new() doesn't reject
    // the config — the whole point of this function is to *compute* the max.
    config.tab.size_pct = config.tab.size_pct.clamp(0.15, 0.25);
    config.tab.taper = config.tab.taper.clamp(0.57, 1.32);
    if let Some(ref mut max) = config.tab.size_pct_max {
        *max = max.clamp(0.15, 0.25);
    }
    if let Some(ref mut max) = config.tab.taper_max {
        *max = max.clamp(0.57, 1.32);
    }

    let grid = match PuzzleGrid::new(config) {
        Ok(g) => g,
        Err(e) => return format!(r#"{{"error":"{}"}}"#, e),
    };

    let max = grid.safe_tab_max();
    format!(r#"{{"max":{:.4}}}"#, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_grid_valid_config() {
        let config_json = r#"{"rows":6,"cols":8,"width":297.0,"height":210.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"test-seed"}"#;
        let result = generate_grid(config_json);

        // Should be valid JSON, not an error
        assert!(!result.contains(r#""error""#), "Got error: {}", result);

        // Parse and check fields
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["rows"], 6);
        assert_eq!(parsed["cols"], 8);
        assert_eq!(parsed["width_mm"], 297.0);
        assert_eq!(parsed["height_mm"], 210.0);
        assert_eq!(parsed["seed"], "test-seed");

        // Check piece breakdown
        assert_eq!(parsed["piece_breakdown"]["total"], 48);
        assert_eq!(parsed["piece_breakdown"]["corners"], 4);

        // Check edge summary exists
        assert!(parsed["edge_summary"]["h_edge_count"].is_number());
        assert!(parsed["edge_summary"]["v_edge_count"].is_number());

        // Check pieces array
        let pieces = parsed["pieces"].as_array().unwrap();
        assert_eq!(pieces.len(), 48);
    }

    #[test]
    fn test_generate_grid_deterministic() {
        let config_json = r#"{"rows":4,"cols":5,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"determinism-test"}"#;
        let result1 = generate_grid(config_json);
        let result2 = generate_grid(config_json);

        assert_eq!(result1, result2, "Same seed must produce identical output");
    }

    #[test]
    fn test_generate_grid_empty_seed_uses_default() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":""}"#;
        let result = generate_grid(config_json);
        assert!(!result.contains(r#""error""#), "Got error: {}", result);

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["seed"], "default");
    }

    #[test]
    fn test_generate_grid_invalid_json() {
        let result = generate_grid("not valid json");
        assert!(result.contains(r#""error""#));
    }

    #[test]
    fn test_generate_grid_invalid_config() {
        // rows=1 is below minimum (2)
        let config_json = r#"{"rows":1,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"test"}"#;
        let result = generate_grid(config_json);
        assert!(result.contains(r#""error""#));
    }

    #[test]
    fn test_generate_grid_piece_types_correct() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"piece-types"}"#;
        let result = generate_grid(config_json);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let pieces = parsed["pieces"].as_array().unwrap();

        // 3x4 grid: 4 corners, 6 edges, 2 interior
        let corners = pieces
            .iter()
            .filter(|p| p["piece_type"] == "corner")
            .count();
        let edges = pieces.iter().filter(|p| p["piece_type"] == "edge").count();
        let interior = pieces
            .iter()
            .filter(|p| p["piece_type"] == "interior")
            .count();

        assert_eq!(corners, 4);
        assert_eq!(edges, 6);
        assert_eq!(interior, 2);
    }

    #[test]
    fn test_generate_grid_edge_counts() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"edge-counts"}"#;
        let result = generate_grid(config_json);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 3x4 grid: h_edges = (3+1)*4 = 16, v_edges = 3*(4+1) = 15
        assert_eq!(parsed["edge_summary"]["h_edge_count"], 16);
        assert_eq!(parsed["edge_summary"]["v_edge_count"], 15);
    }

    #[test]
    fn test_compute_pieces_still_works() {
        // Backward compatibility: compute_pieces endpoint unchanged
        let result = compute_pieces(r#"{"rows":3,"cols":4}"#);
        assert!(!result.contains(r#""error""#));

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["total"], 12);
        assert_eq!(parsed["corners"], 4);
        assert_eq!(parsed["edges"], 6);
        assert_eq!(parsed["interior"], 2);
    }

    #[test]
    fn test_generate_grid_json_roundtrip() {
        let config_json = r#"{"rows":6,"cols":8,"width":297.0,"height":210.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"test-seed"}"#;
        let result = generate_grid(config_json);

        // Verify it's valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result);
        assert!(parsed.is_ok(), "Result is not valid JSON: {}", result);

        // Verify no error
        let value = parsed.unwrap();
        assert!(value.get("error").is_none(), "Got error: {}", result);

        // Verify expected fields exist
        assert!(value.get("rows").is_some());
        assert!(value.get("cols").is_some());
        assert!(value.get("width_mm").is_some());
        assert!(value.get("height_mm").is_some());
        assert!(value.get("seed").is_some());
        assert!(value.get("piece_breakdown").is_some());
        assert!(value.get("edge_summary").is_some());
        assert!(value.get("pieces").is_some());
    }

    // ─── generate_svg Tests ───────────────────────────────────────

    #[test]
    fn test_generate_svg_returns_svg() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"svg-test"}"#;
        let result = generate_svg(config_json);
        assert!(
            result.starts_with("<svg"),
            "should start with <svg, got: {}...",
            &result[..50.min(result.len())]
        );
        assert!(result.contains("</svg>"), "should contain </svg>");
        assert!(!result.contains(r#""error""#), "should not be error JSON");
    }

    #[test]
    fn test_generate_svg_has_connectors() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"conn-svg"}"#;
        let result = generate_svg(config_json);
        // Extract path data
        let d_start = result.find("d='").expect("should have d attribute") + 3;
        let d_end = result[d_start..].find('\'').unwrap() + d_start;
        let path_data = &result[d_start..d_end];
        assert!(
            path_data.contains('C'),
            "path data should contain C commands (cubic bezier curves from connectors)"
        );
    }

    #[test]
    fn test_generate_svg_deterministic() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"determ-svg"}"#;
        let svg1 = generate_svg(config_json);
        let svg2 = generate_svg(config_json);
        assert_eq!(svg1, svg2, "same config must produce identical SVG");
    }

    #[test]
    fn test_generate_svg_invalid_json() {
        let result = generate_svg("not valid json");
        assert!(
            result.contains(r#""error""#),
            "should return error JSON for invalid input"
        );
    }

    // ─── Border Shape Tests ──────────────────────────────────────

    #[test]
    fn test_generate_svg_with_heart_border() {
        // Heart border SVG should contain cubic bezier curves from the heart shape,
        // not the rectangular border lines.
        let config_json = r#"{"rows":6,"cols":8,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"heart-svg","border_shape":"heart"}"#;
        let result = generate_svg(config_json);

        assert!(
            result.starts_with("<svg"),
            "should return SVG, got: {}...",
            &result[..80.min(result.len())]
        );
        assert!(!result.contains(r#""error""#), "should not be error JSON");

        // Heart border produces cubic bezier curves
        let d_start = result.find("d='").expect("should have d attribute") + 3;
        let d_end = result[d_start..].find('\'').unwrap() + d_start;
        let path_data = &result[d_start..d_end];

        assert!(
            path_data.contains('C'),
            "heart border SVG should contain C (cubic bezier) commands"
        );
    }

    #[test]
    fn test_generate_svg_with_star_border() {
        // Star border SVG should work and produce valid SVG with line-based border.
        let config_json = r#"{"rows":6,"cols":8,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"star-svg","border_shape":"star"}"#;
        let result = generate_svg(config_json);

        assert!(
            result.starts_with("<svg"),
            "should return SVG, got: {}...",
            &result[..80.min(result.len())]
        );
        assert!(!result.contains(r#""error""#), "should not be error JSON");

        // Star border produces line segments (L commands) in addition to M commands
        let d_start = result.find("d='").expect("should have d attribute") + 3;
        let d_end = result[d_start..].find('\'').unwrap() + d_start;
        let path_data = &result[d_start..d_end];

        assert!(
            path_data.contains('L'),
            "star border SVG should contain L (lineTo) commands from star polygon"
        );
    }

    #[test]
    fn test_generate_svg_no_border_shape_unchanged() {
        // Without border_shape, generate_svg should produce the same SVG as before.
        let config_with_none = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"compat-svg"}"#;
        let config_with_null = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"compat-svg","border_shape":null}"#;

        let svg_none = generate_svg(config_with_none);
        let svg_null = generate_svg(config_with_null);

        assert!(svg_none.starts_with("<svg"), "should be SVG");
        assert_eq!(
            svg_none, svg_null,
            "absent and null border_shape should produce identical SVG"
        );
    }

    #[test]
    fn test_generate_grid_with_border_shape_fewer_pieces() {
        // Heart border should include fewer pieces than the full rectangular grid.
        let config_no_border = r#"{"rows":6,"cols":8,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"pieces-cmp"}"#;
        let config_heart = r#"{"rows":6,"cols":8,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"pieces-cmp","border_shape":"heart"}"#;

        let result_full = generate_grid(config_no_border);
        let result_heart = generate_grid(config_heart);

        assert!(!result_full.contains(r#""error""#), "full grid error: {}", result_full);
        assert!(!result_heart.contains(r#""error""#), "heart grid error: {}", result_heart);

        let parsed_full: serde_json::Value = serde_json::from_str(&result_full).unwrap();
        let parsed_heart: serde_json::Value = serde_json::from_str(&result_heart).unwrap();

        let full_total = parsed_full["piece_breakdown"]["total"].as_u64().unwrap();
        let heart_total = parsed_heart["piece_breakdown"]["total"].as_u64().unwrap();

        assert!(
            heart_total < full_total,
            "heart border should have fewer pieces ({}) than full grid ({})",
            heart_total,
            full_total
        );
        assert!(
            heart_total > 0,
            "heart border should still have some pieces"
        );

        // Also check that pieces array length matches total
        let heart_pieces = parsed_heart["pieces"].as_array().unwrap();
        assert_eq!(
            heart_pieces.len() as u64, heart_total,
            "pieces array length should match total"
        );
    }

    #[test]
    fn test_border_shape_invalid_returns_error() {
        // Unknown shape name should return an error.
        let config_json = r#"{"rows":6,"cols":8,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"bad-shape","border_shape":"hexagon"}"#;

        let svg_result = generate_svg(config_json);
        assert!(
            svg_result.contains(r#""error""#),
            "unknown border_shape should return error in generate_svg, got: {}",
            svg_result
        );
        assert!(
            svg_result.contains("Unknown border shape"),
            "error should mention unknown shape"
        );

        let grid_result = generate_grid(config_json);
        assert!(
            grid_result.contains(r#""error""#),
            "unknown border_shape should return error in generate_grid"
        );
    }

    #[test]
    fn test_border_shape_resolution_heart() {
        // Verify resolve_border_shape produces a valid BezPath for "heart".
        let boundary = resolve_border_shape("heart", 200.0, 150.0);
        assert!(boundary.is_ok(), "heart should resolve to a valid BezPath");
    }

    #[test]
    fn test_border_shape_resolution_star() {
        // Verify resolve_border_shape produces a valid BezPath for "star".
        let boundary = resolve_border_shape("star", 200.0, 150.0);
        assert!(boundary.is_ok(), "star should resolve to a valid BezPath");
    }

    #[test]
    fn test_border_shape_resolution_unknown() {
        // Verify resolve_border_shape returns error for unknown shapes.
        let boundary = resolve_border_shape("hexagon", 200.0, 150.0);
        assert!(boundary.is_err(), "hexagon should return error");
        assert!(
            boundary.unwrap_err().contains("Unknown border shape"),
            "error should mention unknown shape"
        );
    }

    #[test]
    fn test_generate_svg_heart_border_deterministic() {
        // Same config with heart border should produce identical SVG.
        let config_json = r#"{"rows":6,"cols":8,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"determ-heart","border_shape":"heart"}"#;
        let svg1 = generate_svg(config_json);
        let svg2 = generate_svg(config_json);
        assert_eq!(
            svg1, svg2,
            "same seed + heart border must produce identical SVG"
        );
    }

    // ─── piece_count Tests ───────────────────────────────────────

    /// Verify the piece count logic that generate_edges_binary uses.
    ///
    /// generate_edges_binary returns JsValue which cannot be inspected in
    /// native tests (Reflect::get panics on non-wasm targets). Instead we
    /// verify the same logic via generate_grid (which returns JSON) and
    /// confirm WASM compilation via `cargo check --target wasm32-unknown-unknown`.
    #[test]
    fn test_piece_count_rectangular() {
        // For rectangular puzzles, piece_count should equal rows * cols.
        let config_json = r#"{"rows":4,"cols":5,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"pc-rect"}"#;
        let result = generate_grid(config_json);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let total = parsed["piece_breakdown"]["total"].as_u64().unwrap();
        assert_eq!(total, 4 * 5, "rectangular piece count should be rows * cols");
    }

    #[test]
    fn test_piece_count_heart_border() {
        // For heart-bordered puzzles, piece_count should be less than rows * cols
        // but greater than zero (some cells are excluded by the boundary).
        let config_json = r#"{"rows":6,"cols":8,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"pc-heart","border_shape":"heart"}"#;
        let result = generate_grid(config_json);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let total = parsed["piece_breakdown"]["total"].as_u64().unwrap();

        assert!(total > 0, "heart border should have some pieces, got 0");
        assert!(
            total < 6 * 8,
            "heart border should have fewer pieces than full grid ({}), got {}",
            6 * 8,
            total
        );
    }

    #[test]
    fn test_piece_count_star_border() {
        // Star border should also have fewer pieces than the full grid.
        let config_json = r#"{"rows":6,"cols":8,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"pc-star","border_shape":"star"}"#;
        let result = generate_grid(config_json);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let total = parsed["piece_breakdown"]["total"].as_u64().unwrap();

        assert!(total > 0, "star border should have some pieces, got 0");
        assert!(
            total < 6 * 8,
            "star border should have fewer pieces than full grid ({}), got {}",
            6 * 8,
            total
        );
    }

    #[test]
    fn test_piece_count_matches_pieces_array_length() {
        // The piece_breakdown.total should match the actual pieces array length
        // for both rectangular and boundary puzzles.
        let configs = [
            r#"{"rows":4,"cols":5,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"pc-match"}"#,
            r#"{"rows":6,"cols":8,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"pc-match","border_shape":"heart"}"#,
            r#"{"rows":6,"cols":8,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"seed":"pc-match","border_shape":"star"}"#,
        ];

        for config in &configs {
            let result = generate_grid(config);
            assert!(!result.contains(r#""error""#), "error for config: {}", config);
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            let total = parsed["piece_breakdown"]["total"].as_u64().unwrap();
            let pieces_len = parsed["pieces"].as_array().unwrap().len() as u64;
            assert_eq!(
                total, pieces_len,
                "piece_breakdown.total should match pieces array length for config: {}",
                config
            );
        }
    }

}
