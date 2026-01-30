# OpenDuckRust

A high-performance Rust port of the [Open Duck Mini](https://github.com/apirrone/Open_Duck_Mini) on-device runtime, purpose-built for zero-overhead bipedal robot control on resource-constrained hardware.

## About

OpenDuckRust replaces the Python runtime that runs on the Raspberry Pi inside the Open Duck Mini robot with a native Rust binary. The robot's brain — a reinforcement learning policy trained in simulation — stays in its universal ONNX format. Only the real-time control loop that executes on the robot is rewritten.

This project stands on the shoulders of the **Open Duck Mini team**, led by **Antoine Pirrone** and **Gregoire Passault**, whose work building an affordable, open-source bipedal walking robot inspired by Disney's BDX droid has been extraordinary. Their contributions to open-source robotics — including the hardware design, sim-to-real training pipeline, and the original Python runtime — make projects like this one possible. OpenDuckRust is a direct derivative of their [Open_Duck_Mini_Runtime](https://github.com/apirrone/Open_Duck_Mini_Runtime).

## Why Rust?

The Open Duck Mini runs on a **Raspberry Pi Zero 2W** — a computer with 512MB of RAM and a modest quad-core ARM CPU. The robot relies on active balancing at 50Hz: every 20 milliseconds, it must read sensors, run neural network inference, and command 14 servo motors. A single missed deadline means the robot falls over.

Python is the right language for training the policy (PyTorch, MuJoCo, GPU acceleration). But on the robot itself, Python's overhead becomes a liability.

### The Problem with Python on a Pi Zero

| Issue | Impact |
|-------|--------|
| **Garbage collector pauses** | Unpredictable 10-50ms stalls during the control loop |
| **Global Interpreter Lock** | IMU reading, inference, and motor commands cannot truly run in parallel |
| **Memory overhead** | Python interpreter + NumPy + ONNX Runtime consume a large share of 512MB |
| **Startup time** | Importing NumPy, pygame, and ONNX Runtime takes several seconds |
| **No compile-time safety** | Runtime type errors can crash the robot mid-walk |

### What Rust Delivers

| Benefit | Detail |
|---------|--------|
| **Deterministic timing** | No GC, no GIL. The control loop runs at a rock-solid 50Hz (or 100Hz) without jitter. `spin_sleep` provides microsecond-precision timing that avoids OS scheduler latency. |
| **~5x lower memory** | The entire runtime binary fits in a few MB. No interpreter, no dynamic library loading overhead. More RAM is free for the OS and the ONNX model. |
| **True parallelism** | IMU reading, gamepad polling, and the main control loop run on separate OS threads with zero contention — no GIL. |
| **Compile-time correctness** | Servo IDs, observation vector dimensions, and protocol byte layouts are checked at compile time. The robot cannot crash from a `TypeError`. |
| **Single static binary** | `cargo build --release --target aarch64-unknown-linux-gnu` produces one file. Copy it to the Pi and run. No `pip install`, no virtualenv, no dependency conflicts. |
| **Sub-second startup** | The binary loads the ONNX model and begins the control loop in under a second. |

### Architectural Principle

The training stack stays in Python. The runtime runs in Rust. The ONNX model is the bridge.

```
┌──────────────────────────┐     ┌──────────────────────────┐
│   TRAINING (Desktop PC)  │     │    RUNTIME (Pi Zero 2W)  │
│                          │     │                          │
│  Python + PyTorch        │     │  Rust (this project)     │
│  MuJoCo / Isaac Sim      │────▶│  ort (ONNX Runtime)      │
│  RL policy training      │     │  rppal (GPIO / I2C)      │
│                          │ .onnx  serialport (servos)     │
│  Export: policy.onnx     │     │  gilrs (gamepad)         │
└──────────────────────────┘     └──────────────────────────┘
```

## What Gets Ported

The `runtime/` crate is a 1:1 port of every module in the [Open_Duck_Mini_Runtime](https://github.com/apirrone/Open_Duck_Mini_Runtime):

| Python Module | Rust Module | Purpose |
|---------------|-------------|---------|
| `v2_rl_walk_mujoco.py` | `main.rs` | Main control loop: read sensors → infer → write motors |
| `onnx_infer.py` | `inference.rs` | ONNX policy forward pass |
| `rustypot_position_hwi.py` | `motors.rs` | Feetech STS3215 servo protocol over serial USB |
| `raw_imu.py` | `imu.rs` | BNO055 IMU via I2C (gyro + accelerometer) |
| `rl_utils.py` | `rl_utils.rs` | Action filters, joint reordering, quaternion math |
| `poly_reference_motion.py` | `reference_motion.rs` | Gait phase tracking |
| `xbox_controller.py` | `controller.rs` | Gamepad input via gilrs |
| `duck_config.py` | `config.rs` | JSON configuration loader |
| `feet_contacts.py` | `peripherals.rs` | GPIO foot contact sensors |
| `eyes.py` | `peripherals.rs` | LED eye blink animation |
| `projector.py` | `peripherals.rs` | GPIO projector toggle |
| `antennas.py` | `peripherals.rs` | PWM antenna servo control |
| `sounds.py` | `sounds.rs` | WAV audio playback |

## Project Structure

```
openduckrust/
├── runtime/               ← Rust on-device runtime (this is the core)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # Entry point + control loop
│       ├── config.rs          # Duck configuration (JSON)
│       ├── inference.rs       # ONNX policy inference
│       ├── motors.rs          # Feetech servo protocol
│       ├── imu.rs             # BNO055 IMU (I2C)
│       ├── rl_utils.rs        # Action filters, math utilities
│       ├── reference_motion.rs # Gait phase tracker
│       ├── controller.rs      # Xbox gamepad input
│       ├── peripherals.rs     # GPIO: feet, eyes, projector, antennas
│       └── sounds.rs          # Audio playback
├── backend/               ← Rust API server (utoipa, Cedar RBAC)
├── cdk/                   ← AWS CDK infrastructure
├── web-app/               ← React + Vite frontend
├── mobile-app/            ← React Native (Expo)
├── cli/                   ← Rust CLI tool
├── mcp/                   ← Model Context Protocol server
├── desktop/               ← Tauri desktop wrapper
├── marketing-site/        ← Static HTML/CSS
└── scripts/               ← Deployment scripts
```

## Supported Targets

OpenDuckRust compiles to every CPU architecture used in modern robotics. GPU/NPU accelerators (CUDA, TensorRT, Edge TPU) are linked at runtime, not at compile time — the same binary runs with or without a GPU.

### ARM64 (AArch64) — Primary Robotics Target

| Target | Hardware | Build Script |
|--------|----------|--------------|
| `aarch64-unknown-linux-gnu` | **Pi Zero 2W**, Pi 3/4/5, all NVIDIA Jetsons, DGX Spark (GB10), Orange Pi 5, Rock 5B, Qualcomm RB3/RB5, Google Coral, BeagleBone AI-64 | `build-pi-zero2w.sh` |
| `aarch64-unknown-linux-musl` | Any ARM64 Linux (fully static binary, no libc dependency) | `build-aarch64-static.sh` |

### ARM32 (ARMv7 / ARMv6)

| Target | Hardware | Build Script |
|--------|----------|--------------|
| `armv7-unknown-linux-gnueabihf` | Pi (32-bit OS), BeagleBone Black/AI, older ARM SBCs | `build-pi-armv7.sh` |
| `arm-unknown-linux-gnueabihf` | Pi Zero W original (ARMv6, ARM1176JZF-S) | `build-pi-zero-v1.sh` |

### NVIDIA Jetson (CUDA / TensorRT)

All Jetsons compile as `aarch64-unknown-linux-gnu`. GPU acceleration is a runtime feature.

| Board | GPU | TOPS | CUDA | Build Script |
|-------|-----|------|------|--------------|
| Jetson Nano | Maxwell 128-core | — | JetPack 4 | `build-jetson.sh` |
| Jetson TX2 | Pascal 256-core | — | JetPack 4 | `build-jetson.sh` |
| Jetson Xavier NX | Volta 384-core | 21 | JetPack 5 | `build-jetson-cuda.sh` |
| Jetson AGX Xavier | Volta 512-core | 32 | JetPack 5 | `build-jetson-cuda.sh` |
| Jetson Orin Nano | Ampere 1024-core | 67 | JetPack 6 | `build-jetson-cuda.sh` |
| Jetson AGX Orin | Ampere 2048-core | 275 | JetPack 6 | `build-jetson-cuda.sh` |
| **Jetson AGX Thor** | **Blackwell** | **2,070** | JetPack 7 | `build-jetson-cuda.sh` |

For CUDA/TensorRT inference, build ONNX Runtime on-device and set `ORT_DYLIB_PATH`:

```bash
# On the Jetson
export ORT_DYLIB_PATH=/usr/lib/libonnxruntime.so
./openduckrust-runtime --onnx-model-path policy.onnx
```

### NVIDIA DGX Spark (GB10 Grace Blackwell)

| Target | Hardware | Build Script |
|--------|----------|--------------|
| `aarch64-unknown-linux-gnu` | DGX Spark: 20x ARM Neoverse V2 cores + Blackwell GPU, 128GB unified memory, 1 PFLOP FP4 | `build-dgx-spark.sh` |

Requires CUDA 13.0+ for Blackwell (sm_121). On-device native build recommended.

### RISC-V

| Target | Hardware | Build Script |
|--------|----------|--------------|
| `riscv64gc-unknown-linux-gnu` | BeagleV-Ahead, Milk-V Mars/Jupiter/Megrez, VisionFive 2, Banana Pi BPI-F3 | `build-riscv64.sh` |

### x86_64

| Target | Hardware | Build Script |
|--------|----------|--------------|
| `x86_64-unknown-linux-gnu` | Desktop Linux, dev machines, CI, CUDA GPU workstations | `build-x86-linux.sh` |
| `x86_64-unknown-linux-musl` | Portable static binary (no libc dependency) | `build-x86-linux-static.sh` |

### macOS (Development)

| Target | Hardware | Build Script |
|--------|----------|--------------|
| `aarch64-apple-darwin` | Apple Silicon M1/M2/M3/M4 | `build-macos.sh` |
| `x86_64-apple-darwin` | Intel Mac | `build-macos.sh` |

### Other Boards

| Target | Hardware | Build Script |
|--------|----------|--------------|
| `aarch64-unknown-linux-gnu` | Google Coral Dev Board, Qualcomm RB3/RB5, Rubik Pi 3 | `build-coral.sh`, `build-qualcomm.sh` |
| `armv7-unknown-linux-gnueabihf` | BeagleBone Black/AI | `build-beaglebone.sh` |

### GPU / NPU Accelerator Support

Accelerators are linked at runtime via ONNX Runtime execution providers, not at compile time:

| Accelerator | Boards | How to Enable |
|-------------|--------|---------------|
| **CUDA** | Jetson Xavier+, Orin, Thor, DGX Spark, x86_64 GPU workstations | Build ONNX Runtime with `--use_cuda`, set `ORT_DYLIB_PATH` |
| **TensorRT** | Jetson (JetPack 5+), DGX Spark | Build ONNX Runtime with `--use_tensorrt`, set `ORT_DYLIB_PATH` |
| **Edge TPU** | Google Coral | No native Rust SDK; use C FFI to `libedgetpu` or sidecar process |
| **Hexagon DSP** | Qualcomm RB boards | Proprietary Qualcomm SDK only; use CPU ONNX inference instead |
| **Rockchip NPU** | Orange Pi 5, Rock 5B (RK3588) | RKNN SDK (C API); no Rust bindings yet |

### Build Scripts

All build scripts are in `runtime/scripts/`:

```bash
# Build for a specific target
./runtime/scripts/build-pi-zero2w.sh      # Raspberry Pi Zero 2W / Pi 3/4/5
./runtime/scripts/build-jetson-cuda.sh     # NVIDIA Jetson with CUDA
./runtime/scripts/build-riscv64.sh         # RISC-V boards
./runtime/scripts/build-macos.sh           # macOS (development)

# Build ALL targets at once
./runtime/scripts/build-all.sh             # Build all (needs cross-compilers)
./runtime/scripts/build-all.sh --use-cross # Build all via Docker (cross tool)
./runtime/scripts/build-all.sh --list      # List all targets without building
```

## Getting Started

### Prerequisites

- Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- A trained ONNX policy file (from the Open Duck Mini training pipeline)
- A configured Open Duck Mini robot

### Quick Build (Raspberry Pi)

```bash
cd runtime
./scripts/build-pi-zero2w.sh

# Copy to the Pi
scp target/aarch64-unknown-linux-gnu/release/openduckrust-runtime pi@<PI_IP>:~/
```

### Cross-Compile Any Target

```bash
# Install the cross tool (uses Docker, no manual toolchain setup)
cargo install cross

# Build for any target
cross build --target aarch64-unknown-linux-gnu --release    # ARM64
cross build --target armv7-unknown-linux-gnueabihf --release # ARM32
cross build --target riscv64gc-unknown-linux-gnu --release   # RISC-V
```

### Run on the Robot

```bash
./openduckrust-runtime \
    --onnx-model-path ./policy.onnx \
    --duck-config-path ~/duck_config.json \
    --serial-port /dev/ttyACM0 \
    --control-freq 50 \
    --action-scale 0.25 \
    -p 30 \
    --commands
```

### Configuration

The robot uses a `duck_config.json` file (same format as the Python runtime):

```json
{
    "start_paused": false,
    "imu_upside_down": false,
    "phase_frequency_factor_offset": 0.0,
    "expression_features": {
        "eyes": false,
        "projector": false,
        "antennas": false,
        "speaker": false
    },
    "joints_offsets": {
        "left_hip_yaw": 0.0,
        "left_hip_roll": 0.0,
        "left_hip_pitch": 0.0,
        "left_knee": 0.0,
        "left_ankle": 0.0,
        "neck_pitch": 0.0,
        "head_pitch": 0.0,
        "head_yaw": 0.0,
        "head_roll": 0.0,
        "right_hip_yaw": 0.0,
        "right_hip_roll": 0.0,
        "right_hip_pitch": 0.0,
        "right_knee": 0.0,
        "right_ankle": 0.0
    }
}
```

## Rust Crate Dependencies

| Crate | Purpose |
|-------|---------|
| `ort` | ONNX Runtime — runs the trained neural network policy |
| `rppal` | Raspberry Pi GPIO, I2C, PWM — IMU, foot sensors, LEDs, antennas |
| `serialport` | Serial communication — Feetech STS3215 bus servos at 1Mbaud |
| `gilrs` | Cross-platform gamepad — Xbox controller via Bluetooth |
| `nalgebra` | Linear algebra — quaternion and vector math for IMU processing |
| `ndarray` | N-dimensional arrays — ONNX input/output tensors |
| `serde` / `serde_json` | Configuration — duck_config.json parsing |
| `clap` | CLI argument parsing |
| `crossbeam-channel` | Lock-free channels — IMU and gamepad background threads |
| `spin_sleep` | Microsecond-precision sleep — deterministic control loop timing |
| `rodio` | Audio playback — duck sound effects |
| `tracing` | Structured logging — JSON output to stdout |
| `anyhow` | Error handling — rich context on every failure path |
| `byteorder` | Byte encoding — Feetech servo protocol packet construction |

## Credits

This project is built on the work of the **Open Duck Mini** community:

- **[Antoine Pirrone](https://github.com/apirrone)** — Project lead, hardware design, runtime, training pipeline
- **[Gregoire Passault](https://github.com/Rhoban)** — Co-creator, robotics expertise
- **[Open Duck Mini](https://github.com/apirrone/Open_Duck_Mini)** — The original open-source bipedal robot project (2.2k+ stars)
- **[Open Duck Mini Runtime](https://github.com/apirrone/Open_Duck_Mini_Runtime)** — The Python runtime that this project ports to Rust
- **[Open Duck Playground](https://github.com/apirrone/Open_Duck_Playground)** — MuJoCo-based RL training environments
- **[HuggingFace](https://huggingface.co/)** — Project sponsor
- **Disney Research** — The original BDX droid paper that inspired the project

## License

MIT
