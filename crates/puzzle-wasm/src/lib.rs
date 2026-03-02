use wasm_bindgen::prelude::*;

use puzzle_core::{compute_piece_breakdown, GridConfig};

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
