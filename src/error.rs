#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotorDriverError {
    GpioError,
    PwmError,
    InvalidSpeed,
    InvalidConfiguration,
    NotInitialized,
    HardwareFault,
    OverCurrent,
    OverTemperature,
    UnderVoltage,
    OverVoltage,
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