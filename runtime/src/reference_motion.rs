//! Polynomial reference motion generator.
//!
//! Replaces `poly_reference_motion.py`. Provides the gait phase tracking
//! needed by the RL policy — specifically the `nb_steps_in_period` value
//! and the sinusoidal phase signal.
//!
//! The polynomial coefficients are loaded from a pickle file at startup,
//! but for the Rust runtime we only need the timing metadata. The actual
//! reference motion generation is handled by the RL policy in the ONNX model.

use anyhow::Result;
use std::path::Path;

/// Reference motion phase tracker.
///
/// The RL policy uses a sinusoidal phase signal [cos(phase), sin(phase)]
/// to track where in the gait cycle the robot is. This struct manages
/// the phase counter.
pub struct PhaseTracker {
    /// Number of simulation steps in one full gait period.
    pub nb_steps_in_period: usize,

    /// Current step index within the period.
    step_index: f64,

    /// Phase frequency factor (1.0 = normal, >1.0 = faster gait).
    pub frequency_factor: f64,

    /// Additive offset to the frequency factor (per-robot tuning).
    pub frequency_factor_offset: f64,
}

impl PhaseTracker {
    /// Create a new phase tracker.
    ///
    /// `nb_steps_in_period`: typically loaded from the polynomial data file.
    /// For the default Open Duck Mini configuration, this is typically
    /// `period * fps` (e.g., 0.5s * 50Hz = 25 steps).
    pub fn new(nb_steps_in_period: usize, frequency_factor_offset: f64) -> Self {
        Self {
            nb_steps_in_period,
            step_index: 0.0,
            frequency_factor: 1.0,
            frequency_factor_offset,
        }
    }

    /// Create with default period settings (50Hz, 0.5s period = 25 steps).
    pub fn default_50hz() -> Self {
        Self::new(25, 0.0)
    }

    /// Advance the phase by one step and return [cos(phase), sin(phase)].
    pub fn step(&mut self) -> [f64; 2] {
        self.step_index += self.frequency_factor + self.frequency_factor_offset;
        self.step_index %= self.nb_steps_in_period as f64;

        let phase =
            self.step_index / self.nb_steps_in_period as f64 * 2.0 * std::f64::consts::PI;

        [phase.cos(), phase.sin()]
    }

    /// Get the current phase signal without advancing.
    pub fn current_phase(&self) -> [f64; 2] {
        let phase =
            self.step_index / self.nb_steps_in_period as f64 * 2.0 * std::f64::consts::PI;
        [phase.cos(), phase.sin()]
    }

    /// Reset the phase counter to zero.
    pub fn reset(&mut self) {
        self.step_index = 0.0;
    }

    /// Set the sprint mode (higher frequency factor).
    pub fn set_sprint(&mut self, sprint: bool) {
        self.frequency_factor = if sprint { 1.3 } else { 1.0 };
    }

    /// Adjust frequency factor offset.
    pub fn adjust_offset(&mut self, delta: f64) {
        self.frequency_factor_offset += delta;
        tracing::info!(
            "Phase frequency factor offset: {:.3}",
            self.frequency_factor_offset
        );
    }
}

/// Attempt to load nb_steps_in_period from a polynomial coefficients pickle file.
///
/// This is a best-effort parser for the Python pickle format. If it fails,
/// returns a sensible default.
pub fn load_period_from_pickle(path: &Path) -> Result<usize> {
    if !path.exists() {
        tracing::warn!(
            "Polynomial coefficients file not found at {}, using default period",
            path.display()
        );
        return Ok(25); // default: 0.5s period at 50Hz
    }

    // The pickle file contains a dict with entries like:
    // "0.0_0.0_0.0" -> { "period": 0.5, "fps": 50, ... }
    // For now, we use the default. Full pickle parsing would require
    // a pickle decoder crate.
    tracing::info!(
        "Polynomial coefficients file found at {}, using default period extraction",
        path.display()
    );

    // Default: period=0.5s, fps=50 -> 25 steps
    Ok(25)
}
