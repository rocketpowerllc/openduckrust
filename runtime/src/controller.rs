//! Gamepad (Xbox controller) input handling.
//!
//! Replaces `xbox_controller.py`. Uses the `gilrs` crate for cross-platform
//! gamepad support, running input polling in a background thread.

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use std::thread;
use std::time::{Duration, Instant};

/// Velocity command ranges (matching the Python runtime).
const X_RANGE: [f64; 2] = [-0.15, 0.15];
const Y_RANGE: [f64; 2] = [-0.2, 0.2];
const YAW_RANGE: [f64; 2] = [-1.0, 1.0];

const HEAD_PITCH_RANGE: [f64; 2] = [-0.78, 0.3];
const HEAD_YAW_RANGE: [f64; 2] = [-0.5, 0.5];
const HEAD_ROLL_RANGE: [f64; 2] = [-0.5, 0.5];

/// Button state with debounce/trigger detection.
#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonState {
    pub is_pressed: bool,
    pub triggered: bool,
    released: bool,
    last_pressed_time: f64,
}

impl ButtonState {
    const TIMEOUT: f64 = 0.2;

    fn new() -> Self {
        Self {
            released: true,
            ..Default::default()
        }
    }

    fn update(&mut self, value: bool, now: f64) {
        if self.is_pressed && !value {
            self.released = true;
        }
        self.is_pressed = value;

        if self.released && self.is_pressed && (now - self.last_pressed_time > Self::TIMEOUT) {
            self.triggered = true;
            self.last_pressed_time = now;
        } else {
            self.triggered = false;
        }

        if self.is_pressed {
            self.released = false;
        }
    }
}

/// All gamepad button states.
#[derive(Debug, Clone, Default)]
pub struct Buttons {
    pub a: ButtonState,
    pub b: ButtonState,
    pub x: ButtonState,
    pub y: ButtonState,
    pub lb: ButtonState,
    pub rb: ButtonState,
    pub dpad_up: ButtonState,
    pub dpad_down: ButtonState,
}

impl Buttons {
    fn new() -> Self {
        Self {
            a: ButtonState::new(),
            b: ButtonState::new(),
            x: ButtonState::new(),
            y: ButtonState::new(),
            lb: ButtonState::new(),
            rb: ButtonState::new(),
            dpad_up: ButtonState::new(),
            dpad_down: ButtonState::new(),
        }
    }
}

/// Command output from the controller.
#[derive(Debug, Clone)]
pub struct ControllerOutput {
    /// [lin_vel_x, lin_vel_y, ang_vel, neck_pitch, head_pitch, head_yaw, head_roll]
    pub commands: [f64; 7],
    pub buttons: Buttons,
    pub left_trigger: f64,
    pub right_trigger: f64,
}

impl Default for ControllerOutput {
    fn default() -> Self {
        Self {
            commands: [0.0; 7],
            buttons: Buttons::new(),
            left_trigger: 0.0,
            right_trigger: 0.0,
        }
    }
}

/// Xbox controller input handler running in a background thread.
pub struct XBoxController {
    receiver: Receiver<ControllerOutput>,
    stop_tx: Sender<()>,
    last_output: ControllerOutput,
}

impl XBoxController {
    /// Initialize the gamepad and start the background polling thread.
    pub fn new(command_freq: u32) -> Self {
        let (data_tx, data_rx) = bounded::<ControllerOutput>(1);
        let (stop_tx, stop_rx) = bounded::<()>(1);

        let period = Duration::from_secs_f64(1.0 / command_freq as f64);

        thread::spawn(move || {
            controller_worker(data_tx, stop_rx, period);
        });

        Self {
            receiver: data_rx,
            stop_tx,
            last_output: ControllerOutput::default(),
        }
    }

    /// Get the latest controller state (non-blocking).
    pub fn get_last_command(&mut self) -> &ControllerOutput {
        if let Ok(output) = self.receiver.try_recv() {
            self.last_output = output;
        }
        &self.last_output
    }

    /// Signal the background thread to stop.
    pub fn stop(&self) {
        let _ = self.stop_tx.try_send(());
    }
}

impl Drop for XBoxController {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Background worker that polls the gamepad at the command frequency.
fn controller_worker(
    data_tx: Sender<ControllerOutput>,
    stop_rx: Receiver<()>,
    period: Duration,
) {
    use gilrs::{Axis, Button, EventType, Gilrs};

    let mut gilrs = match Gilrs::new() {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to initialize gamepad library: {}", e);
            return;
        }
    };

    tracing::info!("Gamepad input thread started");

    // Track raw axis values
    let mut left_x: f64 = 0.0;
    let mut left_y: f64 = 0.0;
    let mut right_x: f64 = 0.0;
    let mut _right_y: f64 = 0.0;
    let mut left_trigger: f64 = 0.0;
    let mut right_trigger: f64 = 0.0;

    let mut a_pressed = false;
    let mut b_pressed = false;
    let mut x_pressed = false;
    let mut y_pressed = false;
    let mut lb_pressed = false;
    let mut rb_pressed = false;
    let mut dpad_up = false;
    let mut dpad_down = false;

    let mut buttons = Buttons::new();
    let mut head_control_mode = false;

    let start_time = Instant::now();

    loop {
        let tick_start = Instant::now();

        if stop_rx.try_recv().is_ok() {
            break;
        }

        // Process all pending events
        while let Some(event) = gilrs.next_event() {
            match event.event {
                EventType::AxisChanged(axis, value, _) => {
                    let v = value as f64;
                    match axis {
                        Axis::LeftStickX => left_x = -v,
                        Axis::LeftStickY => left_y = -v,
                        Axis::RightStickX => right_x = -v,
                        Axis::RightStickY => _right_y = -v,
                        Axis::LeftZ => {
                            left_trigger = ((v + 1.0) / 2.0).max(0.0);
                            if left_trigger < 0.1 {
                                left_trigger = 0.0;
                            }
                        }
                        Axis::RightZ => {
                            right_trigger = ((v + 1.0) / 2.0).max(0.0);
                            if right_trigger < 0.1 {
                                right_trigger = 0.0;
                            }
                        }
                        Axis::DPadX => {
                            // Not used in the Python version
                        }
                        Axis::DPadY => {
                            dpad_up = v > 0.5;
                            dpad_down = v < -0.5;
                        }
                        _ => {}
                    }
                }
                EventType::ButtonPressed(button, _) => match button {
                    Button::South => a_pressed = true,
                    Button::East => b_pressed = true,
                    Button::West => x_pressed = true,
                    Button::North => {
                        y_pressed = true;
                        head_control_mode = !head_control_mode;
                    }
                    Button::LeftTrigger => lb_pressed = true,
                    Button::RightTrigger => rb_pressed = true,
                    Button::DPadUp => dpad_up = true,
                    Button::DPadDown => dpad_down = true,
                    _ => {}
                },
                EventType::ButtonReleased(button, _) => match button {
                    Button::South => a_pressed = false,
                    Button::East => b_pressed = false,
                    Button::West => x_pressed = false,
                    Button::North => y_pressed = false,
                    Button::LeftTrigger => lb_pressed = false,
                    Button::RightTrigger => rb_pressed = false,
                    Button::DPadUp => dpad_up = false,
                    Button::DPadDown => dpad_down = false,
                    _ => {}
                },
                _ => {}
            }
        }

        // Compute commands
        let mut commands = [0.0f64; 7];

        if !head_control_mode {
            // Walking mode: left stick = velocity, right stick X = yaw
            let mut lin_vel_x = left_y;
            let mut lin_vel_y = left_x;
            let mut ang_vel = right_x;

            if lin_vel_x >= 0.0 {
                lin_vel_x *= X_RANGE[1].abs();
            } else {
                lin_vel_x *= X_RANGE[0].abs();
            }

            if lin_vel_y >= 0.0 {
                lin_vel_y *= Y_RANGE[1].abs();
            } else {
                lin_vel_y *= Y_RANGE[0].abs();
            }

            if ang_vel >= 0.0 {
                ang_vel *= YAW_RANGE[1].abs();
            } else {
                ang_vel *= YAW_RANGE[0].abs();
            }

            commands[0] = lin_vel_x;
            commands[1] = lin_vel_y;
            commands[2] = ang_vel;
        } else {
            // Head control mode
            let mut head_yaw = left_x;
            let mut head_pitch = left_y;
            let mut head_roll = right_x;

            if head_yaw >= 0.0 {
                head_yaw *= HEAD_YAW_RANGE[0].abs();
            } else {
                head_yaw *= HEAD_YAW_RANGE[1].abs();
            }

            if head_pitch >= 0.0 {
                head_pitch *= HEAD_PITCH_RANGE[0].abs();
            } else {
                head_pitch *= HEAD_PITCH_RANGE[1].abs();
            }

            if head_roll >= 0.0 {
                head_roll *= HEAD_ROLL_RANGE[0].abs();
            } else {
                head_roll *= HEAD_ROLL_RANGE[1].abs();
            }

            commands[4] = head_pitch;
            commands[5] = head_yaw;
            commands[6] = head_roll;
        }

        // Update button states
        let now = start_time.elapsed().as_secs_f64();
        buttons.a.update(a_pressed, now);
        buttons.b.update(b_pressed, now);
        buttons.x.update(x_pressed, now);
        buttons.y.update(y_pressed, now);
        buttons.lb.update(lb_pressed, now);
        buttons.rb.update(rb_pressed, now);
        buttons.dpad_up.update(dpad_up, now);
        buttons.dpad_down.update(dpad_down, now);

        let output = ControllerOutput {
            commands,
            buttons: buttons.clone(),
            left_trigger,
            right_trigger,
        };

        // Non-blocking send
        match data_tx.try_send(output.clone()) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                // Channel full — receiver hasn't consumed yet, skip this update.
                // Next tick will send fresh data.
            }
            Err(TrySendError::Disconnected(_)) => break,
        }

        let elapsed = tick_start.elapsed();
        if elapsed < period {
            spin_sleep::sleep(period - elapsed);
        }
    }

    tracing::info!("Controller worker thread exiting");
}
