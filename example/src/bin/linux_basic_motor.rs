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
        .with_initial_speed(300)
        .build_and_init()?;
    
    motor.enable()?;
    thread::sleep(Duration::from_secs(1));

    motor.set_speed(300)?;
    thread::sleep(Duration::from_secs(3));

    motor.stop()?;
    thread::sleep(Duration::from_secs(1));

    motor.disable()?;
    
    Ok(())
}