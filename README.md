# Motor Driver HAL

A hardware abstraction layer (HAL) for motor drivers built on top of `embedded-hal` traits. This crate provides a generic, platform-independent interface for controlling various types of motor drivers commonly used in embedded systems and robotics applications.

## Supported Motor Driver Types

- **H-Bridge Drivers**: Bidirectional DC motor control with optional brake functionality
  - ✅ **BTS7960**: High-current H-bridge motor driver module
- **Single Direction Drivers**: Simple unidirectional motor control
- **Dual H-Bridge**: Independent control of two motors

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
motor-driver-hal = "0.1.0"
```

For `no_std` environments, disable the default features:

```toml
[dependencies]
motor-driver-hal = { version = "0.1.0", default-features = false }
```

## Quick Start

```rust
use motor_driver_hal::{HBridgeMotorDriver, MotorDriver};
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;

// Your platform-specific GPIO and PWM implementations
// (see examples for Raspberry Pi implementation)

// Create motor driver instance
let enable_pin = your_gpio_pin;
let pwm_channel = your_pwm_channel;

let mut motor = HBridgeMotorDriver::new_single_enable_single_pwm(
    enable_pin,
    pwm_channel,
    1000  // max duty cycle
);

// Initialize and use the motor
motor.initialize()?;
motor.enable()?;
motor.set_speed(500)?;  // 50% forward speed
motor.set_speed(-300)?; // 30% reverse speed
motor.brake()?;
motor.stop()?;
```

## Examples

The `example/` directory contains practical implementations for different scenarios:

### Available Examples

- **`basic_motor`** - Simple motor control demonstration
- **`speed_control`** - Variable speed control with PWM
- **`direction_control`** - Forward/reverse direction control
- **`brake_test`** - Braking functionality demonstration
- **`encoder_monitor`** - Motor with encoder feedback

### Running Examples

```bash
# Navigate to examples directory
cd example/

# Run a specific example (requires Raspberry Pi hardware)
cargo run --bin basic_motor
cargo run --bin speed_control
cargo run --bin direction_control
```

## API Overview

### Core Trait: `MotorDriver`

All motor drivers implement the `MotorDriver` trait:

```rust
pub trait MotorDriver {
    type Error;
    
    // Initialization and control
    fn initialize(&mut self) -> Result<(), Self::Error>;
    fn enable(&mut self) -> Result<(), Self::Error>;
    fn disable(&mut self) -> Result<(), Self::Error>;
    
    // Speed and direction control
    fn set_speed(&mut self, speed: i16) -> Result<(), Self::Error>;
    fn set_direction(&mut self, forward: bool) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
    fn brake(&mut self) -> Result<(), Self::Error>;
    
    // Status reading
    fn get_speed(&self) -> Result<i16, Self::Error>;
    fn get_direction(&self) -> Result<bool, Self::Error>;
    fn get_current(&self) -> Result<f32, Self::Error>;
    fn get_voltage(&self) -> Result<f32, Self::Error>;
    fn get_temperature(&self) -> Result<f32, Self::Error>;
    fn get_fault_status(&self) -> Result<u8, Self::Error>;
}
```

### Speed Values

Speed is controlled using signed 16-bit integers:
- **Positive values**: Forward direction (0 to max_duty)
- **Negative values**: Reverse direction (-max_duty to 0)
- **Zero**: Motor stopped

### Motor States

1. **Uninitialized**: Fresh driver instance, not ready for use
2. **Initialized**: Driver configured and ready, but motor disabled
3. **Enabled**: Motor powered and ready to move
4. **Disabled**: Motor power cut, safe state

## Hardware Integration

To use this crate with your specific hardware platform:

1. Implement the `embedded-hal` traits for your GPIO and PWM peripherals
2. Create wrapper types that adapt your platform's types to the HAL traits
3. Use the motor driver with your wrapped types

### Supported Platforms

- **Raspberry Pi** (via `rppal` crate - see examples)
- **ESP32** (via `esp-hal`)
- **STM32** (via `stm32-hal` family)
- Any platform with `embedded-hal` support