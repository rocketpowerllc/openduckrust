//! GPIO peripherals — feet contacts, LED eyes, projector, antennas.
//!
//! Replaces `feet_contacts.py`, `eyes.py`, `projector.py`, `antennas.py`.
//! All GPIO access goes through the `rppal` crate (Linux-only).

// ── Hardware implementations (Linux only — requires rppal / GPIO) ──

#[cfg(target_os = "linux")]
mod hw {
    use anyhow::{Context, Result};
    use rppal::gpio::{Gpio, InputPin, OutputPin};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    // GPIO pin assignments (BCM numbering, matching the Python runtime)
    const LEFT_FOOT_PIN: u8 = 22;
    const RIGHT_FOOT_PIN: u8 = 27;
    const LEFT_EYE_PIN: u8 = 24;
    const RIGHT_EYE_PIN: u8 = 23;
    const PROJECTOR_PIN: u8 = 25;
    const LEFT_ANTENNA_PIN: u8 = 13;
    const RIGHT_ANTENNA_PIN: u8 = 12;

    // ── Feet Contact Sensors ──

    /// Binary foot contact sensors (pull-up GPIOs).
    pub struct FeetContacts {
        left: InputPin,
        right: InputPin,
    }

    impl FeetContacts {
        pub fn new() -> Result<Self> {
            let gpio = Gpio::new().context("Failed to initialize GPIO")?;
            let left = gpio
                .get(LEFT_FOOT_PIN)
                .context("Failed to get left foot pin")?
                .into_input_pullup();
            let right = gpio
                .get(RIGHT_FOOT_PIN)
                .context("Failed to get right foot pin")?
                .into_input_pullup();

            tracing::info!("Feet contact sensors initialized");
            Ok(Self { left, right })
        }

        /// Returns [left_contact, right_contact] as f64 (0.0 or 1.0).
        pub fn get(&self) -> [f64; 2] {
            // Active low: pin LOW = foot in contact
            let left = if self.left.is_low() { 1.0 } else { 0.0 };
            let right = if self.right.is_low() { 1.0 } else { 0.0 };
            [left, right]
        }
    }

    // ── LED Eyes ──

    /// Blinking LED eyes running in a background thread.
    pub struct Eyes {
        stop_flag: Arc<AtomicBool>,
        _thread: thread::JoinHandle<()>,
    }

    impl Eyes {
        pub fn new() -> Result<Self> {
            let gpio = Gpio::new()?;
            let left_eye = gpio.get(LEFT_EYE_PIN)?.into_output();
            let right_eye = gpio.get(RIGHT_EYE_PIN)?.into_output();

            let stop_flag = Arc::new(AtomicBool::new(false));
            let flag = stop_flag.clone();

            let handle = thread::spawn(move || {
                eyes_worker(left_eye, right_eye, flag);
            });

            tracing::info!("LED eyes initialized");
            Ok(Self {
                stop_flag,
                _thread: handle,
            })
        }

        pub fn stop(&self) {
            self.stop_flag.store(true, Ordering::Relaxed);
        }
    }

    impl Drop for Eyes {
        fn drop(&mut self) {
            self.stop();
        }
    }

    fn eyes_worker(mut left: OutputPin, mut right: OutputPin, stop: Arc<AtomicBool>) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        while !stop.load(Ordering::Relaxed) {
            // Blink: eyes off briefly
            left.set_low();
            right.set_low();
            thread::sleep(Duration::from_millis(100));

            // Eyes on
            left.set_high();
            right.set_high();

            // Random interval before next blink
            let interval_ms = rng.gen_range(1000..4000);
            thread::sleep(Duration::from_millis(interval_ms));
        }

        left.set_low();
        right.set_low();
    }

    // ── Projector ──

    /// GPIO-controlled projector toggle.
    pub struct Projector {
        pin: OutputPin,
        is_on: bool,
    }

    impl Projector {
        pub fn new() -> Result<Self> {
            let gpio = Gpio::new()?;
            let pin = gpio.get(PROJECTOR_PIN)?.into_output();
            tracing::info!("Projector initialized");
            Ok(Self { pin, is_on: false })
        }

        pub fn switch(&mut self) {
            self.is_on = !self.is_on;
            if self.is_on {
                self.pin.set_high();
            } else {
                self.pin.set_low();
            }
        }

        pub fn stop(&mut self) {
            self.pin.set_low();
        }
    }

    impl Drop for Projector {
        fn drop(&mut self) {
            self.stop();
        }
    }

    // ── Antennas (PWM Servos) ──

    /// PWM-controlled antenna servos.
    pub struct Antennas {
        left: OutputPin,
        right: OutputPin,
    }

    impl Antennas {
        pub fn new() -> Result<Self> {
            let gpio = Gpio::new()?;
            let left = gpio.get(LEFT_ANTENNA_PIN)?.into_output();
            let right = gpio.get(RIGHT_ANTENNA_PIN)?.into_output();

            tracing::info!("Antenna servos initialized");
            Ok(Self { left, right })
        }

        /// Set left antenna position (-1.0 to 1.0).
        pub fn set_position_left(&mut self, position: f64) {
            set_antenna_position(&mut self.left, position, 1.0);
        }

        /// Set right antenna position (-1.0 to 1.0).
        pub fn set_position_right(&mut self, position: f64) {
            set_antenna_position(&mut self.right, position, -1.0);
        }

        pub fn stop(&mut self) {
            set_antenna_position(&mut self.left, 0.0, 1.0);
            set_antenna_position(&mut self.right, 0.0, -1.0);
        }
    }

    impl Drop for Antennas {
        fn drop(&mut self) {
            self.stop();
        }
    }

    /// Convert a -1.0..1.0 value to a PWM duty cycle for a hobby servo.
    ///
    /// Uses software PWM bit-banging via rppal output pins.
    /// For production use, consider rppal's hardware PWM channels.
    fn set_antenna_position(pin: &mut OutputPin, value: f64, sign: f64) {
        let v = (value * sign).clamp(-1.0, 1.0);
        // Pulse width: 1.0ms (-1) to 2.0ms (+1), center 1.5ms
        let pulse_width_us = ((1.5 + v * 0.5) * 1000.0) as u64;

        // Software PWM: set high for pulse width, then low for remainder of 20ms period
        pin.set_high();
        thread::sleep(Duration::from_micros(pulse_width_us));
        pin.set_low();
        thread::sleep(Duration::from_micros(20_000 - pulse_width_us));
    }
}

#[cfg(target_os = "linux")]
pub use hw::{Antennas, Eyes, FeetContacts, Projector};

// ── Mock implementations (always available) ──

/// Mock feet contacts that always report no contact.
pub struct MockFeetContacts;

impl MockFeetContacts {
    pub fn get(&self) -> [f64; 2] {
        [0.0, 0.0]
    }
}
