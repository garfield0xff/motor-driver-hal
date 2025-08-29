use motor_driver_hal::wrapper::rppal::{GpioWrapper, PwmWrapper};
use motor_driver_hal::{HBridgeMotorDriver, MotorDriver};
use rppal::gpio::{Gpio, InputPin, OutputPin};
use rppal::pwm::{Channel, Pwm, Polarity};
use std::thread;
use std::time::Duration;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

fn shutdown_motor(motor: &Arc<Mutex<HBridgeMotorDriver<GpioWrapper<OutputPin>, GpioWrapper<OutputPin>, PwmWrapper, PwmWrapper, GpioWrapper<InputPin>, GpioWrapper<InputPin>>>>) {
    if let Ok(mut m) = motor.lock() {
        let _ = m.stop();
        let _ = m.disable();
        println!("Motor stopped safely.");
    }
}

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

    let enc_a = GpioWrapper::new(gpio.get(25)?.into_input_pullup());
    let enc_b = GpioWrapper::new(gpio.get(8)?.into_input_pullup());

    let motor: HBridgeMotorDriver<GpioWrapper<OutputPin>, GpioWrapper<OutputPin>, PwmWrapper, PwmWrapper, GpioWrapper<InputPin>, GpioWrapper<InputPin>> = 
        HBridgeMotorDriver::dual_pwm_with_encoder(r_en, l_en, r_pwm, l_pwm, enc_a, enc_b, 1000);

    let motor = Arc::new(Mutex::new(motor));
    motor.lock().unwrap().initialize()?;
    
    // Setup Ctrl+C handler
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
    motor.lock().unwrap().reset_encoder();
    
    // Run motor at 10% speed
    motor.lock().unwrap().set_speed(100)?;
    
    let mut last_count = 0;
    let mut reading_count = 0;
    let start_time = std::time::Instant::now();
    
    // Encoder reading thread
    let motor_read = motor.clone();
    let running_read = running.clone();
    let encoder_thread = thread::spawn(move || {
        while running_read.load(Ordering::SeqCst) {
            if let Ok(mut m) = motor_read.lock() {
                let _ = m.read_encoder();
            }
            thread::sleep(Duration::from_micros(100)); // Faster reading rate
        }
    });
    
    // Main loop: Display encoder values
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100)); // Display every 100ms
        
        let current_count = motor.lock().unwrap().get_pulse_count();
        let delta = current_count - last_count;
        let elapsed = start_time.elapsed().as_secs_f64();
        let pulses_per_sec = if elapsed > 0.0 {
            (current_count as f64 / elapsed).round() as i32
        } else {
            0
        };
        
        reading_count += 1;
        
        print!("\r[{:4}] Pulses: {:6} | Delta: {:4} | Speed: {:4} pps | Time: {:.1}s", 
               reading_count, current_count, delta, pulses_per_sec, elapsed);
        io::stdout().flush()?;
        
        last_count = current_count;
        
        // Auto-exit after 1 minute
        if elapsed > 60.0 {
            println!("\n\n1 minute elapsed, test completed");
            running.store(false, Ordering::SeqCst);
            break;
        }
    }
    
    // Wait for encoder thread to finish
    let _ = encoder_thread.join();
    
    // Final results
    let final_count = motor.lock().unwrap().get_pulse_count();
    let total_time = start_time.elapsed().as_secs_f64();
    
    println!("\n\n=== Test Results ===");
    println!("Total pulses: {} pulses", final_count);
    println!("Total time: {:.1} seconds", total_time);
    println!("Average speed: {:.1} pulses/second", final_count as f64 / total_time);
    
    // Safely stop motor on normal exit
    shutdown_motor(&motor);
    
    println!("\nMotor stopped, program terminated");
    Ok(())
}