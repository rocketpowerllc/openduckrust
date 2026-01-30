//! OpenDuckRust Runtime — zero-overhead bipedal robot control loop.
//!
//! This is the Rust port of the Open Duck Mini Python runtime. It reads
//! sensors (IMU, foot contacts), runs ONNX policy inference, and drives
//! Feetech bus servos at a rock-solid control frequency on a Raspberry Pi.
//!
//! Usage:
//!   openduckrust-runtime --onnx-model-path policy.onnx [OPTIONS]

mod config;
mod controller;
mod imu;
mod inference;
mod motors;
mod peripherals;
mod reference_motion;
mod rl_utils;
mod sounds;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use config::DuckConfig;
use controller::XBoxController;
use inference::PolicyInference;
use motors::{make_action_dict, MotorController, NUM_DOFS};
use reference_motion::PhaseTracker;
use rl_utils::LowPassActionFilter;
use sounds::Sounds;

// Hardware types: real on Linux, mocks elsewhere
use imu::ImuReader;
#[cfg(target_os = "linux")]
use imu::Imu;
#[cfg(not(target_os = "linux"))]
use imu::MockImu;

#[cfg(target_os = "linux")]
use peripherals::{Antennas, Eyes, FeetContacts, Projector};
#[cfg(not(target_os = "linux"))]
use peripherals::MockFeetContacts;

/// OpenDuckRust: high-performance bipedal robot runtime.
#[derive(Parser, Debug)]
#[command(name = "openduckrust-runtime")]
#[command(about = "Rust runtime for the Open Duck Mini bipedal robot")]
struct Args {
    /// Path to the trained ONNX policy model.
    #[arg(long)]
    onnx_model_path: PathBuf,

    /// Path to the duck configuration JSON file.
    #[arg(long, default_value = "~/duck_config.json")]
    duck_config_path: PathBuf,

    /// Serial port for the Feetech servo bus.
    #[arg(long, default_value = "/dev/ttyACM0")]
    serial_port: String,

    /// Control loop frequency in Hz.
    #[arg(short = 'c', long, default_value_t = 50)]
    control_freq: u32,

    /// Action scale factor applied to policy output.
    #[arg(short = 'a', long, default_value_t = 0.25)]
    action_scale: f64,

    /// PID proportional gain.
    #[arg(short = 'p', default_value_t = 30)]
    kp: u32,

    /// PID integral gain.
    #[arg(short = 'i', default_value_t = 0)]
    ki: u32,

    /// PID derivative gain.
    #[arg(short = 'd', default_value_t = 0)]
    kd: u32,

    /// IMU pitch bias in degrees.
    #[arg(long, default_value_t = 0.0)]
    pitch_bias: f64,

    /// Enable gamepad (Xbox controller) commands.
    #[arg(long, default_value_t = true)]
    commands: bool,

    /// Low-pass filter cutoff frequency (Hz). Disabled if not set.
    #[arg(long)]
    cutoff_frequency: Option<f64>,

    /// Path to polynomial coefficients file for reference motion.
    #[arg(long, default_value = "./polynomial_coefficients.pkl")]
    poly_coefficients: PathBuf,
}

fn main() -> Result<()> {
    // Initialize structured JSON logging
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    // Expand ~ in config path
    let config_path = expand_home(&args.duck_config_path);

    tracing::info!("OpenDuckRust Runtime starting");
    tracing::info!("ONNX model: {}", args.onnx_model_path.display());
    tracing::info!("Config: {}", config_path.display());
    tracing::info!("Control frequency: {} Hz", args.control_freq);

    // Load configuration
    let duck_config = DuckConfig::load(&config_path).context("Failed to load duck config")?;

    // Load ONNX policy
    let mut policy =
        PolicyInference::load(&args.onnx_model_path).context("Failed to load ONNX policy")?;

    // Initialize motor controller
    let mut hwi = MotorController::new(&duck_config, &args.serial_port)
        .context("Failed to initialize motor controller")?;

    // Set PID gains
    let mut kps = vec![args.kp as f64; NUM_DOFS];
    let kds = vec![args.kd as f64; NUM_DOFS];
    // Lower head KPs for compliance
    kps[5] = 8.0;
    kps[6] = 8.0;
    kps[7] = 8.0;
    kps[8] = 8.0;

    hwi.set_kps(&kps)?;
    hwi.set_kds(&kds)?;

    // Turn on motors (gentle startup sequence)
    hwi.turn_on()?;

    // Initialize IMU (real hardware on Linux, mock elsewhere)
    #[cfg(target_os = "linux")]
    let imu_sensor = Imu::new(args.control_freq, duck_config.imu_upside_down)
        .context("Failed to initialize IMU")?;
    #[cfg(not(target_os = "linux"))]
    let imu_sensor = MockImu::new();

    // Initialize feet contacts
    #[cfg(target_os = "linux")]
    let feet_contacts = FeetContacts::new().context("Failed to initialize feet contacts")?;
    #[cfg(not(target_os = "linux"))]
    let feet_contacts = MockFeetContacts;

    // Initialize phase tracker
    let nb_steps = reference_motion::load_period_from_pickle(&args.poly_coefficients)
        .unwrap_or(25);
    let mut phase_tracker =
        PhaseTracker::new(nb_steps, duck_config.phase_frequency_factor_offset);

    // Optional low-pass filter
    let mut action_filter = args
        .cutoff_frequency
        .map(|cutoff| LowPassActionFilter::new(args.control_freq as f64, cutoff));

    // Optional gamepad
    let mut xbox_controller = if args.commands {
        Some(XBoxController::new(20))
    } else {
        None
    };

    // Optional expression features (Linux-only hardware)
    #[cfg(target_os = "linux")]
    let mut eyes = if duck_config.expression_features.eyes {
        Eyes::new().ok()
    } else {
        None
    };

    #[cfg(target_os = "linux")]
    let mut projector = if duck_config.expression_features.projector {
        Projector::new().ok()
    } else {
        None
    };

    #[cfg(target_os = "linux")]
    let mut antennas = if duck_config.expression_features.antennas {
        Antennas::new().ok()
    } else {
        None
    };

    let sound_player = if duck_config.expression_features.speaker {
        Sounds::new(1.0, std::path::Path::new("./assets")).ok()
    } else {
        None
    };

    // ── State vectors ──

    let init_pos = hwi.init_positions_array();
    let joint_names = hwi.joint_names().to_vec();

    let mut last_action = vec![0.0; NUM_DOFS];
    let mut last_last_action = vec![0.0; NUM_DOFS];
    let mut last_last_last_action = vec![0.0; NUM_DOFS];
    let mut motor_targets = init_pos.clone();
    let mut last_commands = [0.0f64; 7];
    let mut paused = duck_config.start_paused;

    let control_period = Duration::from_secs_f64(1.0 / args.control_freq as f64);
    let start_time = Instant::now();

    tracing::info!("Entering control loop at {} Hz", args.control_freq);

    // ── Main control loop ──

    loop {
        let tick_start = Instant::now();

        // ── Gamepad input ──
        if let Some(ref mut controller) = xbox_controller {
            let output = controller.get_last_command();
            last_commands = output.commands;

            // Button handling
            if output.buttons.a.triggered {
                paused = !paused;
                if paused {
                    tracing::info!("PAUSED");
                } else {
                    tracing::info!("UNPAUSED");
                }
            }

            if output.buttons.dpad_up.triggered {
                phase_tracker.adjust_offset(0.05);
            }

            if output.buttons.dpad_down.triggered {
                phase_tracker.adjust_offset(-0.05);
            }

            if output.buttons.lb.is_pressed {
                phase_tracker.set_sprint(true);
            } else {
                phase_tracker.set_sprint(false);
            }

            #[cfg(target_os = "linux")]
            if output.buttons.x.triggered {
                if let Some(ref mut proj) = projector {
                    proj.switch();
                }
            }

            if output.buttons.b.triggered {
                if let Some(ref snd) = sound_player {
                    let _ = snd.play_random();
                }
            }

            #[cfg(target_os = "linux")]
            if let Some(ref mut ant) = antennas {
                ant.set_position_left(output.right_trigger);
                ant.set_position_right(output.left_trigger);
            }
        }

        // Skip control when paused
        if paused {
            std::thread::sleep(Duration::from_millis(100));
            continue;
        }

        // ── Read sensors ──

        let imu_data = imu_sensor.get_data();

        let dof_pos = match hwi.get_present_positions() {
            Some(pos) if pos.len() == NUM_DOFS => pos,
            _ => continue, // skip this tick on read failure
        };

        let dof_vel = match hwi.get_present_velocities() {
            Some(vel) if vel.len() == NUM_DOFS => vel,
            _ => continue,
        };

        let feet = feet_contacts.get();

        // ── Advance gait phase ──

        let imitation_phase = phase_tracker.step();

        // ── Build observation vector ──
        // Layout: [gyro(3), accel(3), commands(7), dof_pos-init(14), dof_vel*0.05(14),
        //          last_action(14), last_last_action(14), last_last_last_action(14),
        //          motor_targets(14), feet_contacts(2), phase(2)]
        // Total: 3+3+7+14+14+14+14+14+14+2+2 = 101
        // Note: the actual dimension depends on the trained model.

        let mut obs = Vec::with_capacity(128);

        // IMU data
        obs.extend_from_slice(&imu_data.gyro);
        obs.extend_from_slice(&imu_data.accel);

        // Commands
        obs.extend_from_slice(&last_commands);

        // Joint positions relative to init
        for i in 0..NUM_DOFS {
            obs.push(dof_pos[i] - init_pos[i]);
        }

        // Joint velocities (scaled)
        for i in 0..NUM_DOFS {
            obs.push(dof_vel[i] * 0.05);
        }

        // Action history
        obs.extend_from_slice(&last_action);
        obs.extend_from_slice(&last_last_action);
        obs.extend_from_slice(&last_last_last_action);

        // Motor targets
        obs.extend_from_slice(&motor_targets);

        // Feet contacts
        obs.extend_from_slice(&feet);

        // Gait phase
        obs.extend_from_slice(&imitation_phase);

        // ── Policy inference ──

        let action = match policy.infer(&obs) {
            Ok(a) => a,
            Err(e) => {
                tracing::error!("Inference failed: {}", e);
                continue;
            }
        };

        // ── Update action history ──

        last_last_last_action = last_last_action.clone();
        last_last_action = last_action.clone();
        last_action = action.clone();

        // ── Compute motor targets ──

        motor_targets = init_pos
            .iter()
            .zip(action.iter())
            .map(|(&init, &act)| init + act * args.action_scale)
            .collect();

        // Optional low-pass filter
        if let Some(ref mut filter) = action_filter {
            filter.push(&motor_targets);
            if start_time.elapsed() > Duration::from_secs(1) {
                motor_targets = filter.get_filtered_action();
            }
        }

        // ── Apply head commands from gamepad ──

        if motor_targets.len() > 8 {
            motor_targets[5] = last_commands[3] + motor_targets[5];
            motor_targets[6] = last_commands[4] + motor_targets[6];
            motor_targets[7] = last_commands[5] + motor_targets[7];
            motor_targets[8] = last_commands[6] + motor_targets[8];
        }

        // ── Send to motors ──

        let action_dict = make_action_dict(&motor_targets, &joint_names);
        if let Err(e) = hwi.set_position_all(&action_dict) {
            tracing::warn!("Motor write failed: {}", e);
        }

        // ── Timing ──

        let took = tick_start.elapsed();
        if took > control_period {
            let overshoot = took - control_period;
            tracing::warn!(
                "Control budget exceeded by {:.1}ms",
                overshoot.as_secs_f64() * 1000.0
            );
        } else {
            // High-precision sleep (avoids OS scheduler jitter)
            spin_sleep::sleep(control_period - took);
        }
    }
}

/// Expand `~` at the start of a path to the user's home directory.
fn expand_home(path: &PathBuf) -> PathBuf {
    if let Some(s) = path.to_str() {
        if s.starts_with("~/") {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(format!("{}{}", home, &s[1..]));
            }
        }
    }
    path.clone()
}
