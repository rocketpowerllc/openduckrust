//! ONNX policy inference — loads a trained neural network and runs forward passes.
//!
//! Replaces `onnx_infer.py`. Uses the `ort` crate (ONNX Runtime bindings for Rust).

use anyhow::{Context, Result};
use ndarray::Array2;
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;

/// ONNX policy wrapper for running the trained walking policy.
pub struct PolicyInference {
    session: Session,
    input_name: String,
}

impl PolicyInference {
    /// Load an ONNX model from disk.
    pub fn load(model_path: &Path) -> Result<Self> {
        let session = Session::builder()
            .context("Failed to create ONNX session builder")?
            .commit_from_file(model_path)
            .context("Failed to load ONNX model")?;

        let input_name = session.inputs()[0].name().to_string();

        tracing::info!(
            "Loaded ONNX policy from {} (input: {})",
            model_path.display(),
            input_name
        );

        Ok(Self {
            session,
            input_name,
        })
    }

    /// Run a forward pass: observation vector in, action vector out.
    ///
    /// The observation is a 1-D float32 array. The output is a 1-D action vector
    /// (typically 14 DOF for Open Duck Mini).
    pub fn infer(&mut self, observation: &[f64]) -> Result<Vec<f64>> {
        // Convert to f32 and reshape to [1, obs_dim]
        let obs_f32: Vec<f32> = observation.iter().map(|&x| x as f32).collect();
        let obs_len = obs_f32.len();
        let input = Array2::from_shape_vec((1, obs_len), obs_f32)
            .context("Failed to create observation array")?;

        let input_tensor =
            Tensor::from_array(input).context("Failed to create input tensor")?;

        let outputs = self
            .session
            .run(ort::inputs![&self.input_name => input_tensor])
            .context("ONNX inference failed")?;

        // Extract the first output tensor data
        let (_, output_data) = outputs[0]
            .try_extract_tensor::<f32>()
            .context("Failed to extract output tensor")?;

        let action: Vec<f64> = output_data.iter().map(|&x| x as f64).collect();

        Ok(action)
    }

    /// Benchmark inference latency (useful for verifying real-time performance).
    pub fn benchmark(&mut self, obs_dim: usize, iterations: usize) -> Result<std::time::Duration> {
        let dummy_obs: Vec<f64> = vec![0.0; obs_dim];
        let start = std::time::Instant::now();

        for _ in 0..iterations {
            self.infer(&dummy_obs)?;
        }

        let elapsed = start.elapsed();
        let avg = elapsed / iterations as u32;
        tracing::info!(
            "Inference benchmark: {} iterations, avg {:.2}ms ({:.0} Hz)",
            iterations,
            avg.as_secs_f64() * 1000.0,
            1.0 / avg.as_secs_f64()
        );

        Ok(avg)
    }
}
