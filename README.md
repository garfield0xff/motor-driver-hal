# Motor Driver HAL

A hardware abstraction layer (HAL) for motor drivers built on top of `embedded-hal` traits. This crate provides a generic, platform-independent interface for controlling H-bridge motor drivers commonly used in embedded systems and robotics applications.

## Features

- **Generic H-Bridge Support**: Works with any H-bridge motor driver
- **Flexible Configuration**: Single PWM or dual PWM control modes
- **Platform Independent**: Built on `embedded-hal` traits
- **Optional Features**: 
  - `std` - Standard library support (enabled by default)
  - `rppal` - Raspberry Pi support via rppal crate
  - `linux-embedded-hal` - Linux GPIO support
- **No-std Compatible**: Can be used in embedded environments

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

### Raspberry Pi Example

```rust
use motor_driver_hal::{HBridgeMotorDriver, MotorDriver, GpioWrapper, PwmWrapper};
use rppal::gpio::Gpio;
use rppal::pwm::{Channel, Pwm, Polarity};

// Create GPIO and PWM instances
let gpio = Gpio::new()?;
let r_en = GpioWrapper::new(gpio.get(23)?.into_output());
let l_en = GpioWrapper::new(gpio.get(24)?.into_output());

let r_pwm = PwmWrapper::new(
    Pwm::with_frequency(Channel::Pwm1, 1000.0, 0.0, Polarity::Normal, true)?, 
    1000
);
let l_pwm = PwmWrapper::new(
    Pwm::with_frequency(Channel::Pwm2, 1000.0, 0.0, Polarity::Normal, true)?, 
    1000
);

// Create motor driver instance
let mut motor = HBridgeMotorDriver::dual_pwm(r_en, l_en, r_pwm, l_pwm, 1000);

// Initialize and use the motor
motor.initialize()?;
motor.enable()?;
motor.set_speed(300)?;  // Forward speed
motor.set_speed(-300)?; // Reverse speed
motor.stop()?;
motor.disable()?;
```

### Generic Example (Any Platform)

```rust
use motor_driver_hal::{HBridgeMotorDriver, MotorDriver};

// Using your platform's embedded-hal implementations
let mut motor = HBridgeMotorDriver::single_pwm(enable_pin, pwm_channel, 1000);

motor.initialize()?;
motor.enable()?;
motor.set_speed(500)?;
motor.stop()?;
```

## Examples

The `example/` directory contains practical Raspberry Pi implementations:

### Available Examples

- **`basic_motor`** - Simple dual PWM motor control
- **`speed_control`** - Variable speed control demonstration
- **`direction_control`** - Forward/reverse direction control
- **`brake_test`** - Motor braking functionality
- **`encoder_monitor`** - Motor with encoder feedback (placeholder)

### Running Examples

```bash
# Navigate to examples directory
cd example/

# Run examples on Raspberry Pi with rppal feature
cargo run --bin basic_motor --features rppal
cargo run --bin speed_control --features rppal
cargo run --bin direction_control --features rppal
```

**Note**: Examples require Raspberry Pi hardware with proper GPIO connections.

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
    
    // Status reading (some methods may return default values)
    fn get_speed(&self) -> Result<i16, Self::Error>;
    fn get_direction(&self) -> Result<bool, Self::Error>;
    fn get_current(&self) -> Result<f32, Self::Error>;
    fn get_voltage(&self) -> Result<f32, Self::Error>;
    fn get_temperature(&self) -> Result<f32, Self::Error>;
    fn get_fault_status(&self) -> Result<u8, Self::Error>;
}
```

### HBridgeMotorDriver

The main implementation supports two configurations:

```rust
// Single PWM configuration
HBridgeMotorDriver::single_pwm(enable_pin, pwm_channel, max_duty);

// Dual PWM configuration (for bidirectional control)
HBridgeMotorDriver::dual_pwm(enable1, enable2, pwm1, pwm2, max_duty);
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

### Platform Wrappers

This crate provides wrapper types to adapt platform-specific implementations to `embedded-hal` traits:

- `GpioWrapper` - Wraps GPIO pins implementing `OutputPin`
- `PwmWrapper` - Wraps PWM channels implementing `SetDutyCycle`

### Adding Platform Support

To use with your platform:

1. Implement `embedded-hal::digital::OutputPin` for your GPIO pins
2. Implement `embedded-hal::pwm::SetDutyCycle` for your PWM channels
3. Create instances using the appropriate constructor methods

### Supported Platforms

- **Raspberry Pi** (via `rppal` crate - included wrappers)
- **Linux** (via `linux-embedded-hal` - optional feature)
- **ESP32** (via `esp-hal` - bring your own wrappers)
- **STM32** (via `stm32-hal` family - bring your own wrappers)
- Any platform with `embedded-hal` support

## Configuration Features

Enable platform-specific features in your `Cargo.toml`:

```toml
# For Raspberry Pi
motor-driver-hal = { version = "0.1.0", features = ["rppal"] }

# For Linux GPIO
motor-driver-hal = { version = "0.1.0", features = ["linux-embedded-hal"] }

# For no_std embedded systems
motor-driver-hal = { version = "0.1.0", default-features = false }
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.