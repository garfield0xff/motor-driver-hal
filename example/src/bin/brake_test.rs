use motor_driver_hal::{HBridgeMotorDriver, MotorDriver, NoEncoder};
use motor_driver_hal::wrapper::rppal::{GpioWrapper, PwmWrapper};
use rppal::gpio::Gpio;
use rppal::pwm::{Channel, Pwm, Polarity};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpio = Gpio::new()?;
    
    let r_en = GpioWrapper::new(gpio.get(23)?.into_output());
    let l_en = GpioWrapper::new(gpio.get(24)?.into_output());
    
    let r_pwm = PwmWrapper::new(
        Pwm::with_frequency(Channel::Pwm1, 1000.0, 0.0, Polarity::Normal, true)?, 
        1000
    );
    let l_pwm = PwmWrapper::new(
        Pwm::with_frequency(Channel::Pwm2, 1000.0, 0.0, Polarity::Normal, true)?, 
        1000
    );

    let mut motor: HBridgeMotorDriver<GpioWrapper<rppal::gpio::OutputPin>, GpioWrapper<rppal::gpio::OutputPin>, PwmWrapper, PwmWrapper, NoEncoder, NoEncoder> = 
        HBridgeMotorDriver::dual_pwm(r_en, l_en, r_pwm, l_pwm, 1000);
    
    motor.initialize()?;
    motor.enable()?;

    println!("Running motor at full speed");
    motor.set_speed(1000)?;
    thread::sleep(Duration::from_secs(3));

    println!("Testing coast stop");
    motor.stop()?;
    thread::sleep(Duration::from_secs(2));

    println!("Running motor again");
    motor.set_speed(1000)?;
    thread::sleep(Duration::from_secs(3));

    println!("Testing brake stop");
    motor.brake()?;
    thread::sleep(Duration::from_secs(2));

    motor.disable()?;
    
    Ok(())
}