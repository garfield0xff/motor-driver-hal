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