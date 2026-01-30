//! Feetech STS3215 servo motor control over serial (USB).
//!
//! Replaces `rustypot_position_hwi.py`. Implements the Feetech serial protocol
//! for reading positions/velocities and writing goal positions.

use anyhow::{Context, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::Cursor;
use std::time::Duration;

use crate::config::DuckConfig;

// Feetech protocol constants
const HEADER: [u8; 2] = [0xFF, 0xFF];
const INST_WRITE: u8 = 0x03;
const INST_READ: u8 = 0x02;
const INST_SYNC_WRITE: u8 = 0x83;
const INST_SYNC_READ: u8 = 0x82;

// Register addresses for STS3215
const ADDR_TORQUE_ENABLE: u8 = 40;
const ADDR_GOAL_POSITION: u8 = 42;
const ADDR_PRESENT_POSITION: u8 = 56;
const ADDR_PRESENT_SPEED: u8 = 58;
const ADDR_P_GAIN: u8 = 21;
const ADDR_D_GAIN: u8 = 22;

/// Ordered joint definitions matching the Python runtime.
pub const JOINT_NAMES: &[&str] = &[
    "left_hip_yaw",
    "left_hip_roll",
    "left_hip_pitch",
    "left_knee",
    "left_ankle",
    "neck_pitch",
    "head_pitch",
    "head_yaw",
    "head_roll",
    "right_hip_yaw",
    "right_hip_roll",
    "right_hip_pitch",
    "right_knee",
    "right_ankle",
];

/// Servo IDs corresponding to each joint.
pub const JOINT_IDS: &[u8] = &[
    20, 21, 22, 23, 24, // left leg
    30, 31, 32, 33, // head
    10, 11, 12, 13, 14, // right leg
];

pub const NUM_DOFS: usize = 14;

/// Default initial standing pose (radians).
pub fn default_init_positions() -> HashMap<String, f64> {
    [
        ("left_hip_yaw", 0.002),
        ("left_hip_roll", 0.053),
        ("left_hip_pitch", -0.63),
        ("left_knee", 1.368),
        ("left_ankle", -0.784),
        ("neck_pitch", 0.0),
        ("head_pitch", 0.0),
        ("head_yaw", 0.0),
        ("head_roll", 0.0),
        ("right_hip_yaw", -0.003),
        ("right_hip_roll", -0.065),
        ("right_hip_pitch", 0.635),
        ("right_knee", 1.379),
        ("right_ankle", -0.796),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

/// Hardware interface for the Feetech STS3215 bus servos.
pub struct MotorController {
    port: Box<dyn serialport::SerialPort>,
    joint_ids: Vec<u8>,
    joint_names: Vec<String>,
    offsets: HashMap<String, f64>,
    init_pos: HashMap<String, f64>,
    kps: Vec<f64>,
    kds: Vec<f64>,
}

impl MotorController {
    /// Open the serial port and initialize the motor controller.
    pub fn new(config: &DuckConfig, serial_port: &str) -> Result<Self> {
        let port = serialport::new(serial_port, 1_000_000)
            .timeout(Duration::from_millis(10))
            .open()
            .with_context(|| format!("Failed to open serial port {}", serial_port))?;

        let joint_names: Vec<String> = JOINT_NAMES.iter().map(|s| s.to_string()).collect();
        let joint_ids = JOINT_IDS.to_vec();

        Ok(Self {
            port,
            joint_ids,
            joint_names: joint_names.clone(),
            offsets: config.joints_offset.clone(),
            init_pos: default_init_positions(),
            kps: vec![32.0; NUM_DOFS],
            kds: vec![0.0; NUM_DOFS],
        })
    }

    /// Get the initial standing positions as an ordered array.
    pub fn init_positions_array(&self) -> Vec<f64> {
        self.joint_names
            .iter()
            .map(|name| self.init_pos.get(name).copied().unwrap_or(0.0))
            .collect()
    }

    /// Get joint names in order.
    pub fn joint_names(&self) -> &[String] {
        &self.joint_names
    }

    /// Set PID proportional gains for all joints.
    pub fn set_kps(&mut self, kps: &[f64]) -> Result<()> {
        self.kps = kps.to_vec();
        let ids = self.joint_ids.clone();
        for (i, id) in ids.iter().enumerate() {
            let kp_val = kps[i] as u8;
            self.write_register(*id, ADDR_P_GAIN, &[kp_val])?;
        }
        Ok(())
    }

    /// Set PID derivative gains for all joints.
    pub fn set_kds(&mut self, kds: &[f64]) -> Result<()> {
        self.kds = kds.to_vec();
        let ids = self.joint_ids.clone();
        for (i, id) in ids.iter().enumerate() {
            let kd_val = kds[i] as u8;
            self.write_register(*id, ADDR_D_GAIN, &[kd_val])?;
        }
        Ok(())
    }

    /// Enable torque on all servos (with low KP first, then init position).
    pub fn turn_on(&mut self) -> Result<()> {
        // Enable torque
        let ids = self.joint_ids.clone();
        for &id in &ids {
            self.write_register(id, ADDR_TORQUE_ENABLE, &[1])?;
        }

        // Set low KP for gentle startup
        let low_kps = vec![2.0; NUM_DOFS];
        self.set_kps(&low_kps)?;
        tracing::info!("Motors: low KPs set");

        std::thread::sleep(Duration::from_secs(1));

        // Move to init position
        self.set_position_all_array(&self.init_positions_array())?;
        tracing::info!("Motors: init position set");

        std::thread::sleep(Duration::from_secs(1));

        // Restore full KPs
        let full_kps = self.kps.clone();
        self.set_kps(&full_kps)?;
        tracing::info!("Motors: full KPs set");

        Ok(())
    }

    /// Disable torque on all servos.
    pub fn turn_off(&mut self) -> Result<()> {
        let ids = self.joint_ids.clone();
        for &id in &ids {
            self.write_register(id, ADDR_TORQUE_ENABLE, &[0])?;
        }
        tracing::info!("Motors: torque disabled");
        Ok(())
    }

    /// Write goal positions for all joints (radians). Applies per-joint offsets.
    pub fn set_position_all(&mut self, positions: &HashMap<String, f64>) -> Result<()> {
        let mut ids = Vec::with_capacity(NUM_DOFS);
        let mut raw_positions = Vec::with_capacity(NUM_DOFS);

        for (i, name) in self.joint_names.iter().enumerate() {
            if let Some(&pos) = positions.get(name) {
                let offset = self.offsets.get(name).copied().unwrap_or(0.0);
                ids.push(self.joint_ids[i]);
                raw_positions.push(rad_to_raw(pos + offset));
            }
        }

        self.sync_write_positions(&ids, &raw_positions)
    }

    /// Write goal positions from an ordered array (radians). Applies per-joint offsets.
    pub fn set_position_all_array(&mut self, positions: &[f64]) -> Result<()> {
        let mut ids = Vec::with_capacity(NUM_DOFS);
        let mut raw_positions = Vec::with_capacity(NUM_DOFS);

        for (i, &pos) in positions.iter().enumerate() {
            let name = &self.joint_names[i];
            let offset = self.offsets.get(name).copied().unwrap_or(0.0);
            ids.push(self.joint_ids[i]);
            raw_positions.push(rad_to_raw(pos + offset));
        }

        self.sync_write_positions(&ids, &raw_positions)
    }

    /// Read present positions of all joints (radians), minus offsets.
    /// Returns None if communication fails.
    pub fn get_present_positions(&mut self) -> Option<Vec<f64>> {
        match self.sync_read(&self.joint_ids.clone(), ADDR_PRESENT_POSITION, 2) {
            Ok(raw_values) => {
                let positions: Vec<f64> = raw_values
                    .iter()
                    .enumerate()
                    .map(|(i, &raw)| {
                        let name = &self.joint_names[i];
                        let offset = self.offsets.get(name).copied().unwrap_or(0.0);
                        raw_to_rad(raw) - offset
                    })
                    .collect();
                Some(positions)
            }
            Err(e) => {
                tracing::warn!("Failed to read positions: {}", e);
                None
            }
        }
    }

    /// Read present velocities of all joints (rad/s).
    /// Returns None if communication fails.
    pub fn get_present_velocities(&mut self) -> Option<Vec<f64>> {
        match self.sync_read(&self.joint_ids.clone(), ADDR_PRESENT_SPEED, 2) {
            Ok(raw_values) => {
                let velocities: Vec<f64> =
                    raw_values.iter().map(|&raw| raw_to_rad_per_sec(raw)).collect();
                Some(velocities)
            }
            Err(e) => {
                tracing::warn!("Failed to read velocities: {}", e);
                None
            }
        }
    }

    // ── Low-level protocol ──

    fn write_register(&mut self, id: u8, addr: u8, data: &[u8]) -> Result<()> {
        let length = (data.len() + 3) as u8;
        let mut packet = Vec::with_capacity(6 + data.len());
        packet.extend_from_slice(&HEADER);
        packet.push(id);
        packet.push(length);
        packet.push(INST_WRITE);
        packet.push(addr);
        packet.extend_from_slice(data);

        let checksum = compute_checksum(&packet[2..]);
        packet.push(checksum);

        self.port
            .write_all(&packet)
            .context("Serial write failed")?;
        self.port.flush().context("Serial flush failed")?;

        // Drain any response
        self.drain_response();
        Ok(())
    }

    fn sync_write_positions(&mut self, ids: &[u8], values: &[i16]) -> Result<()> {
        let data_len: u8 = 2; // 2 bytes per position
        let param_len = ids.len() * (1 + data_len as usize);
        let length = (param_len + 4) as u8;

        let mut packet = Vec::with_capacity(8 + param_len);
        packet.extend_from_slice(&HEADER);
        packet.push(0xFE); // broadcast ID
        packet.push(length);
        packet.push(INST_SYNC_WRITE);
        packet.push(ADDR_GOAL_POSITION);
        packet.push(data_len);

        for (i, &id) in ids.iter().enumerate() {
            packet.push(id);
            let mut buf = Vec::new();
            buf.write_i16::<LittleEndian>(values[i])
                .context("Failed to encode position")?;
            packet.extend_from_slice(&buf);
        }

        let checksum = compute_checksum(&packet[2..]);
        packet.push(checksum);

        self.port
            .write_all(&packet)
            .context("Serial write failed")?;
        self.port.flush().context("Serial flush failed")?;

        Ok(())
    }

    fn sync_read(&mut self, ids: &[u8], addr: u8, data_len: u8) -> Result<Vec<i16>> {
        // Build sync read packet
        let length = (ids.len() + 4) as u8;
        let mut packet = Vec::with_capacity(8 + ids.len());
        packet.extend_from_slice(&HEADER);
        packet.push(0xFE); // broadcast
        packet.push(length);
        packet.push(INST_SYNC_READ);
        packet.push(addr);
        packet.push(data_len);
        packet.extend_from_slice(ids);

        let checksum = compute_checksum(&packet[2..]);
        packet.push(checksum);

        self.port
            .write_all(&packet)
            .context("Serial write failed")?;
        self.port.flush()?;

        // Read responses: each servo replies with [0xFF, 0xFF, id, len, err, data..., checksum]
        let mut values = Vec::with_capacity(ids.len());
        let response_size = (6 + data_len as usize) * ids.len();
        let mut buf = vec![0u8; response_size];

        // Allow partial reads
        std::thread::sleep(Duration::from_micros(500));
        let bytes_read = self.port.read(&mut buf).unwrap_or(0);

        if bytes_read == 0 {
            anyhow::bail!("No response from servos");
        }

        // Parse individual servo responses
        let mut cursor = 0;
        for _ in 0..ids.len() {
            if cursor + 6 + data_len as usize > bytes_read {
                // Pad with zero if we got a short read
                values.push(0);
                continue;
            }

            // Skip header (0xFF 0xFF), id, length, error
            cursor += 5;

            let mut rdr = Cursor::new(&buf[cursor..cursor + data_len as usize]);
            let val = rdr.read_i16::<LittleEndian>().unwrap_or(0);
            values.push(val);

            cursor += data_len as usize + 1; // data + checksum
        }

        Ok(values)
    }

    fn drain_response(&mut self) {
        let mut buf = [0u8; 256];
        std::thread::sleep(Duration::from_micros(200));
        let _ = self.port.read(&mut buf);
    }
}

/// Compute Feetech checksum: ~(sum of bytes) & 0xFF.
fn compute_checksum(data: &[u8]) -> u8 {
    let sum: u16 = data.iter().map(|&b| b as u16).sum();
    (!sum as u8) & 0xFF
}

/// Convert radians to raw servo position (STS3215: 0-4095, center at 2048).
fn rad_to_raw(rad: f64) -> i16 {
    // STS3215: 0-4095 maps to 0-360 degrees, center at 2048
    let degrees = rad.to_degrees();
    let raw = (degrees / 360.0 * 4096.0 + 2048.0) as i16;
    raw.clamp(0, 4095)
}

/// Convert raw servo position to radians.
fn raw_to_rad(raw: i16) -> f64 {
    ((raw as f64 - 2048.0) / 4096.0 * 360.0).to_radians()
}

/// Convert raw velocity to rad/s.
fn raw_to_rad_per_sec(raw: i16) -> f64 {
    // STS3215 velocity unit: ~0.0116 RPM per step
    let rpm = raw as f64 * 0.0116;
    rpm * std::f64::consts::PI / 30.0
}

/// Build a name->position HashMap from an ordered action array and joint name list.
pub fn make_action_dict(action: &[f64], joint_names: &[String]) -> HashMap<String, f64> {
    let mut dict = HashMap::new();
    for (i, name) in joint_names.iter().enumerate() {
        if !name.contains("antenna") {
            if let Some(&val) = action.get(i) {
                dict.insert(name.clone(), val);
            }
        }
    }
    dict
}
