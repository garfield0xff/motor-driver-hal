use motor_driver_hal::driver::linux::LinuxMotorDriverBuilder;
use motor_driver_hal::MotorDriver;
use linux_embedded_hal::gpio_cdev::Chip;
use std::thread;
use std::time::Duration;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

fn shutdown_motor<T: MotorDriver>(motor: &Arc<Mutex<T>>) {
    if let Ok(mut m) = motor.lock() {
        let _ = m.stop();
        let _ = m.disable();
        println!("Motor stopped safely.");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chip = Chip::new("/dev/gpiochip0")?;

    let motor = LinuxMotorDriverBuilder::new_linux()
        .with_dual_gpio_enable(&mut chip, 23, 24)?
        .with_dual_pwm_channels(0, 0, 1, 1000)
        .build_and_init()?;

    let motor = Arc::new(Mutex::new(motor));
    
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let motor_clone = motor.clone();
    
    ctrlc::set_handler(move || {
        println!("\n\nCtrl+C detected! Shutting down safely...");
        r.store(false, Ordering::SeqCst);
        
        shutdown_motor(&motor_clone);
        std::process::exit(0);
    })?;
    
    motor.lock().unwrap().enable()?;
    
    println!("Starting encoder measurement...");
    
    motor.lock().unwrap().set_speed(100)?;
    
    let mut reading_count = 0;
    let start_time = std::time::Instant::now();
    
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
        
        let elapsed = start_time.elapsed().as_secs_f64();
        
        reading_count += 1;
        
        print!("\r[{:4}] Speed: 100 | Time: {:.1}s", 
               reading_count, elapsed);
        io::stdout().flush()?;
        
        if elapsed > 60.0 {
            println!("\n\n1 minute elapsed, test completed");
            running.store(false, Ordering::SeqCst);
            break;
        }
    }
    
    let total_time = start_time.elapsed().as_secs_f64();
    
    println!("\n\n=== Test Results ===");
    println!("Total time: {:.1} seconds", total_time);
    
    shutdown_motor(&motor);
    
    println!("\nMotor stopped, program terminated");
    Ok(())
}