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
    /// Tab size as a fraction of edge length (0.15..=0.45, default 0.25).
    pub size_pct: f64,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self { size_pct: 0.25 }
    }
}

impl TabConfig {
    /// Validate that tab size is within acceptable bounds.
    pub fn validate(&self) -> Result<(), String> {
        if self.size_pct < 0.15 || self.size_pct > 0.45 {
            return Err(format!(
                "tab size_pct must be between 0.15 and 0.45, got {}",
                self.size_pct
            ));
        }
        Ok(())
    }
}

/// Jitter/randomness configuration for edge variation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitterConfig {
    /// Jitter amount as a fraction (0.0..=1.0, default 0.5).
    /// 0.0 = all connectors identical, 1.0 = maximum variation.
    pub amount: f64,
}

impl Default for JitterConfig {
    fn default() -> Self {
        Self { amount: 0.5 }
    }
}

impl JitterConfig {
    /// Validate that jitter amount is within acceptable bounds.
    pub fn validate(&self) -> Result<(), String> {
        if self.amount < 0.0 || self.amount > 1.0 {
            return Err(format!(
                "jitter amount must be between 0.0 and 1.0, got {}",
                self.amount
            ));
        }
        Ok(())
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
    /// Jitter/randomness configuration.
    pub jitter: JitterConfig,
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
            jitter: JitterConfig::default(),
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
        self.jitter.validate()?;
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
        jitter: JitterConfig,
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
            jitter,
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
        config.tab.size_pct = 0.50;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_jitter_negative() {
        let mut config = PuzzleConfig::default();
        config.jitter.amount = -0.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_jitter_over_one() {
        let mut config = PuzzleConfig::default();
        config.jitter.amount = 1.1;
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
            JitterConfig::default(),
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
            JitterConfig::default(),
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
            JitterConfig::default(),
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
            TabConfig { size_pct: 0.15 },
            JitterConfig { amount: 0.0 },
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
            TabConfig { size_pct: 0.45 },
            JitterConfig { amount: 1.0 },
            BorderConfig {
                corner_radius: 10.0,
            },
            String::new(),
        );
        assert!(config.is_ok());
    }
}
