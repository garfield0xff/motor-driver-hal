# Motor Driver HAL

A hardware abstraction layer (HAL) for motor drivers built on top of `embedded-hal` traits. This crate provides a generic, platform-independent interface for controlling H-bridge motor drivers commonly used in embedded systems and robotics applications.

## Installation

### Basic Installation

```toml
[dependencies]
motor-driver-hal = "0.1.3"
```

### Platform-Specific Installation

For **Raspberry Pi** projects:
```toml
[dependencies]
motor-driver-hal = { version = "0.1.3", features = ["rppal"] }
```

For **Linux GPIO** projects:
```toml
[dependencies]
motor-driver-hal = { version = "0.1.3", features = ["linux-embedded-hal"] }
```

For **embedded/no_std** environments:
```toml
[dependencies]
motor-driver-hal = { version = "0.1.3", default-features = false }
```

## Quick Start

### Raspberry Pi Example (Builder Pattern)

```rust
use motor_driver_hal::driver::rppal::RppalMotorDriverBuilder;
use motor_driver_hal::MotorDriver;
use rppal::gpio::Gpio;
use rppal::pwm::Channel;

// Initialize GPIO interface
let gpio = Gpio::new()?;

// Create motor driver with builder pattern
let mut motor = RppalMotorDriverBuilder::new_rppal()
    .with_dual_gpio_enable(&gpio, 23, 24)?        // Enable pins
    .with_dual_pwm_channels(                      // PWM configuration:
        Channel::Pwm1,                            //   - Channel1 
        Channel::Pwm2,                            //   - Channel2 
        1000.0,                                   //   - Frequency
        1000                                      //   - Max duty
    )?
    .with_encoder_pins(&gpio, 25, 8)?            // Encoder pins
    .with_ppr(1000)                              // Pulses per revolution: 1000
    .build_and_init()?;

// Control the motor
motor.enable()?;
motor.set_speed(300)?;  // 30% forward speed
motor.set_speed(-300)?; // 30% reverse speed
motor.stop()?;
motor.disable()?;
```

### Linux Example (GPIO and PWM)

```rust
use motor_driver_hal::driver::linux::LinuxMotorDriverBuilder;
use motor_driver_hal::MotorDriver;
use linux_embedded_hal::gpio_cdev::Chip;

// Initialize GPIO chip
let mut chip = Chip::new("/dev/gpiochip0")?;

// Create motor driver with builder pattern
let mut motor = LinuxMotorDriverBuilder::new_linux()
    .with_dual_gpio_enable(&mut chip, 23, 24)?   // Enable pins: GPIO 23, 24
    .with_dual_pwm_channels(                     // PWM configuration:
        0,                                       //   - PWM chip 0
        0, 1,                                    //   - Channels 0, 1
        1000                                     //   - Max duty: 1000
    )
    .build_and_init()?;

// Control the motor
motor.enable()?;
motor.set_speed(300)?;  // 30% forward speed
motor.stop()?;
motor.disable()?;
```

## Examples

The `example/` directory contains practical Raspberry Pi implementations:

### Available Examples

- **`rpi_basic_motor`** - Simple Raspberry Pi motor control
- **`rpi_speed_control`** - Variable speed control on Raspberry Pi  
- **`rpi_direction_control`** - Forward/reverse direction control on Raspberry Pi
- **`rpi_brake_test`** - Motor braking functionality on Raspberry Pi
- **`rpi_encoder_monitor`** - Raspberry Pi motor with encoder feedback
- **`linux_basic_motor`** - Simple Linux GPIO motor control
- **`linux_speed_control`** - Variable speed control on Linux
- **`linux_direction_control`** - Forward/reverse direction control on Linux
- **`linux_brake_test`** - Motor braking functionality on Linux

### Running Examples

```bash
# Navigate to examples directory
cd example/

# Run Raspberry Pi examples with rppal feature
cargo run --features rppal --bin rpi_basic_motor
cargo run --features rppal --bin rpi_speed_control  
cargo run --features rppal --bin rpi_encoder_monitor

# Run Linux examples with linux-embedded-hal feature
cargo run --features linux-embedded-hal --bin linux_basic_motor
cargo run --features linux-embedded-hal --bin linux_speed_control
```

**Note**: Examples require appropriate hardware with proper GPIO connections.

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
    
    // Encoder support
    fn set_ppr(&mut self, ppr: i16) -> Result<bool, Self::Error>;
    fn check_ppr(&mut self) -> Result<(), Self::Error>;
    
    // Get Status 
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

### Motor Control Modes

- **Forward**: Positive speed values, normal rotation
- **Reverse**: Negative speed values, opposite rotation
- **Brake**: Active braking (both PWM channels high for dual PWM)
- **Coast**: Free spinning (all PWM channels low)

### Encoder Features

For motors with encoders:
- Quadrature encoder reading (A/B channels)
- Pulse counting with configurable PPR (Pulses Per Revolution)
- Encoder reset and target pulse positioning
- Real-time pulse monitoring

## Hardware Integration

### Platform Wrappers

This crate provides wrapper types to adapt platform-specific implementations to `embedded-hal` traits:

- `GpioWrapper` - Wraps GPIO pins implementing `OutputPin`
- `PwmWrapper` - Wraps PWM channels implementing `SetDutyCycle`

### Supported Platforms

- ✅ **Raspberry Pi** (via `rppal` crate - included wrappers)
- ✅ **Linux** (via `linux-embedded-hal` - optional feature)
- 🧪 **ESP32** (via `esp-hal` - bring your own wrappers) *Testing in progress*
- 🧪 **STM32** (via `stm32-hal` family - bring your own wrappers) *Testing in progress*
- 🧪 Any platform with `embedded-hal` support *Testing in progress*

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