//! Basic motor control example
//! 
//! This example demonstrates basic motor operations including initialization,
//! enabling, speed control, and stopping using the motor-driver-hal library
//! with Raspberry Pi GPIO and PWM interfaces.

use motor_driver_hal::{HBridgeMotorDriver, MotorDriver};
use rppal::gpio::Gpio;
use rppal::pwm::{Channel, Pwm, Polarity};
use std::thread;
use std::time::Duration;
use embedded_hal::pwm::{SetDutyCycle, ErrorType};
use embedded_hal::digital::OutputPin;

#[derive(Debug)]
struct PwmError;

impl embedded_hal::pwm::Error for PwmError {
    fn kind(&self) -> embedded_hal::pwm::ErrorKind {
        embedded_hal::pwm::ErrorKind::Other
    }
}

struct PwmWrapper {
    pwm: Pwm,
    max_duty: u16,
}

impl PwmWrapper {
    fn new(channel: Channel) -> Result<Self, Box<dyn std::error::Error>> {
        let pwm = Pwm::with_frequency(channel, 1000.0, 0.0, Polarity::Normal, true)?;
        Ok(Self {
            pwm,
            max_duty: 1000,
        })
    }
}

impl ErrorType for PwmWrapper {
    type Error = PwmError;
}

impl SetDutyCycle for PwmWrapper {
    fn max_duty_cycle(&self) -> u16 {
        self.max_duty
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        let duty_percent = duty as f64 / self.max_duty as f64;
        self.pwm.set_duty_cycle(duty_percent).map_err(|_| PwmError)?;
        Ok(())
    }
}

#[derive(Debug)]
struct GpioError;

impl embedded_hal::digital::Error for GpioError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

struct GpioOutputWrapper {
    pin: rppal::gpio::OutputPin,
}

impl GpioOutputWrapper {
    fn new(pin: rppal::gpio::OutputPin) -> Self {
        Self { pin }
    }
}

impl embedded_hal::digital::ErrorType for GpioOutputWrapper {
    type Error = GpioError;
}

impl OutputPin for GpioOutputWrapper {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high();
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let gpio = Gpio::new()?;
    
    let r_en = GpioOutputWrapper::new(gpio.get(23)?.into_output());
    let l_en = GpioOutputWrapper::new(gpio.get(24)?.into_output());
    
    let l_pwm = PwmWrapper::new(Channel::Pwm1)?;
    let r_pwm = PwmWrapper::new(Channel::Pwm2)?;

    let mut motor = HBridgeMotorDriver::dual_pwm(
        r_en, 
        l_en, 
        r_pwm, 
        l_pwm, 
        1000
    );
    
    motor.initialize()?;
    motor.enable()?;

    thread::sleep(Duration::from_secs(1));

    motor.set_speed(300)?;
    thread::sleep(Duration::from_secs(3));

    motor.stop()?;
    thread::sleep(Duration::from_secs(1));

    motor.disable()?;
    
    Ok(())
}