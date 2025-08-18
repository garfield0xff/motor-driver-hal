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

    let enc_a = GpioWrapper::new(gpio.get(25)?.into_output());
    let enc_b = GpioWrapper::new(gpio.get(26)?.into_output());

    let mut motor: HBridgeMotorDriver<GpioWrapper, GpioWrapper, PwmWrapper, PwmWrapper, GpioWrapper, GpioWrapper> = 
        HBridgeMotorDriver::dual_pwm_with_encoder(r_en, l_en, r_pwm, l_pwm, enc_a, enc_b, 1000);
    
    motor.initialize()?;
    motor.enable()?;

    println!("Running motor with encoder monitoring");
    motor.set_speed(100)?;
    
    for i in 1..=10 {
        println!("Step {}: Speed = {:?}, Direction = {:?}", 
                i, motor.get_speed()?, motor.get_direction()?);
        thread::sleep(Duration::from_millis(500));
    }

    motor.set_speed(-300)?;
    
    for i in 1..=5 {
        println!("Reverse Step {}: Speed = {:?}, Direction = {:?}", 
                i, motor.get_speed()?, motor.get_direction()?);
        thread::sleep(Duration::from_millis(500));
    }

    motor.stop()?;
    motor.disable()?;
    
    Ok(())
}