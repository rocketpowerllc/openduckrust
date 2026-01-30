//! Duck configuration loader — reads duck_config.json for per-robot tuning.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Top-level duck configuration, loaded from JSON.
#[derive(Debug, Clone, Deserialize)]
pub struct DuckConfig {
    #[serde(default)]
    pub start_paused: bool,

    #[serde(default)]
    pub imu_upside_down: bool,

    #[serde(default)]
    pub phase_frequency_factor_offset: f64,

    #[serde(default)]
    pub expression_features: ExpressionFeatures,

    #[serde(default = "default_joints_offsets", rename = "joints_offsets")]
    pub joints_offset: HashMap<String, f64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExpressionFeatures {
    #[serde(default)]
    pub eyes: bool,
    #[serde(default)]
    pub projector: bool,
    #[serde(default)]
    pub antennas: bool,
    #[serde(default)]
    pub speaker: bool,
    #[serde(default)]
    pub microphone: bool,
    #[serde(default)]
    pub camera: bool,
}

fn default_joints_offsets() -> HashMap<String, f64> {
    [
        ("left_hip_yaw", 0.0),
        ("left_hip_roll", 0.0),
        ("left_hip_pitch", 0.0),
        ("left_knee", 0.0),
        ("left_ankle", 0.0),
        ("neck_pitch", 0.0),
        ("head_pitch", 0.0),
        ("head_yaw", 0.0),
        ("head_roll", 0.0),
        ("right_hip_yaw", 0.0),
        ("right_hip_roll", 0.0),
        ("right_hip_pitch", 0.0),
        ("right_knee", 0.0),
        ("right_ankle", 0.0),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

impl DuckConfig {
    /// Load configuration from a JSON file. Falls back to defaults if the file is missing.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            tracing::warn!(
                "Config file not found at {}, using defaults",
                path.display()
            );
            return Ok(Self::default());
        }

        let contents =
            std::fs::read_to_string(path).context("Failed to read duck config file")?;

        let config: DuckConfig =
            serde_json::from_str(&contents).context("Failed to parse duck config JSON")?;

        Ok(config)
    }

    /// Get joint offset by name, defaulting to 0.0.
    pub fn joint_offset(&self, name: &str) -> f64 {
        self.joints_offset.get(name).copied().unwrap_or(0.0)
    }
}

impl Default for DuckConfig {
    fn default() -> Self {
        Self {
            start_paused: false,
            imu_upside_down: false,
            phase_frequency_factor_offset: 0.0,
            expression_features: ExpressionFeatures::default(),
            joints_offset: default_joints_offsets(),
        }
    }
}
