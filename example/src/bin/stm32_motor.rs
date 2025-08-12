//! STM32 motor control example
//! 
//! This example demonstrates motor control using the motor-driver-hal library
//! with STM32 HAL interfaces. This example uses no_std and is designed for
//! STM32 microcontrollers.

#![no_std]
#![no_main]

use panic_halt as _;
use motor_driver_hal::{HBridgeMotorDriver, MotorDriver};
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;

// Import your specific STM32 HAL crate
use stm32f4xx_hal as hal;
use hal::{
    pac,
    prelude::*,
    gpio::{Output, PushPull, Pin},
    timer::{pwm::PwmChannels, Channel1, Channel2},
    time::Hertz,
};
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // Initialize STM32 peripherals
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    
    // Configure clocks
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    
    // Configure GPIO pins
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    
    // Configure motor enable pins - MODIFY PIN NUMBERS AS NEEDED
    let r_en = gpioa.pa0.into_push_pull_output();  // Right motor enable pin - 예: PA0
    let l_en = gpioa.pa1.into_push_pull_output();  // Left motor enable pin - 예: PA1
    
    // Configure PWM timer and channels - MODIFY PIN NUMBERS AS NEEDED
    let pins = (
        gpioa.pa2.into_alternate(),  // TIM2 CH1 - Right motor PWM - 예: PA2
        gpioa.pa3.into_alternate(),  // TIM2 CH2 - Left motor PWM - 예: PA3
    );
    
    let mut pwm = dp.TIM2.pwm_hz(pins, 1.kHz(), &clocks);
    let (mut r_pwm, mut l_pwm) = pwm.split();
    
    // Enable PWM channels
    r_pwm.enable();
    l_pwm.enable();
    
    // Set PWM duty cycle to maximum
    let max_duty = r_pwm.get_max_duty();
    r_pwm.set_duty(0);
    l_pwm.set_duty(0);
    
    // Create motor driver instance
    let mut motor = HBridgeMotorDriver::dual_pwm(
        r_en,
        l_en, 
        r_pwm,
        l_pwm,
        max_duty
    );
    
    // Initialize and control motor
    if let Ok(()) = motor.initialize() {
        if let Ok(()) = motor.enable() {
            // Forward motion at 30% speed
            let _ = motor.set_speed((max_duty as i16) * 30 / 100);
            
            // Simple delay (replace with proper delay implementation)
            for _ in 0..1_000_000 {
                cortex_m::asm::nop();
            }
            
            // Reverse motion at 50% speed
            let _ = motor.set_speed(-((max_duty as i16) * 50 / 100));
            
            // Delay
            for _ in 0..1_000_000 {
                cortex_m::asm::nop();
            }
            
            // Stop motor
            let _ = motor.stop();
            
            // Delay
            for _ in 0..500_000 {
                cortex_m::asm::nop();
            }
            
            // Brake
            let _ = motor.brake();
            
            // Disable motor
            let _ = motor.disable();
        }
    }
    
    // Main application loop
    loop {
        // Your main application logic here
        cortex_m::asm::wfi(); // Wait for interrupt
    }
}