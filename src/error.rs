//! Error types for motor driver operations.
//!
//! This module defines the comprehensive error types that can occur during motor driver
//! operations. The error types are designed to provide detailed information about
//! failures while maintaining compatibility with embedded environments.

/// Errors that can occur during motor driver operations.
///
/// This enum covers all possible error conditions that may arise when working with
/// motor drivers, from basic hardware communication errors to advanced fault conditions
/// detected by smart motor driver ICs.
///
/// ## Error Categories
///
/// - **Hardware Communication**: `GpioError`, `PwmError`, `CommunicationError`
/// - **Configuration Issues**: `InvalidSpeed`, `InvalidConfiguration`, `NotInitialized`  
/// - **Fault Conditions**: `HardwareFault`, `OverCurrent`, `OverTemperature`, etc.
///
/// ## Usage
///
/// ```rust
/// use motor_driver_hal::{MotorDriverError, MotorDriver};
///
/// # struct MyMotor;
/// # impl MotorDriver for MyMotor {
/// #   type Error = MotorDriverError;
/// #   fn initialize(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #   fn set_speed(&mut self, speed: i16) -> Result<(), Self::Error> {
/// #     if speed > 1000 { Err(MotorDriverError::InvalidSpeed) } else { Ok(()) }
/// #   }
/// #   fn set_direction(&mut self, forward: bool) -> Result<(), Self::Error> { Ok(()) }
/// #   fn stop(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #   fn brake(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #   fn enable(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #   fn disable(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #   fn get_speed(&self) -> Result<i16, Self::Error> { Ok(0) }
/// #   fn get_direction(&self) -> Result<bool, Self::Error> { Ok(true) }
/// #   fn get_current(&self) -> Result<f32, Self::Error> { Ok(0.0) }
/// #   fn get_voltage(&self) -> Result<f32, Self::Error> { Ok(0.0) }
/// #   fn get_temperature(&self) -> Result<f32, Self::Error> { Ok(0.0) }
/// #   fn get_fault_status(&self) -> Result<u8, Self::Error> { Ok(0) }
/// # }
///
/// let mut motor = MyMotor;
/// 
/// match motor.set_speed(1500) {
///     Ok(_) => println!("Speed set successfully"),
///     Err(MotorDriverError::InvalidSpeed) => println!("Speed too high!"),
///     Err(MotorDriverError::NotInitialized) => println!("Initialize motor first"),
///     Err(e) => println!("Other error: {}", e),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotorDriverError {
    /// GPIO pin control error.
    ///
    /// Occurs when setting or reading GPIO pin states fails. This can happen due to:
    /// - Pin already in use by another process
    /// - Insufficient permissions
    /// - Hardware malfunction
    GpioError,
    
    /// PWM channel control error.
    ///
    /// Occurs when PWM operations fail. Common causes include:
    /// - PWM channel not available
    /// - Invalid frequency or duty cycle settings
    /// - Hardware PWM peripheral issues
    PwmError,
    
    /// Invalid speed value provided.
    ///
    /// The requested speed exceeds the maximum duty cycle configured for the driver.
    /// Speed values must be within the range [-max_duty, +max_duty].
    InvalidSpeed,
    
    /// Invalid driver configuration.
    ///
    /// The motor driver configuration is invalid or incompatible with the hardware.
    /// This can occur during initialization or when changing operating modes.
    InvalidConfiguration,
    
    /// Driver not initialized.
    ///
    /// An operation was attempted on a motor driver that hasn't been initialized.
    /// Call `initialize()` before using any other driver functions.
    NotInitialized,
    
    /// Hardware fault detected.
    ///
    /// Generic hardware fault condition. This can indicate:
    /// - Motor driver IC internal fault
    /// - Sensor reading failure
    /// - Unspecified hardware malfunction
    HardwareFault,
    
    /// Over-current condition detected.
    ///
    /// The motor is drawing more current than the safe operating limit.
    /// This typically triggers automatic protection mechanisms.
    /// 
    /// **Note**: This feature requires current sensing hardware and is not yet implemented.
    /// TODO: Implement current monitoring support.
    OverCurrent,
    
    /// Over-temperature condition detected.
    ///
    /// The motor driver or motor temperature exceeds safe operating limits.
    /// Operation should be stopped immediately to prevent damage.
    ///
    /// **Note**: This feature requires temperature sensing hardware and is not yet implemented.
    /// TODO: Implement temperature monitoring support.
    OverTemperature,
    
    /// Under-voltage condition detected.
    ///
    /// The supply voltage has dropped below the minimum operating threshold.
    /// Motor performance may be degraded or operation may be unreliable.
    ///
    /// **Note**: This feature requires voltage monitoring hardware and is not yet implemented.
    /// TODO: Implement voltage monitoring support.
    UnderVoltage,
    
    /// Over-voltage condition detected.
    ///
    /// The supply voltage exceeds the maximum safe operating threshold.
    /// Continued operation may damage the motor driver or motor.
    ///
    /// **Note**: This feature requires voltage monitoring hardware and is not yet implemented.
    /// TODO: Implement voltage monitoring support.
    OverVoltage,
    
    /// Communication error with motor driver IC.
    ///
    /// Failed to communicate with a smart motor driver IC over I2C, SPI, or other
    /// communication protocol. This can indicate:
    /// - Bus communication failure
    /// - Device not responding
    /// - Protocol errors
    ///
    /// **Note**: This feature is for advanced motor driver ICs and is not yet implemented.
    /// TODO: Implement smart motor driver communication support.
    CommunicationError,
}

impl core::fmt::Display for MotorDriverError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MotorDriverError::GpioError => write!(f, "GPIO control error"),
            MotorDriverError::PwmError => write!(f, "PWM control error"),
            MotorDriverError::InvalidSpeed => write!(f, "Invalid speed value"),
            MotorDriverError::InvalidConfiguration => write!(f, "Invalid configuration"),
            MotorDriverError::NotInitialized => write!(f, "Driver not initialized"),
            MotorDriverError::HardwareFault => write!(f, "Hardware fault detected"),
            MotorDriverError::OverCurrent => write!(f, "Over current condition"),
            MotorDriverError::OverTemperature => write!(f, "Over temperature condition"),
            MotorDriverError::UnderVoltage => write!(f, "Under voltage condition"),
            MotorDriverError::OverVoltage => write!(f, "Over voltage condition"),
            MotorDriverError::CommunicationError => write!(f, "Communication error"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MotorDriverError {}