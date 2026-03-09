use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Unit system for puzzle dimensions.
///
/// All internal computation uses millimeters. This enum allows
/// accepting user input in either unit and converting at the boundary.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Unit {
    Millimeters,
    Inches,
}

impl Unit {
    /// Convert a value in this unit to millimeters.
    pub fn to_mm(&self, value: f64) -> f64 {
        match self {
            Unit::Millimeters => value,
            Unit::Inches => value * 25.4,
        }
    }

    /// Convert a value in millimeters to this unit.
    pub fn from_mm(&self, value_mm: f64) -> f64 {
        match self {
            Unit::Millimeters => value_mm,
            Unit::Inches => value_mm / 25.4,
        }
    }
}

/// Tab/knob configuration for connector generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabConfig {
    /// Tab size as a fraction of edge length (0.15..=0.25, default 0.25).
    /// The effective maximum is dynamically clamped based on grid dimensions
    /// to prevent opposing tabs from overlapping.
    pub size_pct: f64,
    /// Taper amount controlling the neck-to-body ratio (0.57..=1.32, default 0.57).
    /// 0.57 = mild taper, 0.95 = moderate snap-fit, 1.32 = aggressive taper
    /// (narrow neck, wide body). Note: the UI presents this as a normalized 0-1
    /// range; the WASM layer maps user 0→internal 0.57, user 1→internal 1.32.
    #[serde(default = "default_taper")]
    pub taper: f64,
    /// Optional max for per-edge randomization. When Some, each edge gets
    /// a random size_pct in [size_pct, size_pct_max] range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_pct_max: Option<f64>,
    /// Optional max for per-edge taper randomization. When Some, each edge gets
    /// a random taper in [taper, taper_max] range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taper_max: Option<f64>,
}

fn default_taper() -> f64 {
    0.57
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            size_pct: 0.25,
            taper: 0.57,
            size_pct_max: None,
            taper_max: None,
        }
    }
}

impl TabConfig {
    /// Validate that tab parameters are within acceptable bounds.
    pub fn validate(&self) -> Result<(), String> {
        if self.size_pct < 0.15 || self.size_pct > 0.25 {
            return Err(format!(
                "tab size_pct must be between 0.15 and 0.25, got {}",
                self.size_pct
            ));
        }
        if self.taper < 0.57 || self.taper > 1.32 {
            return Err(format!(
                "tab taper must be between 0.57 and 1.32, got {}",
                self.taper
            ));
        }
        if let Some(max) = self.size_pct_max {
            if max < 0.15 || max > 0.25 {
                return Err(format!(
                    "tab size_pct_max must be between 0.15 and 0.25, got {}",
                    max
                ));
            }
            if max < self.size_pct {
                return Err(format!(
                    "tab size_pct_max ({}) must be >= size_pct ({})",
                    max, self.size_pct
                ));
            }
        }
        if let Some(max) = self.taper_max {
            if max < 0.57 || max > 1.32 {
                return Err(format!(
                    "tab taper_max must be between 0.57 and 1.32, got {}",
                    max
                ));
            }
            if max < self.taper {
                return Err(format!(
                    "tab taper_max ({}) must be >= taper ({})",
                    max, self.taper
                ));
            }
        }
        Ok(())
    }

    /// Compute the neck-to-body width ratio from the taper value.
    /// taper=0.57 → ratio=0.715 (mild), taper=0.95 → ratio=0.525, taper=1.32 → ratio=0.34 (aggressive).
    pub fn neck_ratio(&self) -> f64 {
        1.0 - self.taper * 0.5
    }

    /// Return the effective tab size for a single edge, optionally randomized.
    ///
    /// When `size_pct_max` is None, returns the fixed `size_pct` clamped to
    /// `safe_max` without consuming any RNG values (backward compatible).
    /// When `size_pct_max` is Some, returns a random value in
    /// [size_pct.min(safe_max), size_pct_max.min(safe_max)].
    pub fn randomize_tab_size(&self, safe_max: f64, rng: &mut ChaCha8Rng) -> f64 {
        match self.size_pct_max {
            None => self.size_pct.min(safe_max),
            Some(max) => {
                let lo = self.size_pct.min(safe_max);
                let hi = max.min(safe_max);
                if (hi - lo).abs() < 1e-10 {
                    lo
                } else {
                    rng.random_range(lo..=hi)
                }
            }
        }
    }

    /// Return the effective neck ratio for a single edge, optionally randomized.
    ///
    /// When `taper_max` is None, returns the fixed neck_ratio without consuming
    /// any RNG values (backward compatible).
    /// When `taper_max` is Some, picks a random taper in [taper, taper_max] and
    /// computes neck_ratio from it.
    pub fn randomize_neck_ratio(&self, rng: &mut ChaCha8Rng) -> f64 {
        match self.taper_max {
            None => self.neck_ratio(),
            Some(max) => {
                let lo = self.taper;
                let hi = max;
                let t = if (hi - lo).abs() < 1e-10 {
                    lo
                } else {
                    rng.random_range(lo..=hi)
                };
                1.0 - t * 0.5
            }
        }
    }
}

/// Border configuration for the puzzle outline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderConfig {
    /// Corner radius in millimeters (0.0..=10.0, default 2.0).
    /// 0.0 = sharp corners, 2.0 = typical laser-cutting-friendly radius.
    pub corner_radius: f64,
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self { corner_radius: 2.0 }
    }
}

impl BorderConfig {
    /// Validate that corner radius is within acceptable bounds.
    pub fn validate(&self) -> Result<(), String> {
        if self.corner_radius < 0.0 || self.corner_radius > 10.0 {
            return Err(format!(
                "border corner_radius must be between 0.0 and 10.0, got {}",
                self.corner_radius
            ));
        }
        Ok(())
    }
}

/// Top-level puzzle configuration.
///
/// All dimensions are stored in millimeters internally.
/// Use [`PuzzleConfig::from_input`] to construct from user-provided
/// values in any supported unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleConfig {
    /// Grid rows (2..=100).
    pub rows: u32,
    /// Grid columns (2..=100).
    pub cols: u32,
    /// Puzzle width in mm (after unit conversion).
    pub width: f64,
    /// Puzzle height in mm (after unit conversion).
    pub height: f64,
    /// Display/input unit.
    pub unit: Unit,
    /// Tab/knob configuration.
    pub tab: TabConfig,
    /// Border configuration.
    pub border: BorderConfig,
    /// User seed string (empty = auto-generate).
    pub seed: String,
}

impl Default for PuzzleConfig {
    fn default() -> Self {
        Self {
            rows: 6,
            cols: 8,
            width: 297.0,
            height: 210.0,
            unit: Unit::Millimeters,
            tab: TabConfig::default(),
            border: BorderConfig::default(),
            seed: String::new(),
        }
    }
}

impl PuzzleConfig {
    /// Validate all configuration bounds.
    ///
    /// Returns the first error found.
    pub fn validate(&self) -> Result<(), String> {
        if self.rows < 2 || self.rows > 100 {
            return Err(format!("rows must be between 2 and 100, got {}", self.rows));
        }
        if self.cols < 2 || self.cols > 100 {
            return Err(format!("cols must be between 2 and 100, got {}", self.cols));
        }
        if self.width <= 0.0 {
            return Err(format!("width must be positive, got {}", self.width));
        }
        if self.height <= 0.0 {
            return Err(format!("height must be positive, got {}", self.height));
        }
        self.tab.validate()?;
        self.border.validate()?;
        Ok(())
    }

    /// Construct a PuzzleConfig from user input, converting units to mm.
    ///
    /// When `unit` is `Inches`, width and height are multiplied by 25.4
    /// before storing. All other parameters are unit-independent.
    pub fn from_input(
        rows: u32,
        cols: u32,
        width: f64,
        height: f64,
        unit: Unit,
        tab: TabConfig,
        border: BorderConfig,
        seed: String,
    ) -> Result<Self, String> {
        let config = Self {
            rows,
            cols,
            width: unit.to_mm(width),
            height: unit.to_mm(height),
            unit,
            tab,
            border,
            seed,
        };
        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_mm_identity() {
        assert_eq!(Unit::Millimeters.to_mm(100.0), 100.0);
        assert_eq!(Unit::Millimeters.from_mm(100.0), 100.0);
    }

    #[test]
    fn test_unit_inches_to_mm() {
        let result = Unit::Inches.to_mm(1.0);
        assert!((result - 25.4).abs() < 1e-10);
    }

    #[test]
    fn test_unit_mm_to_inches() {
        let result = Unit::Inches.from_mm(25.4);
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_unit_roundtrip() {
        let original = 12.5;
        let mm = Unit::Inches.to_mm(original);
        let back = Unit::Inches.from_mm(mm);
        assert!((back - original).abs() < 1e-10);
    }

    #[test]
    fn test_default_config_is_valid() {
        let config = PuzzleConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.rows, 6);
        assert_eq!(config.cols, 8);
        assert!((config.width - 297.0).abs() < 1e-10);
        assert!((config.height - 210.0).abs() < 1e-10);
        assert_eq!(config.unit, Unit::Millimeters);
        assert!(config.seed.is_empty());
    }

    #[test]
    fn test_validate_rows_too_low() {
        let mut config = PuzzleConfig::default();
        config.rows = 1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_rows_too_high() {
        let mut config = PuzzleConfig::default();
        config.rows = 101;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_cols_too_low() {
        let mut config = PuzzleConfig::default();
        config.cols = 1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_cols_too_high() {
        let mut config = PuzzleConfig::default();
        config.cols = 101;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_negative_width() {
        let mut config = PuzzleConfig::default();
        config.width = -10.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_tab_too_small() {
        let mut config = PuzzleConfig::default();
        config.tab.size_pct = 0.10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_tab_too_large() {
        let mut config = PuzzleConfig::default();
        config.tab.size_pct = 0.30;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_border_negative() {
        let mut config = PuzzleConfig::default();
        config.border.corner_radius = -1.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_border_too_large() {
        let mut config = PuzzleConfig::default();
        config.border.corner_radius = 11.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_from_input_inches_converts() {
        let config = PuzzleConfig::from_input(
            4,
            6,
            10.0,
            8.0,
            Unit::Inches,
            TabConfig::default(),
            BorderConfig::default(),
            "test".to_string(),
        )
        .unwrap();

        assert!((config.width - 254.0).abs() < 1e-10);
        assert!((config.height - 203.2).abs() < 1e-10);
    }

    #[test]
    fn test_from_input_mm_no_conversion() {
        let config = PuzzleConfig::from_input(
            4,
            6,
            200.0,
            150.0,
            Unit::Millimeters,
            TabConfig::default(),
            BorderConfig::default(),
            String::new(),
        )
        .unwrap();

        assert!((config.width - 200.0).abs() < 1e-10);
        assert!((config.height - 150.0).abs() < 1e-10);
    }

    #[test]
    fn test_from_input_validates() {
        let result = PuzzleConfig::from_input(
            0,
            6,
            200.0,
            150.0,
            Unit::Millimeters,
            TabConfig::default(),
            BorderConfig::default(),
            String::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_boundary_values_valid() {
        // Minimum valid config
        let config = PuzzleConfig::from_input(
            2,
            2,
            1.0,
            1.0,
            Unit::Millimeters,
            TabConfig {
                size_pct: 0.15,
                taper: 0.57,
                size_pct_max: None,
                taper_max: None,
            },
            BorderConfig { corner_radius: 0.0 },
            String::new(),
        );
        assert!(config.is_ok());

        // Maximum valid config
        let config = PuzzleConfig::from_input(
            100,
            100,
            1000.0,
            1000.0,
            Unit::Millimeters,
            TabConfig {
                size_pct: 0.25,
                taper: 1.32,
                size_pct_max: None,
                taper_max: None,
            },
            BorderConfig {
                corner_radius: 10.0,
            },
            String::new(),
        );
        assert!(config.is_ok());

        // Maximum valid config with ranges
        let config = PuzzleConfig::from_input(
            100,
            100,
            1000.0,
            1000.0,
            Unit::Millimeters,
            TabConfig {
                size_pct: 0.15,
                taper: 0.57,
                size_pct_max: Some(0.25),
                taper_max: Some(1.32),
            },
            BorderConfig {
                corner_radius: 10.0,
            },
            String::new(),
        );
        assert!(config.is_ok());
    }

    #[test]
    fn test_randomize_tab_size_none_returns_fixed() {
        use crate::seed::create_rng;
        let tab = TabConfig::default(); // size_pct_max = None
        let mut rng1 = create_rng("test");
        let mut rng2 = create_rng("test");

        let val = tab.randomize_tab_size(0.25, &mut rng1);
        assert!((val - 0.25).abs() < 1e-10, "should return fixed size_pct");

        // RNG should not have been consumed — next random_bool should match fresh rng
        let b1: bool = rng1.random_bool(0.5);
        let b2: bool = rng2.random_bool(0.5);
        assert_eq!(
            b1, b2,
            "RNG should not be consumed when size_pct_max is None"
        );
    }

    #[test]
    fn test_randomize_tab_size_some_produces_range() {
        use crate::seed::create_rng;
        let tab = TabConfig {
            size_pct: 0.15,
            taper: 0.57,
            size_pct_max: Some(0.25),
            taper_max: None,
        };
        let mut rng = create_rng("range-test");
        let mut values = Vec::new();
        for _ in 0..20 {
            let v = tab.randomize_tab_size(0.25, &mut rng);
            assert!(
                v >= 0.15 - 1e-10 && v <= 0.25 + 1e-10,
                "value {} out of range",
                v
            );
            values.push(v);
        }
        // With 20 samples from [0.15, 0.25], we should see at least 2 distinct values
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        values.dedup_by(|a, b| (*a - *b).abs() < 1e-10);
        assert!(
            values.len() >= 2,
            "should produce varied values, got {:?}",
            values
        );
    }

    #[test]
    fn test_randomize_neck_ratio_none_returns_fixed() {
        use crate::seed::create_rng;
        let tab = TabConfig::default(); // taper_max = None
        let mut rng = create_rng("test");
        let val = tab.randomize_neck_ratio(&mut rng);
        assert!(
            (val - tab.neck_ratio()).abs() < 1e-10,
            "should return fixed neck_ratio"
        );
    }

    #[test]
    fn test_randomize_neck_ratio_some_produces_range() {
        use crate::seed::create_rng;
        let tab = TabConfig {
            size_pct: 0.25,
            taper: 0.57,
            size_pct_max: None,
            taper_max: Some(1.32),
        };
        let mut rng = create_rng("neck-range-test");
        let mut values = Vec::new();
        for _ in 0..20 {
            let v = tab.randomize_neck_ratio(&mut rng);
            // taper in [0.57, 1.32] → neck_ratio in [0.34, 0.715]
            assert!(
                v >= 0.34 - 1e-10 && v <= 0.715 + 1e-10,
                "neck_ratio {} out of range",
                v
            );
            values.push(v);
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        values.dedup_by(|a, b| (*a - *b).abs() < 1e-10);
        assert!(
            values.len() >= 2,
            "should produce varied neck ratios, got {:?}",
            values
        );
    }

    #[test]
    fn test_validate_size_pct_max_out_of_range() {
        let tab = TabConfig {
            size_pct: 0.20,
            taper: 0.57,
            size_pct_max: Some(0.30), // too large
            taper_max: None,
        };
        assert!(tab.validate().is_err());
    }

    #[test]
    fn test_validate_size_pct_max_less_than_min() {
        let tab = TabConfig {
            size_pct: 0.20,
            taper: 0.57,
            size_pct_max: Some(0.15), // less than size_pct
            taper_max: None,
        };
        assert!(tab.validate().is_err());
    }

    #[test]
    fn test_validate_taper_max_out_of_range() {
        let tab = TabConfig {
            size_pct: 0.20,
            taper: 0.57,
            size_pct_max: None,
            taper_max: Some(1.50), // too large
        };
        assert!(tab.validate().is_err());
    }

    #[test]
    fn test_validate_taper_max_less_than_min() {
        let tab = TabConfig {
            size_pct: 0.20,
            taper: 0.80,
            size_pct_max: None,
            taper_max: Some(0.60), // less than taper
        };
        assert!(tab.validate().is_err());
    }
}
