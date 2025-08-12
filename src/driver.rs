//! Core motor driver implementations.
//!
//! This module provides concrete implementations of motor drivers for common hardware
//! configurations. The primary implementation is [`HBridgeMotorDriver`] which supports
//! various H-bridge motor driver chips and configurations.

use crate::{MotorDriver, MotorDriverError};
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;

/// H-bridge motor driver implementation.
///
/// This driver supports various H-bridge configurations commonly used with DC motors.
/// It can work with single or dual PWM channels and single or dual enable pins,
/// making it compatible with most H-bridge motor driver ICs.
///
/// ## Supported Configurations
///
/// - **Single Enable + Single PWM**: Simple H-bridge with direction control via enable pins
/// - **Dual Enable + Dual PWM**: Advanced H-bridge with independent control of both sides
///
/// ## Hardware Requirements
///
/// - Enable pins must implement [`OutputPin`] trait
/// - PWM channels must implement [`SetDutyCycle`] trait
/// - All pins must be properly connected to the motor driver IC
///
/// ## Type Parameters
///
/// - `E1`: Type of the primary enable pin
/// - `E2`: Type of the secondary enable pin (optional)
/// - `P1`: Type of the primary PWM channel
/// - `P2`: Type of the secondary PWM channel (optional)
///
/// ## Example
///
/// ```rust,no_run
/// use motor_driver_hal::{HBridgeMotorDriver, MotorDriver};
/// # struct MyPin; struct MyPwm;
/// # impl embedded_hal::digital::ErrorType for MyPin { type Error = (); }
/// # impl embedded_hal::digital::OutputPin for MyPin {
/// #   fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #   fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// # }
/// # impl embedded_hal::pwm::ErrorType for MyPwm { type Error = (); }
/// # impl embedded_hal::pwm::SetDutyCycle for MyPwm {
/// #   fn max_duty_cycle(&self) -> u16 { 1000 }
/// #   fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> { Ok(()) }
/// # }
///
/// let enable = MyPin;
/// let pwm = MyPwm;
///
/// let mut motor = HBridgeMotorDriver::single_pwm(enable, pwm, 1000);
/// motor.initialize()?;
/// motor.enable()?;
/// motor.set_speed(500)?; // 50% speed forward
/// # Ok::<(), motor_driver_hal::MotorDriverError>(())
/// ```
pub struct HBridgeMotorDriver<E1, E2, P1, P2> {
    /// Primary enable pin (always present)
    enable1: E1,
    /// Secondary enable pin (dual enable configuration)
    enable2: Option<E2>,
    /// Primary PWM channel (always present)
    pwm1: P1,
    /// Secondary PWM channel (dual PWM configuration)
    pwm2: Option<P2>,
    /// Maximum duty cycle value for PWM channels
    max_duty: u16,
    /// Current commanded speed (-max_duty to +max_duty)
    current_speed: i16,
    /// Current direction (true = forward, false = reverse)
    direction: bool,
    /// Whether the driver has been initialized
    initialized: bool,
}

impl<E1, E2, P1, P2> HBridgeMotorDriver<E1, E2, P1, P2>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
{
    /// Create a new H-bridge motor driver with single PWM configuration.
    ///
    /// This configuration uses one enable pin and one PWM channel. Direction control
    /// is typically handled by the PWM duty cycle polarity or additional logic.
    ///
    /// # Arguments
    ///
    /// * `enable` - GPIO pin to enable/disable the motor driver
    /// * `pwm` - PWM channel for speed control
    /// * `max_duty` - Maximum duty cycle value (typically 1000 or 255)
    ///
    /// # Returns
    ///
    /// A new uninitialized motor driver instance
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use motor_driver_hal::HBridgeMotorDriver;
    /// # struct MyPin; struct MyPwm;
    /// # impl embedded_hal::digital::ErrorType for MyPin { type Error = (); }
    /// # impl embedded_hal::digital::OutputPin for MyPin {
    /// #   fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #   fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # impl embedded_hal::pwm::ErrorType for MyPwm { type Error = (); }
    /// # impl embedded_hal::pwm::SetDutyCycle for MyPwm {
    /// #   fn max_duty_cycle(&self) -> u16 { 1000 }
    /// #   fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    ///
    /// let enable_pin = MyPin;
    /// let pwm_channel = MyPwm;
    ///
    /// let motor = HBridgeMotorDriver::single_pwm(enable_pin, pwm_channel, 1000);
    /// ```
    pub fn single_pwm(enable: E1, pwm: P1, max_duty: u16) -> Self {
        Self {
            enable1: enable,
            enable2: None,
            pwm1: pwm,
            pwm2: None,
            max_duty,
            current_speed: 0,
            direction: true,
            initialized: false,
        }
    }

    /// Create a new H-bridge motor driver with dual PWM configuration.
    ///
    /// This configuration uses two enable pins and two PWM channels, providing
    /// independent control of both sides of the H-bridge. This allows for more
    /// precise control and better braking capabilities.
    ///
    /// # Arguments
    ///
    /// * `enable1` - GPIO pin to enable the first half-bridge
    /// * `enable2` - GPIO pin to enable the second half-bridge
    /// * `pwm1` - PWM channel for the first half-bridge
    /// * `pwm2` - PWM channel for the second half-bridge
    /// * `max_duty` - Maximum duty cycle value (typically 1000 or 255)
    ///
    /// # Returns
    ///
    /// A new uninitialized motor driver instance
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use motor_driver_hal::HBridgeMotorDriver;
    /// # struct MyPin; struct MyPwm;
    /// # impl embedded_hal::digital::ErrorType for MyPin { type Error = (); }
    /// # impl embedded_hal::digital::OutputPin for MyPin {
    /// #   fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #   fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # impl embedded_hal::pwm::ErrorType for MyPwm { type Error = (); }
    /// # impl embedded_hal::pwm::SetDutyCycle for MyPwm {
    /// #   fn max_duty_cycle(&self) -> u16 { 1000 }
    /// #   fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    ///
    /// let enable1 = MyPin;
    /// let enable2 = MyPin;
    /// let pwm1 = MyPwm;
    /// let pwm2 = MyPwm;
    ///
    /// let motor = HBridgeMotorDriver::dual_pwm(enable1, enable2, pwm1, pwm2, 1000);
    /// ```
    pub fn dual_pwm(enable1: E1, enable2: E2, pwm1: P1, pwm2: P2, max_duty: u16) -> Self {
        Self {
            enable1,
            enable2: Some(enable2),
            pwm1,
            pwm2: Some(pwm2),
            max_duty,
            current_speed: 0,
            direction: true,
            initialized: false,
        }
    }

    /// Update PWM channels based on current speed and direction.
    ///
    /// This internal method handles the logic for setting PWM duty cycles
    /// on the appropriate channels based on the motor's current speed and direction.
    ///
    /// For dual PWM configuration:
    /// - Forward: PWM1 active, PWM2 off
    /// - Reverse: PWM1 off, PWM2 active
    ///
    /// For single PWM configuration:
    /// - Speed controls PWM1 duty cycle
    /// - Direction handled externally or by PWM polarity
    fn update_pwm(&mut self) -> Result<(), MotorDriverError> {
        let duty = if self.current_speed < 0 {
            (-self.current_speed as u16).min(self.max_duty)
        } else {
            (self.current_speed as u16).min(self.max_duty)
        };

        match (&mut self.pwm2, self.direction) {
            (Some(pwm2), true) => {
                self.pwm1.set_duty_cycle(duty).map_err(|_| MotorDriverError::PwmError)?;
                pwm2.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
            }
            (Some(pwm2), false) => {
                self.pwm1.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
                pwm2.set_duty_cycle(duty).map_err(|_| MotorDriverError::PwmError)?;
            }
            (None, _) => {
                self.pwm1.set_duty_cycle(duty).map_err(|_| MotorDriverError::PwmError)?;
            }
        }
        Ok(())
    }
}

impl<E1, E2, P1, P2> MotorDriver for HBridgeMotorDriver<E1, E2, P1, P2>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
{
    type Error = MotorDriverError;

    fn initialize(&mut self) -> Result<(), Self::Error> {
        self.enable1.set_low().map_err(|_| MotorDriverError::GpioError)?;
        if let Some(ref mut enable2) = self.enable2 {
            enable2.set_low().map_err(|_| MotorDriverError::GpioError)?;
        }
        
        self.pwm1.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
        if let Some(ref mut pwm2) = self.pwm2 {
            pwm2.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
        }
        
        self.initialized = true;
        Ok(())
    }

    fn set_speed(&mut self, speed: i16) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        if speed.unsigned_abs() > self.max_duty {
            return Err(MotorDriverError::InvalidSpeed);
        }
        
        self.current_speed = speed;
        self.direction = speed >= 0;
        
        self.update_pwm()
    }

    fn set_direction(&mut self, forward: bool) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.direction = forward;
        self.update_pwm()
    }

    fn stop(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.current_speed = 0;
        self.pwm1.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
        if let Some(ref mut pwm2) = self.pwm2 {
            pwm2.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
        }
        Ok(())
    }

    fn brake(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.current_speed = 0;
        if let Some(ref mut pwm2) = self.pwm2 {
            self.pwm1.set_duty_cycle(self.max_duty).map_err(|_| MotorDriverError::PwmError)?;
            pwm2.set_duty_cycle(self.max_duty).map_err(|_| MotorDriverError::PwmError)?;
        } else {
            self.pwm1.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
        }
        Ok(())
    }

    fn enable(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.enable1.set_high().map_err(|_| MotorDriverError::GpioError)?;
        if let Some(ref mut enable2) = self.enable2 {
            enable2.set_high().map_err(|_| MotorDriverError::GpioError)?;
        }
        Ok(())
    }

    fn disable(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.enable1.set_low().map_err(|_| MotorDriverError::GpioError)?;
        if let Some(ref mut enable2) = self.enable2 {
            enable2.set_low().map_err(|_| MotorDriverError::GpioError)?;
        }
        Ok(())
    }

    fn get_speed(&self) -> Result<i16, Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        Ok(self.current_speed)
    }

    fn get_direction(&self) -> Result<bool, Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        Ok(self.direction)
    }

    fn get_current(&self) -> Result<f32, Self::Error> {
        // TODO: Implement current sensing support
        // This would require ADC integration and current sensing hardware
        Err(MotorDriverError::HardwareFault)
    }

    fn get_voltage(&self) -> Result<f32, Self::Error> {
        // TODO: Implement voltage sensing support  
        // This would require ADC integration and voltage divider circuits
        Err(MotorDriverError::HardwareFault)
    }

    fn get_temperature(&self) -> Result<f32, Self::Error> {
        // TODO: Implement temperature sensing support
        // This would require temperature sensor integration (thermistor, etc.)
        Err(MotorDriverError::HardwareFault)
    }

    fn get_fault_status(&self) -> Result<u8, Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        Ok(0)
    }
}