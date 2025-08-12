//! High-level motor driver wrapper with builder pattern.
//!
//! This module provides a flexible wrapper around motor drivers that supports
//! various hardware configurations through a builder pattern. It's designed to
//! handle different combinations of enable pins and PWM channels commonly found
//! in motor driver circuits.

use crate::{MotorDriver, MotorDriverError};
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;

/// Configuration for motor enable pins.
///
/// Different motor driver ICs and circuits use different enable pin configurations:
/// - Some have no enable pins (always enabled)
/// - Some have a single enable pin for the entire driver
/// - Some have separate enable pins for each half-bridge
///
/// # Examples
///
/// ```rust
/// use motor_driver_hal::EnablePins;
/// # struct MyPin;
/// # impl embedded_hal::digital::ErrorType for MyPin { type Error = (); }
/// # impl embedded_hal::digital::OutputPin for MyPin {
/// #   fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #   fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// # }
///
/// // No enable pins - driver always active
/// let no_enable: EnablePins<MyPin, MyPin> = EnablePins::None;
///
/// // Single enable pin for entire driver
/// let single_enable = EnablePins::Single(MyPin);
///
/// // Dual enable pins for independent control
/// let dual_enable = EnablePins::Dual(MyPin, MyPin);
/// ```
pub enum EnablePins<E1, E2> {
    /// No enable pins - motor driver is always enabled
    None,
    /// Single enable pin controlling the entire motor driver
    Single(E1),
    /// Dual enable pins for independent control of each half-bridge
    Dual(E1, E2),
}

/// Configuration for PWM channels.
///
/// Motor drivers can use different PWM control schemes:
/// - No PWM: Simple on/off control (digital output only)
/// - Single PWM: Sign-magnitude control (one PWM + direction pin)
/// - Dual PWM: Independent PWM for each motor direction
///
/// # PWM Control Methods
///
/// ## Single PWM (Sign-Magnitude)
/// - One PWM channel controls speed
/// - Direction controlled by enable pins or separate direction pin
/// - Common in simple motor drivers
///
/// ## Dual PWM (Locked Anti-phase)
/// - Two PWM channels, one for each direction
/// - Forward: PWM1 active, PWM2 off
/// - Reverse: PWM1 off, PWM2 active
/// - Allows for regenerative braking
///
/// # Examples
///
/// ```rust
/// use motor_driver_hal::PwmChannels;
/// # struct MyPwm;
/// # impl embedded_hal::pwm::ErrorType for MyPwm { type Error = (); }
/// # impl embedded_hal::pwm::SetDutyCycle for MyPwm {
/// #   fn max_duty_cycle(&self) -> u16 { 1000 }
/// #   fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> { Ok(()) }
/// # }
///
/// // Digital on/off control only
/// let no_pwm: PwmChannels<MyPwm, MyPwm> = PwmChannels::None;
///
/// // Single PWM for speed control
/// let single_pwm = PwmChannels::Single(MyPwm);
///
/// // Dual PWM for directional control
/// let dual_pwm = PwmChannels::Dual(MyPwm, MyPwm);
/// ```
pub enum PwmChannels<P1, P2> {
    /// No PWM control - simple digital on/off operation
    None,
    /// Single PWM channel for speed control (sign-magnitude method)
    Single(P1),
    /// Dual PWM channels for directional control (locked anti-phase method)
    Dual(P1, P2),
}

/// Motor direction and control states.
///
/// Defines the various states a motor can be in, including directional movement
/// and stopping modes. The stopping modes (Brake vs Coast) provide different
/// characteristics for motor control applications.
///
/// # States
///
/// - **Forward/Reverse**: Active movement in specified direction
/// - **Brake**: Active braking - both motor terminals connected to same potential
/// - **Coast**: Passive stopping - motor terminals disconnected (high impedance)
///
/// # Brake vs Coast
///
/// ## Brake (Active/Hard Stop)
/// - Motor terminals short-circuited or connected to ground
/// - Motor acts as generator, creating back-EMF that opposes rotation
/// - Quick, controlled deceleration
/// - Higher stress on mechanical components
/// - Better for precision positioning
///
/// ## Coast (Passive/Soft Stop)
/// - Motor terminals disconnected (high impedance)
/// - Motor freely decelerates due to friction and load
/// - Gradual, natural deceleration  
/// - Lower stress on components
/// - Motor can be back-driven by external forces
///
/// # Examples
///
/// ```rust
/// use motor_driver_hal::MotorDirection;
///
/// let direction = MotorDirection::Forward;
/// 
/// match direction {
///     MotorDirection::Forward => println!("Moving forward"),
///     MotorDirection::Reverse => println!("Moving reverse"),
///     MotorDirection::Brake => println!("Active braking"),
///     MotorDirection::Coast => println!("Coasting to stop"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorDirection {
    /// Forward direction movement
    Forward,
    /// Reverse direction movement  
    Reverse,
    /// Active braking - quick stop with electrical braking
    Brake,
    /// Coast to stop - passive deceleration
    Coast,
}

/// Flexible motor driver wrapper supporting various hardware configurations.
///
/// This wrapper provides a unified interface for different motor driver hardware
/// configurations through configurable enable pins and PWM channels. It supports
/// the builder pattern for easy configuration and provides comprehensive motor
/// control capabilities.
///
/// # Type Parameters
///
/// - `E1`: Primary enable pin type
/// - `E2`: Secondary enable pin type (for dual enable configurations)
/// - `P1`: Primary PWM channel type
/// - `P2`: Secondary PWM channel type (for dual PWM configurations)
///
/// # Hardware Configurations Supported
///
/// - **Digital Only**: No PWM, just on/off control via enable pins
/// - **Single PWM**: One PWM channel with enable pin(s) for direction
/// - **Dual PWM**: Two PWM channels for independent directional control
/// - **Various Enable**: No enable, single enable, or dual enable pins
///
/// # Usage Example
///
/// ```rust,no_run
/// use motor_driver_hal::{MotorDriverWrapper, MotorDriver, EnablePins, PwmChannels};
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
/// // Create motor driver with builder pattern
/// let mut motor = MotorDriverWrapper::builder()
///     .with_enable_pins(EnablePins::Single(MyPin))
///     .with_pwm_channels(PwmChannels::Dual(MyPwm, MyPwm))
///     .with_max_duty(1000)
///     .build();
///
/// // Use the motor
/// motor.initialize()?;
/// motor.enable()?;
/// motor.set_speed(500)?;  // 50% forward speed
/// motor.brake()?;         // Active braking
/// motor.disable()?;
/// # Ok::<(), motor_driver_hal::MotorDriverError>(())
/// ```
pub struct MotorDriverWrapper<E1, E2, P1, P2> {
    /// Enable pin configuration
    enable_pins: EnablePins<E1, E2>,
    /// PWM channel configuration
    pwm_channels: PwmChannels<P1, P2>,
    /// Maximum duty cycle value for PWM channels
    max_duty: u16,
    /// Current commanded speed (-max_duty to +max_duty)
    current_speed: i16,
    /// Current motor direction/state
    direction: MotorDirection,
    /// Whether the driver has been initialized
    initialized: bool,
}

impl<E1, E2, P1, P2> MotorDriverWrapper<E1, E2, P1, P2>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
{
    /// Create a new builder for configuring the motor driver.
    ///
    /// Returns a builder instance that allows you to configure the motor driver
    /// with different combinations of enable pins and PWM channels using a
    /// fluent interface.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use motor_driver_hal::{MotorDriverWrapper, EnablePins, PwmChannels};
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
    /// let motor = MotorDriverWrapper::builder()
    ///     .with_enable_pins(EnablePins::Dual(MyPin, MyPin))
    ///     .with_pwm_channels(PwmChannels::Single(MyPwm))
    ///     .with_max_duty(255)
    ///     .build();
    /// ```
    pub fn builder() -> MotorDriverBuilder<E1, E2, P1, P2> {
        MotorDriverBuilder::new()
    }

    /// Control enable pins based on configuration.
    ///
    /// This internal method handles enabling or disabling the motor driver
    /// based on the enable pin configuration. It supports all enable pin
    /// configurations: None, Single, and Dual.
    ///
    /// # Arguments
    ///
    /// * `enable` - `true` to enable the motor driver, `false` to disable
    ///
    /// # Errors
    ///
    /// Returns `MotorDriverError::GpioError` if GPIO operations fail.
    fn control_enable(&mut self, enable: bool) -> Result<(), MotorDriverError> {
        match &mut self.enable_pins {
            EnablePins::None => Ok(()),
            EnablePins::Single(pin) => {
                if enable {
                    pin.set_high().map_err(|_| MotorDriverError::GpioError)?;
                } else {
                    pin.set_low().map_err(|_| MotorDriverError::GpioError)?;
                }
                Ok(())
            }
            EnablePins::Dual(pin1, pin2) => {
                if enable {
                    pin1.set_high().map_err(|_| MotorDriverError::GpioError)?;
                    pin2.set_high().map_err(|_| MotorDriverError::GpioError)?;
                } else {
                    pin1.set_low().map_err(|_| MotorDriverError::GpioError)?;
                    pin2.set_low().map_err(|_| MotorDriverError::GpioError)?;
                }
                Ok(())
            }
        }
    }

    /// Update PWM channels based on current speed and direction.
    ///
    /// This internal method handles the complex logic of setting PWM duty cycles
    /// based on the current motor state and PWM configuration. It supports all
    /// PWM configurations and motor directions including braking and coasting.
    ///
    /// # PWM Control Logic
    ///
    /// ## Single PWM Configuration
    /// - Speed controls PWM duty cycle
    /// - Direction typically controlled by enable pins
    /// - Coast mode: PWM duty = 0
    ///
    /// ## Dual PWM Configuration
    /// - Forward: PWM1 = duty, PWM2 = 0
    /// - Reverse: PWM1 = 0, PWM2 = duty
    /// - Brake: PWM1 = max_duty, PWM2 = max_duty
    /// - Coast: PWM1 = 0, PWM2 = 0
    ///
    /// # Errors
    ///
    /// Returns `MotorDriverError::PwmError` if PWM operations fail.
    fn update_pwm(&mut self) -> Result<(), MotorDriverError> {
        let duty = self.current_speed.unsigned_abs().min(self.max_duty);

        match (&mut self.pwm_channels, self.direction) {
            (PwmChannels::None, _) => Ok(()),
            (PwmChannels::Single(pwm), _) => {
                if self.direction == MotorDirection::Coast {
                    pwm.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
                } else {
                    pwm.set_duty_cycle(duty).map_err(|_| MotorDriverError::PwmError)?;
                }
                Ok(())
            }
            (PwmChannels::Dual(pwm1, pwm2), MotorDirection::Forward) => {
                pwm1.set_duty_cycle(duty).map_err(|_| MotorDriverError::PwmError)?;
                pwm2.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
                Ok(())
            }
            (PwmChannels::Dual(pwm1, pwm2), MotorDirection::Reverse) => {
                pwm1.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
                pwm2.set_duty_cycle(duty).map_err(|_| MotorDriverError::PwmError)?;
                Ok(())
            }
            (PwmChannels::Dual(pwm1, pwm2), MotorDirection::Brake) => {
                pwm1.set_duty_cycle(self.max_duty).map_err(|_| MotorDriverError::PwmError)?;
                pwm2.set_duty_cycle(self.max_duty).map_err(|_| MotorDriverError::PwmError)?;
                Ok(())
            }
            (PwmChannels::Dual(pwm1, pwm2), MotorDirection::Coast) => {
                pwm1.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
                pwm2.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
                Ok(())
            }
        }
    }
}

impl<E1, E2, P1, P2> MotorDriver for MotorDriverWrapper<E1, E2, P1, P2>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
{
    type Error = MotorDriverError;

    fn initialize(&mut self) -> Result<(), Self::Error> {
        self.control_enable(false)?;
        
        match &mut self.pwm_channels {
            PwmChannels::None => {},
            PwmChannels::Single(pwm) => {
                pwm.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
            }
            PwmChannels::Dual(pwm1, pwm2) => {
                pwm1.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
                pwm2.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
            }
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
        if speed < 0 {
            self.direction = MotorDirection::Reverse;
        } else if speed > 0 {
            self.direction = MotorDirection::Forward;
        }
        
        self.update_pwm()
    }

    fn set_direction(&mut self, forward: bool) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.direction = if forward {
            MotorDirection::Forward
        } else {
            MotorDirection::Reverse
        };
        
        self.update_pwm()
    }

    fn stop(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.current_speed = 0;
        self.direction = MotorDirection::Coast;
        self.update_pwm()
    }

    fn brake(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.current_speed = 0;
        self.direction = MotorDirection::Brake;
        self.update_pwm()
    }

    fn enable(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.control_enable(true)
    }

    fn disable(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.control_enable(false)
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
        Ok(self.direction == MotorDirection::Forward)
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

/// Builder for configuring motor driver wrapper instances.
///
/// This builder provides a fluent interface for configuring motor drivers with
/// different hardware configurations. It ensures that all necessary components
/// are properly configured before creating the final driver instance.
///
/// # Type Parameters
///
/// - `E1`: Primary enable pin type
/// - `E2`: Secondary enable pin type
/// - `P1`: Primary PWM channel type  
/// - `P2`: Secondary PWM channel type
///
/// # Default Values
///
/// If not specified during building:
/// - Enable pins: `EnablePins::None` (no enable pins)
/// - PWM channels: `PwmChannels::None` (digital on/off only)
/// - Max duty: `u16::MAX` (full range)
///
/// # Examples
///
/// ```rust,no_run
/// use motor_driver_hal::{MotorDriverBuilder, EnablePins, PwmChannels};
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
/// // Configure a motor driver with dual PWM and single enable
/// let motor = MotorDriverBuilder::new()
///     .with_enable_pins(EnablePins::Single(MyPin))
///     .with_pwm_channels(PwmChannels::Dual(MyPwm, MyPwm))
///     .with_max_duty(1000)
///     .build();
/// ```
pub struct MotorDriverBuilder<E1, E2, P1, P2> {
    /// Enable pin configuration (optional)
    enable_pins: Option<EnablePins<E1, E2>>,
    /// PWM channel configuration (optional)
    pwm_channels: Option<PwmChannels<P1, P2>>,
    /// Maximum duty cycle value (optional)
    max_duty: Option<u16>,
}

impl<E1, E2, P1, P2> MotorDriverBuilder<E1, E2, P1, P2> {
    /// Create a new builder with default (empty) configuration.
    ///
    /// All configuration options start as `None` and will use default values
    /// if not explicitly set before calling `build()`.
    pub fn new() -> Self {
        Self {
            enable_pins: None,
            pwm_channels: None,
            max_duty: None,
        }
    }

    /// Configure the enable pins for the motor driver.
    ///
    /// # Arguments
    ///
    /// * `pins` - Enable pin configuration (`None`, `Single`, or `Dual`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use motor_driver_hal::{MotorDriverBuilder, EnablePins};
    /// # struct MyPin;
    /// # impl embedded_hal::digital::ErrorType for MyPin { type Error = (); }
    /// # impl embedded_hal::digital::OutputPin for MyPin {
    /// #   fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #   fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    ///
    /// let builder = MotorDriverBuilder::new()
    ///     .with_enable_pins(EnablePins::Dual(MyPin, MyPin));
    /// ```
    pub fn with_enable_pins(mut self, pins: EnablePins<E1, E2>) -> Self {
        self.enable_pins = Some(pins);
        self
    }

    /// Configure the PWM channels for the motor driver.
    ///
    /// # Arguments
    ///
    /// * `channels` - PWM channel configuration (`None`, `Single`, or `Dual`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use motor_driver_hal::{MotorDriverBuilder, PwmChannels};
    /// # struct MyPwm;
    /// # impl embedded_hal::pwm::ErrorType for MyPwm { type Error = (); }
    /// # impl embedded_hal::pwm::SetDutyCycle for MyPwm {
    /// #   fn max_duty_cycle(&self) -> u16 { 1000 }
    /// #   fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    ///
    /// let builder = MotorDriverBuilder::new()
    ///     .with_pwm_channels(PwmChannels::Single(MyPwm));
    /// ```
    pub fn with_pwm_channels(mut self, channels: PwmChannels<P1, P2>) -> Self {
        self.pwm_channels = Some(channels);
        self
    }

    /// Set the maximum duty cycle value for PWM channels.
    ///
    /// This value determines the range of speed control and should match
    /// your PWM hardware capabilities. Common values are 255, 1000, or 4095.
    ///
    /// # Arguments
    ///
    /// * `max_duty` - Maximum duty cycle value (1 to 65535)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use motor_driver_hal::MotorDriverBuilder;
    ///
    /// // For 8-bit PWM resolution
    /// let builder = MotorDriverBuilder::new().with_max_duty(255);
    ///
    /// // For 12-bit PWM resolution
    /// let builder = MotorDriverBuilder::new().with_max_duty(4095);
    /// ```
    pub fn with_max_duty(mut self, max_duty: u16) -> Self {
        self.max_duty = Some(max_duty);
        self
    }

    /// Build the final motor driver wrapper instance.
    ///
    /// Creates a `MotorDriverWrapper` with the configured settings.
    /// Any unspecified settings will use their default values.
    ///
    /// # Default Values
    ///
    /// - Enable pins: `EnablePins::None`
    /// - PWM channels: `PwmChannels::None`
    /// - Max duty: `u16::MAX`
    ///
    /// # Returns
    ///
    /// A new, uninitialized `MotorDriverWrapper` instance ready for use.
    /// Call `initialize()` on the returned instance before using it.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use motor_driver_hal::{MotorDriverBuilder, MotorDriver};
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
    /// let mut motor = MotorDriverBuilder::new()
    ///     .with_max_duty(1000)
    ///     .build();
    ///
    /// motor.initialize()?;
    /// # Ok::<(), motor_driver_hal::MotorDriverError>(())
    /// ```
    pub fn build(self) -> MotorDriverWrapper<E1, E2, P1, P2> {
        MotorDriverWrapper {
            enable_pins: self.enable_pins.unwrap_or(EnablePins::None),
            pwm_channels: self.pwm_channels.unwrap_or(PwmChannels::None),
            max_duty: self.max_duty.unwrap_or(u16::MAX),
            current_speed: 0,
            direction: MotorDirection::Coast,
            initialized: false,
        }
    }
}

impl<E1, E2, P1, P2> Default for MotorDriverBuilder<E1, E2, P1, P2> {
    fn default() -> Self {
        Self::new()
    }
}