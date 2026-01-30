//! Reinforcement learning utilities — joint reordering and action filters.
//!
//! Replaces `rl_utils.py`.

/// Mujoco joint ordering (matches the ONNX model output).
pub const MUJOCO_JOINTS_ORDER: &[&str] = &[
    "left_hip_yaw",
    "left_hip_roll",
    "left_hip_pitch",
    "left_knee",
    "left_ankle",
    "neck_pitch",
    "head_pitch",
    "head_yaw",
    "head_roll",
    "left_antenna",
    "right_antenna",
    "right_hip_yaw",
    "right_hip_roll",
    "right_hip_pitch",
    "right_knee",
    "right_ankle",
];

/// Convert action-scale offsets to absolute PD targets.
#[inline]
pub fn action_to_pd_targets(action: &[f64], offset: &[f64], scale: f64) -> Vec<f64> {
    action
        .iter()
        .zip(offset.iter())
        .map(|(&a, &o)| o + scale * a)
        .collect()
}

/// Rotate a vector by the inverse of a quaternion [x, y, z, w].
pub fn quat_rotate_inverse(q: &[f64; 4], v: &[f64; 3]) -> [f64; 3] {
    let q_w = q[3];
    let q_vec = [q[0], q[1], q[2]];

    let a = [
        v[0] * (2.0 * q_w * q_w - 1.0),
        v[1] * (2.0 * q_w * q_w - 1.0),
        v[2] * (2.0 * q_w * q_w - 1.0),
    ];

    let cross = cross_product(&q_vec, v);
    let b = [cross[0] * q_w * 2.0, cross[1] * q_w * 2.0, cross[2] * q_w * 2.0];

    let dot = dot_product(&q_vec, v);
    let c = [q_vec[0] * dot * 2.0, q_vec[1] * dot * 2.0, q_vec[2] * dot * 2.0];

    [a[0] - b[0] + c[0], a[1] - b[1] + c[1], a[2] - b[2] + c[2]]
}

#[inline]
fn cross_product(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn dot_product(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Simple moving-average action filter.
pub struct ActionFilter {
    window_size: usize,
    buffer: Vec<Vec<f64>>,
}

impl ActionFilter {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            buffer: Vec::with_capacity(window_size),
        }
    }

    pub fn push(&mut self, action: &[f64]) {
        self.buffer.push(action.to_vec());
        if self.buffer.len() > self.window_size {
            self.buffer.remove(0);
        }
    }

    pub fn get_filtered_action(&self) -> Vec<f64> {
        if self.buffer.is_empty() {
            return Vec::new();
        }
        let n = self.buffer[0].len();
        let count = self.buffer.len() as f64;
        (0..n)
            .map(|i| self.buffer.iter().map(|a| a[i]).sum::<f64>() / count)
            .collect()
    }
}

/// First-order IIR low-pass action filter.
/// Eliminates high-frequency jitter from the policy output.
pub struct LowPassActionFilter {
    alpha: f64,
    last_action: Vec<f64>,
    current_action: Vec<f64>,
    initialized: bool,
}

impl LowPassActionFilter {
    pub fn new(control_freq: f64, cutoff_frequency: f64) -> Self {
        let alpha = (1.0 / cutoff_frequency) / (1.0 / control_freq + 1.0 / cutoff_frequency);
        Self {
            alpha,
            last_action: Vec::new(),
            current_action: Vec::new(),
            initialized: false,
        }
    }

    pub fn push(&mut self, action: &[f64]) {
        if !self.initialized {
            self.last_action = action.to_vec();
            self.initialized = true;
        }
        self.current_action = action.to_vec();
    }

    pub fn get_filtered_action(&mut self) -> Vec<f64> {
        self.last_action = self
            .last_action
            .iter()
            .zip(self.current_action.iter())
            .map(|(&last, &current)| self.alpha * last + (1.0 - self.alpha) * current)
            .collect();
        self.last_action.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_pass_filter_converges() {
        let mut filter = LowPassActionFilter::new(50.0, 30.0);
        let target = vec![1.0, 2.0, 3.0];

        // Push the same value many times — should converge
        for _ in 0..200 {
            filter.push(&target);
            filter.get_filtered_action();
        }

        let result = filter.get_filtered_action();
        for (r, t) in result.iter().zip(target.iter()) {
            assert!((r - t).abs() < 0.01, "Filter did not converge: {} vs {}", r, t);
        }
    }

    #[test]
    fn test_action_filter_average() {
        let mut filter = ActionFilter::new(3);
        filter.push(&[1.0, 2.0]);
        filter.push(&[4.0, 5.0]);
        filter.push(&[7.0, 8.0]);

        let avg = filter.get_filtered_action();
        assert!((avg[0] - 4.0).abs() < 1e-10);
        assert!((avg[1] - 5.0).abs() < 1e-10);
    }
}
