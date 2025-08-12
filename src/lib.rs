//! # Motor Driver HAL
//!
//! A hardware abstraction layer (HAL) for motor drivers built on top of `embedded-hal` traits.
//! This crate provides a generic, platform-independent interface for controlling various types
//! of motor drivers commonly used in embedded systems and robotics applications.
//!
//! ## Features
//!
//! - **Platform Independent**: Built on `embedded-hal` traits for maximum portability
//! - **Multiple Motor Types**: Support for H-bridge, single direction, and brushless motors
//! - **Flexible Configuration**: Configurable PWM channels, enable pins, and control modes
//! - **Safety First**: Built-in protections and error handling
//! - **no_std Support**: Works in bare-metal embedded environments
//!
//! ## Supported Motor Driver Types
//!
//! - **H-Bridge Drivers**: Bidirectional DC motor control with optional brake functionality
//! - **Single Direction Drivers**: Simple unidirectional motor control
//! - **Dual H-Bridge**: Independent control of two motors
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use motor_driver_hal::{HBridgeMotorDriver, MotorDriver};
//! use embedded_hal::digital::OutputPin;
//! use embedded_hal::pwm::SetDutyCycle;
//!
//! // Your platform-specific GPIO and PWM implementations
//! # struct MyOutputPin;
//! # impl embedded_hal::digital::ErrorType for MyOutputPin {
//! #     type Error = ();
//! # }
//! # impl OutputPin for MyOutputPin {
//! #     fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! # }
//! # struct MyPwm;
//! # impl embedded_hal::pwm::ErrorType for MyPwm {
//! #     type Error = ();
//! # }
//! # impl SetDutyCycle for MyPwm {
//! #     fn max_duty_cycle(&self) -> u16 { 1000 }
//! #     fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> { Ok(()) }
//! # }
//!
//! // Create motor driver instance
//! let enable_pin = MyOutputPin;
//! let pwm_channel = MyPwm;
//!
//! let mut motor = HBridgeMotorDriver::new_single_enable_single_pwm(
//!     enable_pin,
//!     pwm_channel,
//!     1000  // max duty cycle
//! );
//!
//! // Initialize and use the motor
//! motor.initialize();
//! motor.enable();
//! motor.set_speed(500);  // 50% forward speed
//! motor.set_speed(-300); // 30% reverse speed
//! motor.brake();
//! motor.stop();
//! ```
//!
//! ## Architecture
//!
//! The crate is organized into several modules:
//!
//! - [`driver`]: Core motor driver implementations
//! - [`error`]: Error types and handling
//! - [`wrapper`]: High-level wrapper with builder pattern
//!
//! ## Hardware Integration
//!
//! To use this crate with your specific hardware platform, you'll need to:
//!
//! 1. Implement the `embedded-hal` traits for your GPIO and PWM peripherals
//! 2. Create wrapper types that adapt your platform's types to the HAL traits
//! 3. Use the motor driver with your wrapped types
//!
//! See the examples directory for platform-specific implementations.
//!
//! ## Error Handling
//!
//! All operations return `Result` types with detailed error information.
//! The [`MotorDriverError`] enum provides comprehensive error reporting for
//! debugging and runtime error handling.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod driver;
pub mod error;
pub mod wrapper;

pub use driver::HBridgeMotorDriver;
pub use error::MotorDriverError;
pub use wrapper::{MotorDriverWrapper, MotorDriverBuilder, EnablePins, PwmChannels, MotorDirection};

/// Core trait defining the interface for all motor drivers.
///
/// This trait provides a unified interface for controlling different types of motor drivers.
/// All motor driver implementations in this crate implement this trait, ensuring consistent
/// API across different hardware configurations.
///
/// ## Speed Control
///
/// Speed values are represented as signed 16-bit integers:
/// - Positive values: Forward direction
/// - Negative values: Reverse direction  
/// - Zero: Stopped
/// - Range depends on the specific driver's `max_duty` setting
///
/// ## State Management
///
/// Motors have several states that must be managed properly:
/// 1. **Uninitialized**: Fresh driver instance, not ready for use
/// 2. **Initialized**: Driver configured and ready, but motor disabled
/// 3. **Enabled**: Motor powered and ready to move
/// 4. **Disabled**: Motor power cut, safe state
///
/// ## Example Usage
///
/// ```rust,no_run
/// use motor_driver_hal::MotorDriver;
/// # struct MyMotor;
/// # impl MotorDriver for MyMotor {
/// #     type Error = ();
/// #     fn initialize(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     fn set_speed(&mut self, speed: i16) -> Result<(), Self::Error> { Ok(()) }
/// #     fn set_direction(&mut self, forward: bool) -> Result<(), Self::Error> { Ok(()) }
/// #     fn stop(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     fn brake(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     fn enable(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     fn disable(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     fn get_speed(&self) -> Result<i16, Self::Error> { Ok(0) }
/// #     fn get_direction(&self) -> Result<bool, Self::Error> { Ok(true) }
/// #     fn get_current(&self) -> Result<f32, Self::Error> { Ok(0.0) }
/// #     fn get_voltage(&self) -> Result<f32, Self::Error> { Ok(0.0) }
/// #     fn get_temperature(&self) -> Result<f32, Self::Error> { Ok(0.0) }
/// #     fn get_fault_status(&self) -> Result<u8, Self::Error> { Ok(0) }
/// # }
///
/// let mut motor = MyMotor;
///
/// // Proper initialization sequence
/// motor.initialize()?;
/// motor.enable()?;
///
/// // Control motor
/// motor.set_speed(750)?;   // Forward at 75% speed
/// motor.set_speed(-250)?;  // Reverse at 25% speed
/// motor.brake()?;          // Emergency stop
///
/// // Cleanup
/// motor.disable()?;
/// # Ok::<(), ()>(())
/// ```
pub trait MotorDriver {
    /// The error type returned by driver operations.
    type Error;

    /// Initialize the motor driver hardware.
    ///
    /// This method configures the motor driver hardware and prepares it for operation.
    /// Must be called before any other motor operations. After successful initialization,
    /// the motor will be in a disabled state and must be enabled before movement.
    ///
    /// # Errors
    ///
    /// Returns an error if hardware initialization fails, such as:
    /// - GPIO configuration errors
    /// - PWM setup failures
    /// - Hardware communication issues
    fn initialize(&mut self) -> Result<(), Self::Error>;

    /// Set motor speed and direction.
    ///
    /// Controls both the speed and direction of the motor using a signed integer value.
    /// The motor must be initialized and enabled before calling this method.
    ///
    /// # Arguments
    ///
    /// * `speed` - Signed speed value where:
    ///   - Positive values: Forward direction (0 to max_duty)
    ///   - Negative values: Reverse direction (-max_duty to 0)
    ///   - Zero: Stop the motor
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    /// - `NotEnabled`: Motor not enabled
    /// - `InvalidSpeed`: Speed exceeds maximum duty cycle
    /// - Hardware errors from PWM or GPIO operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use motor_driver_hal::MotorDriver;
    /// # fn example(motor: &mut impl MotorDriver) -> Result<(), Box<dyn std::error::Error>> {
    /// motor.set_speed(500)?;   // Forward at 50% (assuming max_duty = 1000)
    /// motor.set_speed(-250)?;  // Reverse at 25%
    /// motor.set_speed(0)?;     // Stop
    /// # Ok(())
    /// # }
    /// ```
    fn set_speed(&mut self, speed: i16) -> Result<(), Self::Error>;

    /// Set motor direction explicitly.
    ///
    /// Changes the motor direction without affecting the current speed magnitude.
    /// This is an alternative to using signed values in `set_speed()`.
    ///
    /// # Arguments
    ///
    /// * `forward` - `true` for forward direction, `false` for reverse
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    /// - Hardware errors from GPIO operations
    fn set_direction(&mut self, forward: bool) -> Result<(), Self::Error>;

    /// Stop the motor by coasting to a halt.
    ///
    /// Sets the motor speed to zero and allows it to coast to a stop.
    /// This is a "soft" stop that doesn't apply active braking.
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    /// - Hardware errors from PWM operations
    fn stop(&mut self) -> Result<(), Self::Error>;

    /// Apply active braking to stop the motor immediately.
    ///
    /// Applies electrical braking to stop the motor as quickly as possible.
    /// This is a "hard" stop that actively resists motor movement.
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    /// - `BrakeNotSupported`: Hardware doesn't support braking
    /// - Hardware errors from GPIO or PWM operations
    fn brake(&mut self) -> Result<(), Self::Error>;

    /// Enable motor power and control.
    ///
    /// Turns on the motor driver, allowing the motor to be controlled.
    /// The motor must be initialized before enabling.
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    /// - Hardware errors from GPIO operations
    fn enable(&mut self) -> Result<(), Self::Error>;

    /// Disable motor power.
    ///
    /// Turns off the motor driver, cutting power to the motor for safety.
    /// The motor will stop immediately when disabled.
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    /// - Hardware errors from GPIO operations
    fn disable(&mut self) -> Result<(), Self::Error>;

    /// Get the current motor speed setting.
    ///
    /// Returns the last speed value set via `set_speed()`. This is the
    /// commanded speed, not necessarily the actual motor speed.
    ///
    /// # Returns
    ///
    /// Signed speed value (positive = forward, negative = reverse)
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    fn get_speed(&self) -> Result<i16, Self::Error>;

    /// Get the current motor direction setting.
    ///
    /// # Returns
    ///
    /// `true` for forward direction, `false` for reverse
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    fn get_direction(&self) -> Result<bool, Self::Error>;

    /// Get motor current consumption.
    ///
    /// # Returns
    ///
    /// Current consumption in Amperes
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    /// - `NotSupported`: Hardware doesn't support current sensing
    ///
    /// # Note
    ///
    /// This feature is not yet implemented and will return `NotSupported`.
    /// TODO: Implement current sensing support.
    fn get_current(&self) -> Result<f32, Self::Error>;

    /// Get motor supply voltage.
    ///
    /// # Returns
    ///
    /// Supply voltage in Volts
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    /// - `NotSupported`: Hardware doesn't support voltage sensing
    ///
    /// # Note
    ///
    /// This feature is not yet implemented and will return `NotSupported`.
    /// TODO: Implement voltage sensing support.
    fn get_voltage(&self) -> Result<f32, Self::Error>;

    /// Get motor driver temperature.
    ///
    /// # Returns
    ///
    /// Temperature in degrees Celsius
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    /// - `NotSupported`: Hardware doesn't support temperature sensing
    ///
    /// # Note
    ///
    /// This feature is not yet implemented and will return `NotSupported`.
    /// TODO: Implement temperature sensing support.
    fn get_temperature(&self) -> Result<f32, Self::Error>;

    /// Get fault status from motor driver.
    ///
    /// Returns a bitmask indicating various fault conditions.
    /// The specific meaning of bits depends on the hardware implementation.
    ///
    /// # Returns
    ///
    /// Fault status bitmask where each bit represents a different fault condition
    ///
    /// # Errors
    ///
    /// - `NotInitialized`: Motor driver not initialized
    /// - `NotSupported`: Hardware doesn't support fault reporting
    ///
    /// # Note
    ///
    /// This feature is not yet implemented and will return `NotSupported`.
    /// TODO: Implement fault status reporting.
    fn get_fault_status(&self) -> Result<u8, Self::Error>;
}