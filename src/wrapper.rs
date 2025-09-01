use crate::{MotorDriver, MotorDriverError};
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;

pub enum EnablePins<E1, E2> {
    None,
    Single(E1),
    Dual(E1, E2),
}

pub enum PwmChannels<P1, P2> {
    None,
    Single(P1),
    Dual(P1, P2),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorDirection {
    Forward,
    Reverse,
    Brake,
    Coast,
}

pub struct MotorDriverWrapper<E1, E2, P1, P2> {
    enable_pins: EnablePins<E1, E2>,
    pwm_channels: PwmChannels<P1, P2>,
    max_duty: u16,
    current_speed: i16,
    current_pulse: i16,
    ppr: i16,
    direction: MotorDirection,
    initialized: bool,
}

impl<E1, E2, P1, P2> MotorDriverWrapper<E1, E2, P1, P2>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
{
    pub fn builder() -> MotorDriverBuilder<E1, E2, P1, P2> {
        MotorDriverBuilder::new()
    }

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
    
    fn set_ppr(&mut self, ppr: i16) -> Result<bool, Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        self.ppr = ppr;
        Ok(true)
    }

    fn check_ppr(&mut self) -> Result<(), Self::Error> {
        if self.ppr == 0 {
            return Err(MotorDriverError::NotInitialized);
        }
        Ok(())
    }
}

pub struct MotorDriverBuilder<E1, E2, P1, P2> {
    enable_pins: Option<EnablePins<E1, E2>>,
    pwm_channels: Option<PwmChannels<P1, P2>>,
    max_duty: Option<u16>,
    initial_speed: Option<i16>,
    initial_direction: Option<MotorDirection>,
    ppr: Option<i16>,
}

impl<E1, E2, P1, P2> MotorDriverBuilder<E1, E2, P1, P2> {
    /// Create a new motor driver builder with default settings.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let builder = MotorDriverBuilder::new()
    ///     .with_single_enable(enable_pin)
    ///     .with_single_pwm(pwm_channel, 1000);
    /// ```
    pub fn new() -> Self {
        Self {
            enable_pins: None,
            pwm_channels: None,
            max_duty: None,
            initial_speed: None,
            initial_direction: None,
            ppr: None,
        }
    }

    pub fn with_single_enable(mut self, enable: E1) -> Self {
        self.enable_pins = Some(EnablePins::Single(enable));
        self
    }

    /// Configure dual enable pins for H-bridge motor control.
    /// 
    /// # Arguments
    /// 
    /// * `enable1` - First enable pin for one side of H-bridge
    /// * `enable2` - Second enable pin for other side of H-bridge
    /// 
    /// # Example
    /// 
    /// ```rust
    /// builder.with_dual_enable(enable_pin1, enable_pin2)
    /// ```
    pub fn with_dual_enable(mut self, enable1: E1, enable2: E2) -> Self {
        self.enable_pins = Some(EnablePins::Dual(enable1, enable2));
        self
    }

    pub fn with_single_pwm(mut self, pwm: P1) -> Self {
        self.pwm_channels = Some(PwmChannels::Single(pwm));
        self
    }

    pub fn with_dual_pwm(mut self, pwm1: P1, pwm2: P2) -> Self {
        self.pwm_channels = Some(PwmChannels::Dual(pwm1, pwm2));
        self
    }

    pub fn with_enable_pins(mut self, pins: EnablePins<E1, E2>) -> Self {
        self.enable_pins = Some(pins);
        self
    }

    pub fn with_pwm_channels(mut self, channels: PwmChannels<P1, P2>) -> Self {
        self.pwm_channels = Some(channels);
        self
    }

    pub fn with_max_duty(mut self, max_duty: u16) -> Self {
        self.max_duty = Some(max_duty);
        self
    }

    pub fn with_initial_speed(mut self, speed: i16) -> Self {
        self.initial_speed = Some(speed);
        self
    }

    pub fn with_initial_direction(mut self, direction: MotorDirection) -> Self {
        self.initial_direction = Some(direction);
        self
    }

    pub fn with_ppr(mut self, ppr: i16) -> Self {
        self.ppr = Some(ppr);
        self
    }

    pub fn build(self) -> MotorDriverWrapper<E1, E2, P1, P2> {
        MotorDriverWrapper {
            enable_pins: self.enable_pins.unwrap_or(EnablePins::None),
            pwm_channels: self.pwm_channels.unwrap_or(PwmChannels::None),
            max_duty: self.max_duty.unwrap_or(1000),
            current_speed: self.initial_speed.unwrap_or(0),
            current_pulse: 0,
            ppr: self.ppr.unwrap_or(0),
            direction: self.initial_direction.unwrap_or(MotorDirection::Coast),
            initialized: false,
        }
    }

    pub fn build_and_init(self) -> Result<MotorDriverWrapper<E1, E2, P1, P2>, MotorDriverError>
    where
        E1: OutputPin,
        E2: OutputPin,
        P1: SetDutyCycle,
        P2: SetDutyCycle,
    {
        let mut driver = self.build();
        driver.initialize()?;
        Ok(driver)
    }
}

impl<E1, E2, P1, P2> Default for MotorDriverBuilder<E1, E2, P1, P2> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "rppal")]
pub mod rppal {
    use super::*;
    use embedded_hal::digital::{OutputPin, InputPin};
    use embedded_hal::pwm::SetDutyCycle;
    use ::rppal::gpio::OutputPin as RppalOutputPin;
    use ::rppal::gpio::InputPin as RppalInputPin;
    use ::rppal::pwm::Pwm;

    #[derive(Debug)]
    pub struct RppalError;

    impl embedded_hal::pwm::Error for RppalError {
        fn kind(&self) -> embedded_hal::pwm::ErrorKind {
            embedded_hal::pwm::ErrorKind::Other
        }
    }

    impl embedded_hal::digital::Error for RppalError {
        fn kind(&self) -> embedded_hal::digital::ErrorKind {
            embedded_hal::digital::ErrorKind::Other
        }
    }

    pub struct GpioWrapper<P> {
        pin: P,
    }

    impl<P> GpioWrapper<P> {
        pub fn new(pin: P) -> Self {
            Self { pin }
        }
    }

    impl<P> embedded_hal::digital::ErrorType for GpioWrapper<P> {
        type Error = RppalError;
    }

    impl OutputPin for GpioWrapper<RppalOutputPin> {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.pin.set_low();
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.pin.set_high();
            Ok(())
        }
    }

    impl InputPin for GpioWrapper<RppalInputPin> {
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            Ok(self.pin.is_high())
        }

        fn is_low(&mut self) -> Result<bool, Self::Error> {
            Ok(self.pin.is_low())
        }
    }

    pub struct PwmWrapper {
        pwm: Pwm,
        max_duty: u16,
    }

    impl PwmWrapper {
        pub fn new(pwm: Pwm, max_duty: u16) -> Self {
            Self { pwm, max_duty }
        }
    }

    impl embedded_hal::pwm::ErrorType for PwmWrapper {
        type Error = RppalError;
    }

    impl SetDutyCycle for PwmWrapper {
        fn max_duty_cycle(&self) -> u16 {
            self.max_duty
        }

        fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
            let duty_percent = duty as f64 / self.max_duty as f64;
            self.pwm.set_duty_cycle(duty_percent).map_err(|_| RppalError)?;
            Ok(())
        }
    }

    pub type RppalMotorBuilder = MotorDriverBuilder<
        GpioWrapper<RppalOutputPin>,
        GpioWrapper<RppalOutputPin>,
        PwmWrapper,
        PwmWrapper
    >;

    impl RppalMotorBuilder {
        /// Create a new Raspberry Pi motor wrapper builder.
        /// 
        /// # Returns
        /// 
        /// A builder for simple motor control using rppal GPIO and PWM.
        /// 
        /// # Example
        /// 
        /// ```rust
        /// let motor = RppalMotorBuilder::new_rppal()
        ///     .with_dual_gpio_enable(&gpio, 23, 24)?
        ///     .with_pwm_channels(&pwm, 18, 19, 1000.0, 1000)?
        ///     .build()?;
        /// ```
        pub fn new_rppal() -> Self {
            MotorDriverBuilder::new()
        }

        pub fn with_gpio_enable(self, gpio: &::rppal::gpio::Gpio, pin: u8) -> Result<Self, ::rppal::gpio::Error> {
            Ok(self.with_single_enable(GpioWrapper::new(gpio.get(pin)?.into_output())))
        }

        pub fn with_dual_gpio_enable(self, gpio: &::rppal::gpio::Gpio, pin1: u8, pin2: u8) -> Result<Self, ::rppal::gpio::Error> {
            Ok(self.with_dual_enable(
                GpioWrapper::new(gpio.get(pin1)?.into_output()),
                GpioWrapper::new(gpio.get(pin2)?.into_output())
            ))
        }

        pub fn with_pwm_channel(self, channel: ::rppal::pwm::Channel, frequency: f64, max_duty: u16) -> Result<Self, ::rppal::pwm::Error> {
            let pwm = Pwm::with_frequency(channel, frequency, 0.0, ::rppal::pwm::Polarity::Normal, true)?;
            Ok(self.with_single_pwm(PwmWrapper::new(pwm, max_duty)))
        }

        pub fn with_dual_pwm_channels(
            self, 
            channel1: ::rppal::pwm::Channel, 
            channel2: ::rppal::pwm::Channel, 
            frequency: f64, 
            max_duty: u16
        ) -> Result<Self, ::rppal::pwm::Error> {
            let pwm1 = Pwm::with_frequency(channel1, frequency, 0.0, ::rppal::pwm::Polarity::Normal, true)?;
            let pwm2 = Pwm::with_frequency(channel2, frequency, 0.0, ::rppal::pwm::Polarity::Normal, true)?;
            Ok(self.with_dual_pwm(PwmWrapper::new(pwm1, max_duty), PwmWrapper::new(pwm2, max_duty)))
        }
    }
}

#[cfg(feature = "linux-embedded-hal")]
pub mod linux {
    use embedded_hal::digital::OutputPin;
    use embedded_hal::pwm::SetDutyCycle;
    use linux_embedded_hal::CdevPin;

    #[derive(Debug)]
    pub struct LinuxError;

    impl embedded_hal::pwm::Error for LinuxError {
        fn kind(&self) -> embedded_hal::pwm::ErrorKind {
            embedded_hal::pwm::ErrorKind::Other
        }
    }

    impl embedded_hal::digital::Error for LinuxError {
        fn kind(&self) -> embedded_hal::digital::ErrorKind {
            embedded_hal::digital::ErrorKind::Other
        }
    }

    pub struct GpioWrapper {
        pin: CdevPin,
    }

    impl GpioWrapper {
        pub fn new(pin: CdevPin) -> Self {
            Self { pin }
        }
    }

    impl embedded_hal::digital::ErrorType for GpioWrapper {
        type Error = LinuxError;
    }

    impl OutputPin for GpioWrapper {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.pin.set_value(0).map_err(|_| LinuxError)
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.pin.set_value(1).map_err(|_| LinuxError)
        }
    }

    pub struct PwmWrapper {
        chip: u32,
        number: u32,
        max_duty: u16,
    }

    impl PwmWrapper {
        pub fn new(chip: u32, number: u32, max_duty: u16) -> Self {
            Self { chip, number, max_duty }
        }
    }

    impl embedded_hal::pwm::ErrorType for PwmWrapper {
        type Error = LinuxError;
    }

    impl SetDutyCycle for PwmWrapper {
        fn max_duty_cycle(&self) -> u16 {
            self.max_duty
        }

        fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
            Ok(())
        }
    }
}