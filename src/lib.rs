
//! # Motor Driver HAL
//!
//! A hardware abstraction layer (HAL) for controlling motor drivers with H-bridge configurations.
//! This crate provides a unified interface for controlling motors across different embedded platforms
//! including Raspberry Pi (via rppal) and Linux systems (via linux-embedded-hal).
//!
//! ## Features
//!
//! - H-bridge motor control with single or dual PWM channels
//! - Single or dual enable pin configurations
//! - Encoder support for position and speed feedback
//! - Cross-platform support (Raspberry Pi, Linux embedded systems)
//! - Builder pattern for easy configuration
//! - No-std compatible (when std feature is disabled)
//!
//! ## Example
//!
//! ```rust
//! use motor_driver_hal::{MotorDriver, MotorDriverWrapper};
//!
//! // Create a motor driver using the builder pattern
//! let mut motor = MotorDriverWrapper::builder()
//!     .with_single_enable(enable_pin)
//!     .with_single_pwm(pwm_channel)
//!     .with_max_duty(1000)
//!     .build();
//!
//! // Initialize and control the motor
//! motor.initialize()?;
//! motor.enable()?;
//! motor.set_speed(500)?; // Set speed to 50% forward
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub mod driver;
pub mod error;
pub mod wrapper;

pub use driver::{HBridgeMotorDriver, NoEncoder};
pub use error::MotorDriverError;
pub use wrapper::{MotorDriverWrapper, MotorDriverBuilder, EnablePins, PwmChannels, MotorDirection};

#[cfg(feature = "rppal")]
pub use wrapper::rppal::{GpioWrapper, PwmWrapper};

/// Core trait defining the interface for motor driver implementations.
/// 
/// This trait provides a unified API for controlling motors across different hardware
/// platforms and configurations. Implementations should handle hardware-specific
/// details while providing consistent behavior through this interface.
/// 
/// # Type Parameters
/// 
/// * `Error` - The error type returned by driver operations
/// 
/// # Example
/// 
/// ```rust
/// use motor_driver_hal::MotorDriver;
/// 
/// fn control_motor<T: MotorDriver>(motor: &mut T) -> Result<(), T::Error> {
///     motor.initialize()?;
///     motor.enable()?;
///     motor.set_speed(250)?;
///     Ok(())
/// }
/// ```
pub trait MotorDriver {
    /// The error type returned by this driver's operations.
    type Error;
    
    /// Initializes the motor driver hardware.
    /// 
    /// This method should be called before any other operations. It sets up
    /// the hardware in a safe initial state with motors disabled and PWM at zero.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` if initialization succeeds
    /// * `Err(Self::Error)` if hardware initialization fails
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.initialize()?;
    /// ```
    fn initialize(&mut self) -> Result<(), Self::Error>;
    
    /// Sets the motor speed and direction.
    /// 
    /// # Arguments
    /// 
    /// * `speed` - Motor speed value. Positive values indicate forward direction,
    ///   negative values indicate reverse direction. The magnitude should not exceed
    ///   the configured maximum duty cycle.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` if speed is set successfully
    /// * `Err(Self::Error)` if speed value is invalid or hardware error occurs
    /// 
    /// # Errors
    /// 
    /// Returns error if driver is not initialized or speed exceeds maximum duty cycle.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.set_speed(500)?;  // 50% forward
    /// motor.set_speed(-300)?; // 30% reverse
    /// ```
    fn set_speed(&mut self, speed: i16) -> Result<(), Self::Error>;
    
    /// Sets the motor direction without changing speed magnitude.
    /// 
    /// # Arguments
    /// 
    /// * `forward` - `true` for forward direction, `false` for reverse
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` if direction is set successfully
    /// * `Err(Self::Error)` if hardware error occurs
    /// 
    /// # Errors
    /// 
    /// Returns error if driver is not initialized.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.set_direction(true)?;  // Forward
    /// motor.set_direction(false)?; // Reverse
    /// ```
    fn set_direction(&mut self, forward: bool) -> Result<(), Self::Error>;
    
    /// Stops the motor by setting PWM to zero (coast stop).
    /// 
    /// This allows the motor to coast to a stop naturally without active braking.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` if motor stops successfully
    /// * `Err(Self::Error)` if hardware error occurs
    /// 
    /// # Errors
    /// 
    /// Returns error if driver is not initialized.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.stop()?; // Coast to stop
    /// ```
    fn stop(&mut self) -> Result<(), Self::Error>;
    
    /// Applies active braking to the motor.
    /// 
    /// This applies maximum PWM to both directions simultaneously (for dual PWM)
    /// or sets PWM to zero (for single PWM), providing active braking force.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` if braking is applied successfully
    /// * `Err(Self::Error)` if hardware error occurs
    /// 
    /// # Errors
    /// 
    /// Returns error if driver is not initialized.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.brake()?; // Apply active braking
    /// ```
    fn brake(&mut self) -> Result<(), Self::Error>;
    
    /// Enables the motor driver.
    /// 
    /// Sets enable pins high to allow motor operation. The motor driver
    /// must be enabled before it can control the motor.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` if driver is enabled successfully
    /// * `Err(Self::Error)` if hardware error occurs
    /// 
    /// # Errors
    /// 
    /// Returns error if driver is not initialized.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.enable()?;
    /// ```
    fn enable(&mut self) -> Result<(), Self::Error>;
    
    /// Disables the motor driver.
    /// 
    /// Sets enable pins low to prevent motor operation. This is a safety
    /// feature that can be used to quickly disable motor control.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` if driver is disabled successfully
    /// * `Err(Self::Error)` if hardware error occurs
    /// 
    /// # Errors
    /// 
    /// Returns error if driver is not initialized.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.disable()?;
    /// ```
    fn disable(&mut self) -> Result<(), Self::Error>;
    
    /// Checks if the encoder pulse count matches the target position.
    /// 
    /// This method validates that the motor has reached the desired position
    /// based on encoder feedback and configured pulses per revolution (PPR).
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` if position matches target
    /// * `Err(Self::Error)` if position doesn't match or encoder error occurs
    /// 
    /// # Errors
    /// 
    /// Returns error if PPR is not configured or encoder reading fails.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.set_target_pulse(1000);
    /// motor.check_ppr()?; // Verify position
    /// ```
    fn check_ppr(&mut self) -> Result<(), Self::Error>;
    
    /// Sets the pulses per revolution for encoder calculations.
    /// 
    /// # Arguments
    /// 
    /// * `ppr` - Number of encoder pulses per complete motor revolution
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` if PPR is set successfully
    /// * `Err(Self::Error)` if PPR value is invalid or driver not initialized
    /// 
    /// # Errors
    /// 
    /// Returns error if driver is not initialized or PPR value is invalid (≤ 0).
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.set_ppr(1000)?; // 1000 pulses per revolution
    /// ```
    fn set_ppr(&mut self, ppr: i16) -> Result<bool, Self::Error>;
    
    /// Gets the current motor speed setting.
    /// 
    /// # Returns
    /// 
    /// * `Ok(speed)` - Current speed value (positive = forward, negative = reverse)
    /// * `Err(Self::Error)` if driver is not initialized
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let current_speed = motor.get_speed()?;
    /// println!("Motor speed: {}", current_speed);
    /// ```
    fn get_speed(&self) -> Result<i16, Self::Error>;
    
    /// Gets the current motor direction.
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` if motor is set to forward direction
    /// * `Ok(false)` if motor is set to reverse direction
    /// * `Err(Self::Error)` if driver is not initialized
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let is_forward = motor.get_direction()?;
    /// ```
    fn get_direction(&self) -> Result<bool, Self::Error>;
    
    /// Gets the current motor current consumption.
    /// 
    /// # Returns
    /// 
    /// * `Ok(current)` - Current consumption in amperes
    /// * `Err(Self::Error)` if current sensing is not supported or hardware error
    /// 
    /// # Errors
    /// 
    /// May return `HardwareFault` if current sensing is not implemented.
    fn get_current(&self) -> Result<f32, Self::Error>;
    
    /// Gets the current motor supply voltage.
    /// 
    /// # Returns
    /// 
    /// * `Ok(voltage)` - Supply voltage in volts
    /// * `Err(Self::Error)` if voltage sensing is not supported or hardware error
    /// 
    /// # Errors
    /// 
    /// May return `HardwareFault` if voltage sensing is not implemented.
    fn get_voltage(&self) -> Result<f32, Self::Error>;
    
    /// Gets the current motor driver temperature.
    /// 
    /// # Returns
    /// 
    /// * `Ok(temperature)` - Temperature in degrees Celsius
    /// * `Err(Self::Error)` if temperature sensing is not supported or hardware error
    /// 
    /// # Errors
    /// 
    /// May return `HardwareFault` if temperature sensing is not implemented.
    fn get_temperature(&self) -> Result<f32, Self::Error>;
    
    /// Gets the current fault status of the motor driver.
    /// 
    /// # Returns
    /// 
    /// * `Ok(status)` - Fault status byte (0 = no faults)
    /// * `Err(Self::Error)` if driver is not initialized or hardware error
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let fault_status = motor.get_fault_status()?;
    /// if fault_status != 0 {
    ///     println!("Motor fault detected: {}", fault_status);
    /// }
    /// ```
    fn get_fault_status(&self) -> Result<u8, Self::Error>;
}