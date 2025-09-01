use crate::{MotorDriver, MotorDriverError};
use embedded_hal::digital::{OutputPin, InputPin};
use embedded_hal::pwm::SetDutyCycle;

/// Placeholder encoder implementation for motors without encoder feedback.
/// 
/// This struct provides a no-op implementation of the `InputPin` trait
/// for use in motor driver configurations that don't require encoder feedback.
/// It always returns low state and is used as a type parameter placeholder.
/// 
/// # Example
/// 
/// ```rust
/// use motor_driver_hal::NoEncoder;
/// 
/// // Used automatically when creating drivers without encoders
/// let motor = HBridgeMotorDriver::single_pwm(enable_pin, pwm_channel, 1000);
/// ```
#[derive(Debug)]
pub struct NoEncoder;

/// Error type for the NoEncoder placeholder implementation.
/// 
/// This error type is never actually returned since NoEncoder operations
/// always succeed, but it's required to implement the ErrorType trait.
#[derive(Debug)]
pub struct NoEncoderError;

impl embedded_hal::digital::Error for NoEncoderError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

impl embedded_hal::digital::ErrorType for NoEncoder {
    type Error = NoEncoderError;
}

impl InputPin for NoEncoder {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Level {
    Low = 0,
    High = 1,
}

/// H-bridge motor driver implementation with optional encoder support.
/// 
/// This struct provides comprehensive motor control functionality including:
/// - Single or dual PWM channel control for H-bridge motor drivers
/// - Single or dual enable pin control
/// - Optional quadrature encoder support for position feedback
/// - Speed and direction control with safety checks
/// 
/// # Type Parameters
/// 
/// * `E1` - Primary enable pin type implementing `OutputPin`
/// * `E2` - Secondary enable pin type implementing `OutputPin` (optional)
/// * `P1` - Primary PWM channel type implementing `SetDutyCycle`
/// * `P2` - Secondary PWM channel type implementing `SetDutyCycle` (optional)
/// * `Enc1` - Encoder A channel type implementing `InputPin` (optional)
/// * `Enc2` - Encoder B channel type implementing `InputPin` (optional)
/// 
/// # Example
/// 
/// ```rust
/// use motor_driver_hal::HBridgeMotorDriver;
/// 
/// // Create a simple single PWM motor driver
/// let motor = HBridgeMotorDriver::single_pwm(enable_pin, pwm_channel, 1000);
/// 
/// // Create a dual PWM motor driver with encoders
/// let motor = HBridgeMotorDriver::dual_pwm_with_encoder(
///     enable1, enable2, pwm1, pwm2, enc_a, enc_b, 1000
/// );
/// ```
pub struct HBridgeMotorDriver<E1, E2, P1, P2, Enc1, Enc2> {
    enable1: E1,
    enable2: Option<E2>,
    pwm1: P1,
    pwm2: Option<P2>,
    encoder1: Option<Enc1>,
    encoder2: Option<Enc2>,
    max_duty: u16,
    current_speed: i16,
    pulse_count: i32,
    pulse_offset: i32,
    target_pulse: i32,
    ppr: u16,
    last_enc_a: Level,
    last_enc_b: Level,
    direction: bool,
    initialized: bool,
}

const QEM: [i8; 16] = [
     0, -1,  1,  0,
     1,  0,  0, -1,
    -1,  0,  0,  1,
     0,  1, -1,  0,
];

/// Builder for constructing HBridgeMotorDriver instances.
/// 
/// This builder provides a flexible way to configure motor drivers with
/// various combinations of enable pins, PWM channels, and encoders.
/// 
/// # Type Parameters
/// 
/// * `E1, E2` - Enable pin types implementing `OutputPin`
/// * `P1, P2` - PWM channel types implementing `SetDutyCycle`
/// * `Enc1, Enc2` - Encoder channel types implementing `InputPin`
/// 
/// # Example
/// 
/// ```rust
/// use motor_driver_hal::HBridgeMotorDriver;
/// 
/// let motor = HBridgeMotorDriver::builder()
///     .with_enable(enable_pin)
///     .with_pwm(pwm_channel)
///     .with_max_duty(1000)
///     .with_ppr(1024)
///     .build();
/// ```
pub struct HBridgeMotorDriverBuilder<E1, E2, P1, P2, Enc1, Enc2> {
    enable1: Option<E1>,
    enable2: Option<E2>,
    pwm1: Option<P1>,
    pwm2: Option<P2>,
    encoder1: Option<Enc1>,
    encoder2: Option<Enc2>,
    max_duty: Option<u16>,
    ppr: Option<u16>,
    initial_speed: Option<i16>,
}

impl<E1, E2, P1, P2, Enc1, Enc2> HBridgeMotorDriverBuilder<E1, E2, P1, P2, Enc1, Enc2> {
    /// Creates a new builder instance with all fields unset.
    /// 
    /// # Returns
    /// 
    /// A new `HBridgeMotorDriverBuilder` with default (None) values
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let builder = HBridgeMotorDriverBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            enable1: None,
            enable2: None,
            pwm1: None,
            pwm2: None,
            encoder1: None,
            encoder2: None,
            max_duty: None,
            ppr: None,
            initial_speed: None,
        }
    }

    /// Sets the primary enable pin for the motor driver.
    /// 
    /// # Arguments
    /// 
    /// * `enable` - GPIO pin implementing `OutputPin` used to enable/disable the motor driver
    /// 
    /// # Returns
    /// 
    /// The builder instance for method chaining
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let builder = builder.with_enable(gpio_pin_18);
    /// ```
    pub fn with_enable(mut self, enable: E1) -> Self {
        self.enable1 = Some(enable);
        self
    }

    /// Sets both enable pins for dual-enable motor driver configurations.
    /// 
    /// # Arguments
    /// 
    /// * `enable1` - Primary enable pin implementing `OutputPin`
    /// * `enable2` - Secondary enable pin implementing `OutputPin`
    /// 
    /// # Returns
    /// 
    /// The builder instance for method chaining
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let builder = builder.with_dual_enable(gpio_pin_18, gpio_pin_19);
    /// ```
    pub fn with_dual_enable(mut self, enable1: E1, enable2: E2) -> Self {
        self.enable1 = Some(enable1);
        self.enable2 = Some(enable2);
        self
    }

    /// Sets the primary PWM channel for motor speed control.
    /// 
    /// # Arguments
    /// 
    /// * `pwm` - PWM channel implementing `SetDutyCycle` trait
    /// 
    /// # Returns
    /// 
    /// The builder instance for method chaining
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let builder = builder.with_pwm(pwm_channel_0);
    /// ```
    pub fn with_pwm(mut self, pwm: P1) -> Self {
        self.pwm1 = Some(pwm);
        self
    }

    /// Sets both PWM channels for dual-PWM motor driver configurations.
    /// 
    /// This configuration allows for more precise direction control where
    /// one PWM controls forward motion and the other controls reverse motion.
    /// 
    /// # Arguments
    /// 
    /// * `pwm1` - Primary PWM channel (typically forward direction)
    /// * `pwm2` - Secondary PWM channel (typically reverse direction)
    /// 
    /// # Returns
    /// 
    /// The builder instance for method chaining
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let builder = builder.with_dual_pwm(pwm_channel_0, pwm_channel_1);
    /// ```
    pub fn with_dual_pwm(mut self, pwm1: P1, pwm2: P2) -> Self {
        self.pwm1 = Some(pwm1);
        self.pwm2 = Some(pwm2);
        self
    }

    /// Sets the quadrature encoder channels for position feedback.
    /// 
    /// # Arguments
    /// 
    /// * `encoder1` - Encoder A channel implementing `InputPin`
    /// * `encoder2` - Encoder B channel implementing `InputPin`
    /// 
    /// # Returns
    /// 
    /// The builder instance for method chaining
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let builder = builder.with_encoder(encoder_a_pin, encoder_b_pin);
    /// ```
    pub fn with_encoder(mut self, encoder1: Enc1, encoder2: Enc2) -> Self {
        self.encoder1 = Some(encoder1);
        self.encoder2 = Some(encoder2);
        self
    }

    /// Sets the maximum duty cycle value for PWM control.
    /// 
    /// This value determines the resolution and maximum speed of the motor.
    /// Higher values provide finer speed control resolution.
    /// 
    /// # Arguments
    /// 
    /// * `max_duty` - Maximum duty cycle value (typical values: 255, 1000, 4095)
    /// 
    /// # Returns
    /// 
    /// The builder instance for method chaining
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let builder = builder.with_max_duty(1000); // 0-1000 speed range
    /// ```
    pub fn with_max_duty(mut self, max_duty: u16) -> Self {
        self.max_duty = Some(max_duty);
        self
    }

    /// Sets the pulses per revolution for encoder calculations.
    /// 
    /// This value is used for position control and speed calculations
    /// when encoders are present.
    /// 
    /// # Arguments
    /// 
    /// * `ppr` - Number of encoder pulses per complete motor revolution
    /// 
    /// # Returns
    /// 
    /// The builder instance for method chaining
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let builder = builder.with_ppr(1024); // 1024 pulses per revolution
    /// ```
    pub fn with_ppr(mut self, ppr: u16) -> Self {
        self.ppr = Some(ppr);
        self
    }

    /// Sets the initial speed value for the motor driver.
    /// 
    /// The motor will be configured to this speed when built, but will
    /// not actually move until `enable()` is called.
    /// 
    /// # Arguments
    /// 
    /// * `speed` - Initial speed value (positive = forward, negative = reverse)
    /// 
    /// # Returns
    /// 
    /// The builder instance for method chaining
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let builder = builder.with_initial_speed(0); // Start stopped
    /// ```
    pub fn with_initial_speed(mut self, speed: i16) -> Self {
        self.initial_speed = Some(speed);
        self
    }

    /// Builds the motor driver instance from the configured parameters.
    /// 
    /// # Returns
    /// 
    /// A configured `HBridgeMotorDriver` instance ready for initialization
    /// 
    /// # Panics
    /// 
    /// Panics if required parameters (enable pin and PWM channel) are not set
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let motor = HBridgeMotorDriver::builder()
    ///     .with_enable(enable_pin)
    ///     .with_pwm(pwm_channel)
    ///     .build();
    /// ```
    pub fn build(self) -> HBridgeMotorDriver<E1, E2, P1, P2, Enc1, Enc2> {
        HBridgeMotorDriver {
            enable1: self.enable1.expect("Enable pin is required"),
            enable2: self.enable2,
            pwm1: self.pwm1.expect("PWM channel is required"),
            pwm2: self.pwm2,
            encoder1: self.encoder1,
            encoder2: self.encoder2,
            max_duty: self.max_duty.unwrap_or(1000),
            current_speed: self.initial_speed.unwrap_or(0),
            pulse_count: 0,
            pulse_offset: 0,
            target_pulse: 0,
            ppr: self.ppr.unwrap_or(0),
            last_enc_a: Level::Low,
            last_enc_b: Level::Low,
            direction: true,
            initialized: false,
        }
    }

    /// Builds and initializes the motor driver in one step.
    /// 
    /// This convenience method combines `build()` and `initialize()` operations,
    /// returning a ready-to-use motor driver instance.
    /// 
    /// # Returns
    /// 
    /// * `Ok(driver)` - Initialized motor driver ready for use
    /// * `Err(MotorDriverError)` - If building fails or initialization fails
    /// 
    /// # Errors
    /// 
    /// Returns error if required parameters are missing or hardware initialization fails.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let motor = HBridgeMotorDriver::builder()
    ///     .with_enable(enable_pin)
    ///     .with_pwm(pwm_channel)
    ///     .build_and_init()?;
    /// ```
    pub fn build_and_init(self) -> Result<HBridgeMotorDriver<E1, E2, P1, P2, Enc1, Enc2>, MotorDriverError>
    where
        E1: OutputPin,
        E2: OutputPin,
        P1: SetDutyCycle,
        P2: SetDutyCycle,
        Enc1: InputPin,
        Enc2: InputPin,
    {
        let mut driver = self.build();
        driver.initialize()?;
        Ok(driver)
    }
}

impl<E1, E2, P1, P2> HBridgeMotorDriver<E1, E2, P1, P2, NoEncoder, NoEncoder>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
{   
    /// Creates a new builder for motor drivers without encoder support.
    /// 
    /// # Returns
    /// 
    /// A new builder instance configured for NoEncoder types
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let motor = HBridgeMotorDriver::builder()
    ///     .with_enable(enable_pin)
    ///     .with_pwm(pwm_channel)
    ///     .build();
    /// ```
    pub fn builder() -> HBridgeMotorDriverBuilder<E1, E2, P1, P2, NoEncoder, NoEncoder> {
        HBridgeMotorDriverBuilder::new()
    }

    /// Creates a motor driver with single PWM channel configuration.
    /// 
    /// This is a convenience constructor for the most common motor driver
    /// configuration using one enable pin and one PWM channel.
    /// 
    /// # Arguments
    /// 
    /// * `enable` - GPIO pin for enabling/disabling the motor driver
    /// * `pwm` - PWM channel for speed control
    /// * `max_duty` - Maximum duty cycle value for speed scaling
    /// 
    /// # Returns
    /// 
    /// A configured motor driver instance (not yet initialized)
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let motor = HBridgeMotorDriver::single_pwm(enable_pin, pwm_channel, 1000);
    /// ```
    pub fn single_pwm(enable: E1, pwm: P1, max_duty: u16) -> Self {
        Self::builder()
            .with_enable(enable)
            .with_pwm(pwm)
            .with_max_duty(max_duty)
            .build()
    }

    /// Creates a motor driver with dual PWM and dual enable configuration.
    /// 
    /// This configuration provides the most control options with separate
    /// PWM channels for each direction and separate enable pins.
    /// 
    /// # Arguments
    /// 
    /// * `enable1` - Primary enable pin
    /// * `enable2` - Secondary enable pin
    /// * `pwm1` - Primary PWM channel (forward direction)
    /// * `pwm2` - Secondary PWM channel (reverse direction)
    /// * `max_duty` - Maximum duty cycle value for both PWM channels
    /// 
    /// # Returns
    /// 
    /// A configured motor driver instance (not yet initialized)
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let motor = HBridgeMotorDriver::dual_pwm(
    ///     enable1, enable2, pwm1, pwm2, 1000
    /// );
    /// ```
    pub fn dual_pwm(enable1: E1, enable2: E2, pwm1: P1, pwm2: P2, max_duty: u16) -> Self {
        Self::builder()
            .with_dual_enable(enable1, enable2)
            .with_dual_pwm(pwm1, pwm2)
            .with_max_duty(max_duty)
            .build()
    }
}

impl<E1, E2, P1, P2, Enc1, Enc2> HBridgeMotorDriver<E1, E2, P1, P2, Enc1, Enc2>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
    Enc1: InputPin,
    Enc2: InputPin,
{
    /// Creates a new builder for motor drivers with encoder support.
    /// 
    /// # Returns
    /// 
    /// A new builder instance configured for encoder types Enc1 and Enc2
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let motor = HBridgeMotorDriver::builder_with_encoder()
    ///     .with_dual_enable(enable1, enable2)
    ///     .with_dual_pwm(pwm1, pwm2)
    ///     .with_encoder(enc_a, enc_b)
    ///     .with_ppr(1024)
    ///     .build();
    /// ```
    pub fn builder_with_encoder() -> HBridgeMotorDriverBuilder<E1, E2, P1, P2, Enc1, Enc2> {
        HBridgeMotorDriverBuilder::new()
    }
    
    /// Creates a motor driver with dual PWM, dual enable, and encoder support.
    /// 
    /// This is the most feature-complete configuration providing precise
    /// motor control with position feedback.
    /// 
    /// # Arguments
    /// 
    /// * `enable1` - Primary enable pin
    /// * `enable2` - Secondary enable pin  
    /// * `pwm1` - Primary PWM channel (forward direction)
    /// * `pwm2` - Secondary PWM channel (reverse direction)
    /// * `encoder1` - Encoder A channel pin
    /// * `encoder2` - Encoder B channel pin
    /// * `max_duty` - Maximum duty cycle value
    /// 
    /// # Returns
    /// 
    /// A configured motor driver instance with encoder support
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let motor = HBridgeMotorDriver::dual_pwm_with_encoder(
    ///     enable1, enable2, pwm1, pwm2, enc_a, enc_b, 1000
    /// );
    /// ```
    pub fn dual_pwm_with_encoder(enable1: E1, enable2: E2, pwm1: P1, pwm2: P2, encoder1: Enc1, encoder2: Enc2, max_duty: u16) -> Self {
        Self::builder_with_encoder()
            .with_dual_enable(enable1, enable2)
            .with_dual_pwm(pwm1, pwm2)
            .with_encoder(encoder1, encoder2)
            .with_max_duty(max_duty)
            .build()
    }

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

    /// Reads the current encoder state and updates pulse count.
    /// 
    /// This method implements quadrature encoder decoding using a state machine
    /// to track motor position. It should be called regularly (typically in a
    /// timer interrupt or polling loop) to maintain accurate position tracking.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` if encoder reading succeeds
    /// * `Err(MotorDriverError::GpioError)` if encoder pin reading fails
    /// * `Err(MotorDriverError::HardwareFault)` if encoders are not configured
    /// 
    /// # Example
    /// 
    /// ```rust
    /// // In a timer interrupt or polling loop
    /// motor.read_encoder()?;
    /// let position = motor.get_pulse_count();
    /// ```
    pub fn read_encoder(&mut self) -> Result<(), MotorDriverError> {
        if let (Some(ref mut enc_a), Some(ref mut enc_b)) = (&mut self.encoder1, &mut self.encoder2) {
            let level_a = if enc_a.is_high().map_err(|_| MotorDriverError::GpioError)? { 
                Level::High 
            } else { 
                Level::Low 
            };
            let level_b = if enc_b.is_high().map_err(|_| MotorDriverError::GpioError)? { 
                Level::High 
            } else { 
                Level::Low 
            };

            let index = ((self.last_enc_a as u8) << 3)
                      | ((self.last_enc_b as u8) << 2)
                      | ((level_a as u8) << 1)
                      | (level_b as u8);
            
            self.pulse_count += QEM[index as usize] as i32;
            self.last_enc_a = level_a;
            self.last_enc_b = level_b;
            
            Ok(())
        } else {
            Err(MotorDriverError::HardwareFault)
        }
    }

    /// Gets the current encoder pulse count relative to the last reset.
    /// 
    /// The pulse count is automatically adjusted by the pulse offset set
    /// by `reset_encoder()` to provide relative position measurements.
    /// 
    /// # Returns
    /// 
    /// Current pulse count since last encoder reset
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.reset_encoder(); // Reset to zero
    /// // ... motor movement ...
    /// let position = motor.get_pulse_count(); // Position since reset
    /// ```
    pub fn get_pulse_count(&self) -> i32 {
        self.pulse_count - self.pulse_offset
    }

    /// Resets the encoder position counter to zero.
    /// 
    /// This sets the current position as the new reference point (zero).
    /// Subsequent calls to `get_pulse_count()` will return values relative
    /// to this reset point.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.reset_encoder(); // Set current position as zero
    /// ```
    pub fn reset_encoder(&mut self) {
        self.pulse_offset = self.pulse_count;
    }

    /// Sets the target pulse count for position control.
    /// 
    /// This target is used by `check_ppr()` to verify that the motor
    /// has reached the desired position.
    /// 
    /// # Arguments
    /// 
    /// * `target` - Target pulse count relative to encoder reset point
    /// 
    /// # Example
    /// 
    /// ```rust
    /// motor.set_target_pulse(1000); // Move 1000 pulses from current position
    /// ```
    pub fn set_target_pulse(&mut self, target: i32) {
        self.target_pulse = target;
    }
}

impl<E1, E2, P1, P2, Enc1, Enc2> MotorDriver for HBridgeMotorDriver<E1, E2, P1, P2, Enc1, Enc2>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
    Enc1: InputPin,
    Enc2: InputPin,
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

    fn set_ppr(&mut self, ppr: i16) -> Result<bool, Self::Error> {        
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        if ppr <= 0 {
            return Err(MotorDriverError::InvalidSpeed);
        }
        self.ppr = ppr as u16;
        Ok(true)
    }

    fn check_ppr(&mut self) -> Result<(), Self::Error> {
        if self.ppr == 0 {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.read_encoder()?;
        
        let current_pulse = self.get_pulse_count();
        let current_rotation_pulse = current_pulse % (self.ppr as i32);
        let target_rotation_pulse = self.target_pulse % (self.ppr as i32);
        
        if current_rotation_pulse == target_rotation_pulse {
            Ok(())
        } else {
            Err(MotorDriverError::HardwareFault)
        }
    }


    fn get_current(&self) -> Result<f32, Self::Error> {
        Err(MotorDriverError::HardwareFault)
    }

    fn get_voltage(&self) -> Result<f32, Self::Error> {
        Err(MotorDriverError::HardwareFault)
    }

    fn get_temperature(&self) -> Result<f32, Self::Error> {
        Err(MotorDriverError::HardwareFault)
    }

    fn get_fault_status(&self) -> Result<u8, Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        Ok(0)
    }
}

#[cfg(feature = "rppal")]
pub mod rppal {
    use super::*;
    use crate::wrapper::rppal::{GpioWrapper, PwmWrapper};
    use ::rppal::gpio::{Gpio, InputPin as RppalInputPin, OutputPin as RppalOutputPin};
    use ::rppal::pwm::{Channel, Pwm, Polarity};

    pub type RppalMotorDriverBuilder = HBridgeMotorDriverBuilder<
        GpioWrapper<RppalOutputPin>,
        GpioWrapper<RppalOutputPin>,
        PwmWrapper,
        PwmWrapper,
        GpioWrapper<RppalInputPin>,
        GpioWrapper<RppalInputPin>
    >;

    impl RppalMotorDriverBuilder {
        /// Create a new Raspberry Pi motor driver builder.
        /// 
        /// # Returns
        /// 
        /// A builder configured for use with rppal GPIO and PWM.
        /// 
        /// # Example
        /// 
        /// ```rust
        /// let motor = RppalMotorDriverBuilder::new_rppal()
        ///     .with_dual_gpio_enable(&gpio, 23, 24)?
        ///     .with_dual_pwm_channels(Channel::Pwm1, Channel::Pwm2, 1000.0, 1000)?
        ///     .build_and_init()?;
        /// ```
        pub fn new_rppal() -> Self {
            HBridgeMotorDriverBuilder::new()
        }

        pub fn with_gpio_enable(mut self, gpio: &Gpio, pin: u8) -> Result<Self, ::rppal::gpio::Error> {
            self.enable1 = Some(GpioWrapper::new(gpio.get(pin)?.into_output()));
            Ok(self)
        }

        /// Configure dual GPIO enable pins for H-bridge control.
        /// 
        /// # Arguments
        /// 
        /// * `gpio` - Raspberry Pi GPIO interface
        /// * `pin1` - First enable pin number (0-27)
        /// * `pin2` - Second enable pin number (0-27)
        /// 
        /// # Example
        /// 
        /// ```rust
        /// builder.with_dual_gpio_enable(&gpio, 23, 24)?
        /// ```
        pub fn with_dual_gpio_enable(mut self, gpio: &Gpio, pin1: u8, pin2: u8) -> Result<Self, ::rppal::gpio::Error> {
            self.enable1 = Some(GpioWrapper::new(gpio.get(pin1)?.into_output()));
            self.enable2 = Some(GpioWrapper::new(gpio.get(pin2)?.into_output()));
            Ok(self)
        }

        pub fn with_pwm_channel(mut self, channel: Channel, frequency: f64, max_duty: u16) -> Result<Self, ::rppal::pwm::Error> {
            let pwm = Pwm::with_frequency(channel, frequency, 0.0, Polarity::Normal, true)?;
            self.pwm1 = Some(PwmWrapper::new(pwm, max_duty));
            self.max_duty = Some(max_duty);
            Ok(self)
        }

        /// Configure dual PWM channels for motor speed control.
        /// 
        /// # Arguments
        /// 
        /// * `channel1` - First PWM channel (Channel::Pwm1 or Channel::Pwm2)
        /// * `channel2` - Second PWM channel (Channel::Pwm1 or Channel::Pwm2)
        /// * `frequency` - PWM frequency in Hz (e.g., 1000.0)
        /// * `max_duty` - Maximum duty cycle value (e.g., 1000)
        /// 
        /// # Example
        /// 
        /// ```rust
        /// builder.with_dual_pwm_channels(Channel::Pwm1, Channel::Pwm2, 1000.0, 1000)?
        /// ```
        pub fn with_dual_pwm_channels(
            mut self, 
            channel1: Channel, 
            channel2: Channel, 
            frequency: f64, 
            max_duty: u16
        ) -> Result<Self, ::rppal::pwm::Error> {
            let pwm1 = Pwm::with_frequency(channel1, frequency, 0.0, Polarity::Normal, true)?;
            let pwm2 = Pwm::with_frequency(channel2, frequency, 0.0, Polarity::Normal, true)?;
            self.pwm1 = Some(PwmWrapper::new(pwm1, max_duty));
            self.pwm2 = Some(PwmWrapper::new(pwm2, max_duty));
            self.max_duty = Some(max_duty);
            Ok(self)
        }

        /// Configure quadrature encoder pins for position feedback.
        /// 
        /// # Arguments
        /// 
        /// * `gpio` - Raspberry Pi GPIO interface
        /// * `pin_a` - Encoder A phase pin number (0-27)
        /// * `pin_b` - Encoder B phase pin number (0-27)
        /// 
        /// # Example
        /// 
        /// ```rust
        /// builder.with_encoder_pins(&gpio, 25, 8)?
        /// ```
        pub fn with_encoder_pins(mut self, gpio: &Gpio, pin_a: u8, pin_b: u8) -> Result<Self, ::rppal::gpio::Error> {
            self.encoder1 = Some(GpioWrapper::new(gpio.get(pin_a)?.into_input_pullup()));
            self.encoder2 = Some(GpioWrapper::new(gpio.get(pin_b)?.into_input_pullup()));
            Ok(self)
        }
    }
}

#[cfg(feature = "linux-embedded-hal")]
pub mod linux {
    use super::*;
    use crate::wrapper::linux::{GpioWrapper, PwmWrapper};
    use linux_embedded_hal::{gpio_cdev::Chip, CdevPin};

    pub type LinuxMotorDriverBuilder = HBridgeMotorDriverBuilder<
        GpioWrapper,
        GpioWrapper,
        PwmWrapper,
        PwmWrapper,
        NoEncoder,
        NoEncoder
    >;

    impl LinuxMotorDriverBuilder {
        /// Create a new Linux motor driver builder.
        /// 
        /// # Returns
        /// 
        /// A builder configured for use with linux-embedded-hal GPIO and PWM.
        /// 
        /// # Example
        /// 
        /// ```rust
        /// let motor = LinuxMotorDriverBuilder::new_linux()
        ///     .with_dual_gpio_enable(&mut chip, 23, 24)?
        ///     .with_dual_pwm_channels(0, 0, 1, 1000)
        ///     .build_and_init()?;
        /// ```
        pub fn new_linux() -> Self {
            HBridgeMotorDriverBuilder::new()
        }

        pub fn with_gpio_enable(mut self, chip: &mut Chip, pin: u32) -> Result<Self, linux_embedded_hal::gpio_cdev::errors::Error> {
            let handle = chip.get_line(pin)?.request(
                linux_embedded_hal::gpio_cdev::LineRequestFlags::OUTPUT,
                0,
                "enable"
            )?;
            self.enable1 = Some(GpioWrapper::new(CdevPin::new(handle)?));
            Ok(self)
        }

        pub fn with_dual_gpio_enable(mut self, chip: &mut Chip, pin1: u32, pin2: u32) -> Result<Self, linux_embedded_hal::gpio_cdev::errors::Error> {
            let handle1 = chip.get_line(pin1)?.request(
                linux_embedded_hal::gpio_cdev::LineRequestFlags::OUTPUT,
                0,
                "enable1"
            )?;
            let handle2 = chip.get_line(pin2)?.request(
                linux_embedded_hal::gpio_cdev::LineRequestFlags::OUTPUT,
                0,
                "enable2"
            )?;
            self.enable1 = Some(GpioWrapper::new(CdevPin::new(handle1)?));
            self.enable2 = Some(GpioWrapper::new(CdevPin::new(handle2)?));
            Ok(self)
        }

        pub fn with_pwm_channel(mut self, chip: u32, channel: u32, max_duty: u16) -> Self {
            self.pwm1 = Some(PwmWrapper::new(chip, channel, max_duty));
            self.max_duty = Some(max_duty);
            self
        }

        pub fn with_dual_pwm_channels(mut self, chip: u32, channel1: u32, channel2: u32, max_duty: u16) -> Self {
            self.pwm1 = Some(PwmWrapper::new(chip, channel1, max_duty));
            self.pwm2 = Some(PwmWrapper::new(chip, channel2, max_duty));
            self.max_duty = Some(max_duty);
            self
        }
    }
}

