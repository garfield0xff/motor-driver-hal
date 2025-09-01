use motor_driver_hal::driver::rppal::RppalMotorDriverBuilder;
use motor_driver_hal::MotorDriver;
use rppal::gpio::Gpio;
use rppal::pwm::Channel;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpio = Gpio::new()?;
    
    let mut motor = RppalMotorDriverBuilder::new_rppal()
        .with_dual_gpio_enable(&gpio, 23, 24)?
        .with_dual_pwm_channels(Channel::Pwm1, Channel::Pwm2, 1000.0, 1000)?
        .build_and_init()?;
    
    motor.enable()?;

    println!("Forward direction");
    motor.set_direction(true)?;
    motor.set_speed(100)?;
    thread::sleep(Duration::from_secs(2));

    println!("Reverse direction");
    motor.set_direction(false)?;
    thread::sleep(Duration::from_secs(2));

    println!("Stop motor");
    motor.stop()?;
    thread::sleep(Duration::from_secs(1));

    motor.disable()?;
    
    Ok(())
}