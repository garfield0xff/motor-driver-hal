//! STM32F103C8T6 motor control example
//! 
//! This example demonstrates motor control using the motor-driver-hal library
//! with STM32F103C8T6 (Blue Pill). Pin configuration:
//! - R_EN -> PB0 (Right motor enable)
//! - L_EN -> PB1 (Left motor enable) 
//! - R_PWM -> PA6 (Right motor PWM - TIM3_CH1)
//! - L_PWM -> PA7 (Left motor PWM - TIM3_CH2)
//! - VCC -> 5V
//! - GND -> GND

#![no_std]
#![no_main]

use panic_halt as _;
use motor_driver_hal::{HBridgeMotorDriver, MotorDriver};
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;

// Import STM32F1 HAL crate
use stm32f1xx_hal as hal;
use hal::{
    pac,
    prelude::*,
    gpio::{Output, PushPull},
    timer::pwm::PwmChannels,
    time::U32Ext,
};
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // Initialize STM32 peripherals
    let dp = pac::Peripherals::take().unwrap();
    let _cp = cortex_m::peripheral::Peripherals::take().unwrap();
    
    // Configure clocks
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut dp.FLASH.constrain().acr);
    
    // Configure GPIO pins
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    
    // Configure motor enable pins
    let r_en = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);  // Right motor enable - PB0
    let l_en = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);  // Left motor enable - PB1
    
    // Configure PWM pins for TIM3
    let r_pwm_pin = gpioa.pa6.into_alternate_push_pull(&mut gpioa.crl);  // TIM3 CH1 - PA6
    let l_pwm_pin = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);  // TIM3 CH2 - PA7
    
    // Configure TIM3 for PWM
    let mut pwm = dp.TIM3.pwm(
        (r_pwm_pin, l_pwm_pin),
        &mut rcc.apb1,
        1.khz(),
        clocks,
    );
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