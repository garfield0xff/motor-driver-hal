/// Error types that can occur during motor driver operations.
/// 
/// This enum represents all possible error conditions that can arise when
/// using motor driver implementations. Errors are categorized by their source
/// and can help diagnose hardware or configuration issues.
/// 
/// # Example
/// 
/// ```rust
/// use motor_driver_hal::MotorDriverError;
/// 
/// match motor.set_speed(1500) {
///     Ok(()) => println!("Speed set successfully"),
///     Err(MotorDriverError::InvalidSpeed) => println!("Speed value too high"),
///     Err(MotorDriverError::NotInitialized) => println!("Driver not initialized"),
///     Err(e) => println!("Other error: {}", e),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotorDriverError {
    /// GPIO pin control operation failed.
    /// 
    /// This error occurs when setting GPIO pin states (high/low) fails,
    /// typically due to hardware issues or incorrect pin configuration.
    GpioError,
    
    /// PWM channel operation failed.
    /// 
    /// This error occurs when PWM duty cycle setting fails, which can
    /// happen due to invalid duty cycle values or PWM hardware issues.
    PwmError,
    
    /// Invalid speed value provided.
    /// 
    /// This error occurs when the speed value exceeds the configured
    /// maximum duty cycle or is otherwise invalid for the motor configuration.
    InvalidSpeed,
    
    /// Invalid configuration detected.
    /// 
    /// This error occurs when the motor driver configuration is incomplete
    /// or contains conflicting settings that prevent proper operation.
    InvalidConfiguration,
    
    /// Motor driver has not been initialized.
    /// 
    /// This error occurs when attempting to perform operations before
    /// calling the `initialize()` method.
    NotInitialized,
    
    /// Hardware fault detected.
    /// 
    /// This error indicates a general hardware fault that prevents normal
    /// operation, such as encoder reading failures or unsupported operations.
    HardwareFault,
    
    /// Motor current consumption exceeds safe limits.
    /// 
    /// This error occurs when the motor draws more current than the
    /// configured safe operating limits.
    OverCurrent,
    
    /// Motor driver temperature exceeds safe operating limits.
    /// 
    /// This error occurs when the motor driver's temperature sensor
    /// detects overheating conditions.
    OverTemperature,
    
    /// Supply voltage is below minimum operating requirements.
    /// 
    /// This error occurs when the motor driver's supply voltage
    /// drops below the minimum required for safe operation.
    UnderVoltage,
    
    /// Supply voltage exceeds maximum operating limits.
    /// 
    /// This error occurs when the motor driver's supply voltage
    /// exceeds the maximum safe operating voltage.
    OverVoltage,
    
    /// Communication with motor driver hardware failed.
    /// 
    /// This error occurs when communication protocols (I2C, SPI, UART)
    /// fail to communicate with smart motor drivers.
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