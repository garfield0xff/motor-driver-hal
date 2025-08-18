use motor_driver_hal::{HBridgeMotorDriver, MotorDriver, GpioWrapper, PwmWrapper};
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

    let mut motor: HBridgeMotorDriver<GpioWrapper, GpioWrapper, PwmWrapper, PwmWrapper, (), ()> = 
        HBridgeMotorDriver::dual_pwm(r_en, l_en, r_pwm, l_pwm, 1000);
    
    motor.initialize()?;
    motor.enable()?;

    let speeds = [200, 400, 600, 800, 1000];
    
    for &speed in &speeds {
        println!("Setting speed to: {}", speed);
        motor.set_speed(speed)?;
        thread::sleep(Duration::from_secs(1));
    }

    println!("Testing reverse speeds");
    for &speed in &speeds {
        println!("Setting speed to: -{}", speed);
        motor.set_speed(-speed)?;
        thread::sleep(Duration::from_secs(1));
    }

    motor.brake()?;
    thread::sleep(Duration::from_secs(1));
    
    motor.disable()?;
    
    Ok(())
}