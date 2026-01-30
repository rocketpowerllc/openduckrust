//! BNO055 IMU sensor reading over I2C.
//!
//! Replaces `raw_imu.py`. Reads gyroscope and accelerometer data in a
//! background thread at the control frequency, providing jitter-free data
//! to the main control loop.

// Hardware-specific imports are inside the cfg-gated hw module.

/// IMU data packet: gyroscope and accelerometer readings.
#[derive(Debug, Clone, Copy, Default)]
pub struct ImuData {
    /// Gyroscope readings [x, y, z] in rad/s.
    pub gyro: [f64; 3],
    /// Accelerometer readings [x, y, z] in m/s^2.
    pub accel: [f64; 3],
}

/// Trait for IMU implementations (supports dependency injection for testing).
pub trait ImuReader: Send {
    fn get_data(&self) -> ImuData;
    fn stop(&self);
}

// ── Hardware implementation (Linux only — requires rppal / I2C) ──

#[cfg(target_os = "linux")]
mod hw {
    use super::{ImuData, ImuReader};
    use anyhow::{Context, Result};
    use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
    use rppal::i2c::I2c;
    use std::thread;
    use std::time::{Duration, Instant};

    // BNO055 I2C address
    const BNO055_ADDR: u16 = 0x28;

    // Register addresses
    const BNO055_OPR_MODE: u8 = 0x3D;
    const BNO055_GYRO_DATA: u8 = 0x14; // 6 bytes: X, Y, Z (each 2 bytes LE)
    const BNO055_ACCEL_DATA: u8 = 0x08; // 6 bytes: X, Y, Z (each 2 bytes LE)
    const BNO055_AXIS_MAP_CONFIG: u8 = 0x41;
    const BNO055_AXIS_MAP_SIGN: u8 = 0x42;

    // Operating modes
    const NDOF_MODE: u8 = 0x0C;
    const CONFIG_MODE: u8 = 0x00;

    /// BNO055 IMU reader running in a background thread.
    pub struct Imu {
        receiver: Receiver<ImuData>,
        stop_tx: Sender<()>,
        last_data: std::cell::Cell<ImuData>,
    }

    impl Imu {
        /// Initialize the BNO055 and start the background sampling thread.
        pub fn new(sampling_freq: u32, upside_down: bool) -> Result<Self> {
            let (data_tx, data_rx) = bounded::<ImuData>(1);
            let (stop_tx, stop_rx) = bounded::<()>(1);

            let mut i2c = I2c::new().context("Failed to open I2C bus")?;
            i2c.set_slave_address(BNO055_ADDR)
                .context("Failed to set I2C slave address")?;

            // Enter config mode for axis remap
            i2c.smbus_write_byte(BNO055_OPR_MODE, CONFIG_MODE)?;
            thread::sleep(Duration::from_millis(25));

            // Remap axes for the duck's orientation
            i2c.smbus_write_byte(BNO055_AXIS_MAP_CONFIG, 0x21)?;

            // Set axis signs based on mounting orientation
            if upside_down {
                i2c.smbus_write_byte(BNO055_AXIS_MAP_SIGN, 0x07)?;
            } else {
                i2c.smbus_write_byte(BNO055_AXIS_MAP_SIGN, 0x04)?;
            }

            // Enter NDOF mode
            i2c.smbus_write_byte(BNO055_OPR_MODE, NDOF_MODE)?;
            thread::sleep(Duration::from_millis(25));

            tracing::info!(
                "BNO055 IMU initialized at {} Hz (upside_down={})",
                sampling_freq,
                upside_down
            );

            // Spawn background reader thread
            let period = Duration::from_secs_f64(1.0 / sampling_freq as f64);
            thread::spawn(move || {
                imu_worker(i2c, data_tx, stop_rx, period);
            });

            Ok(Self {
                receiver: data_rx,
                stop_tx,
                last_data: std::cell::Cell::new(ImuData::default()),
            })
        }

        /// Get the latest IMU data (non-blocking).
        pub fn get_data(&self) -> ImuData {
            if let Ok(data) = self.receiver.try_recv() {
                self.last_data.set(data);
            }
            self.last_data.get()
        }

        /// Signal the background thread to stop.
        pub fn stop(&self) {
            let _ = self.stop_tx.try_send(());
        }
    }

    impl ImuReader for Imu {
        fn get_data(&self) -> ImuData {
            self.get_data()
        }

        fn stop(&self) {
            self.stop()
        }
    }

    impl Drop for Imu {
        fn drop(&mut self) {
            self.stop();
        }
    }

    /// Background worker that reads IMU data at a fixed frequency.
    fn imu_worker(
        mut i2c: I2c,
        data_tx: Sender<ImuData>,
        stop_rx: Receiver<()>,
        period: Duration,
    ) {
        loop {
            let start = Instant::now();

            if stop_rx.try_recv().is_ok() {
                break;
            }

            let gyro = match read_vector(&mut i2c, BNO055_GYRO_DATA) {
                Ok(raw) => [raw[0] / 900.0, raw[1] / 900.0, raw[2] / 900.0],
                Err(e) => {
                    tracing::trace!("IMU gyro read error: {}", e);
                    continue;
                }
            };

            let accel = match read_vector(&mut i2c, BNO055_ACCEL_DATA) {
                Ok(raw) => [raw[0] / 100.0, raw[1] / 100.0, raw[2] / 100.0],
                Err(e) => {
                    tracing::trace!("IMU accel read error: {}", e);
                    continue;
                }
            };

            let data = ImuData { gyro, accel };

            match data_tx.try_send(data) {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => {
                    let _ = data_tx.try_recv();
                    let _ = data_tx.try_send(data);
                }
                Err(TrySendError::Disconnected(_)) => break,
            }

            let elapsed = start.elapsed();
            if elapsed < period {
                spin_sleep::sleep(period - elapsed);
            }
        }

        tracing::info!("IMU worker thread exiting");
    }

    /// Read a 3-axis vector (6 bytes, little-endian i16) from the BNO055.
    fn read_vector(i2c: &mut I2c, register: u8) -> Result<[f64; 3]> {
        let mut buf = [0u8; 6];
        i2c.block_read(register, &mut buf)
            .context("I2C block read failed")?;

        let x = i16::from_le_bytes([buf[0], buf[1]]) as f64;
        let y = i16::from_le_bytes([buf[2], buf[3]]) as f64;
        let z = i16::from_le_bytes([buf[4], buf[5]]) as f64;

        Ok([x, y, z])
    }
}

#[cfg(target_os = "linux")]
pub use hw::Imu;

/// Mock IMU for testing without hardware.
pub struct MockImu {
    data: ImuData,
}

impl MockImu {
    pub fn new() -> Self {
        Self {
            data: ImuData {
                gyro: [0.0; 3],
                accel: [0.0, 0.0, 9.81],
            },
        }
    }
}

impl ImuReader for MockImu {
    fn get_data(&self) -> ImuData {
        self.data
    }

    fn stop(&self) {}
}
