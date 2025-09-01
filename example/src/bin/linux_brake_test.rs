use motor_driver_hal::driver::linux::LinuxMotorDriverBuilder;
use motor_driver_hal::MotorDriver;
use linux_embedded_hal::gpio_cdev::Chip;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chip = Chip::new("/dev/gpiochip0")?;
    
    let mut motor = LinuxMotorDriverBuilder::new_linux()
        .with_dual_gpio_enable(&mut chip, 23, 24)?
        .with_dual_pwm_channels(0, 0, 1, 1000)
        .build_and_init()?;
    
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