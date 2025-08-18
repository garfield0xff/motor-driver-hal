use rppal::gpio::{Gpio, Trigger};
use rppal::pwm::{Channel, Pwm, Polarity};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;
use std::time::Duration;

fn main() {
    let gpio = Gpio::new().expect("GPIO 초기화 실패");
    
    // Enable 핀
    let mut r_en = gpio.get(23).unwrap().into_output();  // GPIO23
    let mut l_en = gpio.get(24).unwrap().into_output();  // GPIO24
    
    // PWM 설정
    let l_pwm = Pwm::with_frequency(Channel::Pwm1, 1000.0, 0.0, Polarity::Normal, true)
        .expect("L_PWM 초기화 실패");
    let r_pwm = Pwm::with_frequency(Channel::Pwm2, 1000.0, 0.0, Polarity::Normal, true)
        .expect("R_PWM 초기화 실패");

    // 엔코더 입력 핀
    let mut enc_a = gpio.get(25).unwrap().into_input(); // C1
    let mut enc_b = gpio.get(8).unwrap().into_input();  // C2
    
    // 펄스 카운터 (양방향, signed int)
    let pulse_count = Arc::new(AtomicI32::new(0));
    let pulse_count_clone = Arc::clone(&pulse_count);

    // 마지막 A 채널 상태 저장
    let last_a = Arc::new(AtomicI32::new(0));
    let last_a_clone = Arc::clone(&last_a);
    
    // 인터럽트 설정
    enc_a.set_interrupt(Trigger::Both, None).unwrap();
    
    // 엔코더 모니터링 스레드
    let monitor_handle = thread::spawn(move || {
        loop {
            match enc_a.poll_interrupt(false, None) {
                Ok(Some(_)) => {
                    let current_a = if enc_a.is_high() { 1 } else { 0 };
                    let current_b = if enc_b.is_high() { 1 } else { 0 };
                    let prev_a = last_a_clone.load(Ordering::Relaxed);
                    
                    // 상승 엣지
                    if current_a == 1 && prev_a == 0 {
                        if current_b == 0 {
                            pulse_count_clone.fetch_add(1, Ordering::Relaxed); // 정방향
                        } else {
                            pulse_count_clone.fetch_sub(1, Ordering::Relaxed); // 역방향
                        }
                    }
                    // 하강 엣지
                    else if current_a == 0 && prev_a == 1 {
                        if current_b == 1 {
                            pulse_count_clone.fetch_add(1, Ordering::Relaxed); // 정방향
                        } else {
                            pulse_count_clone.fetch_sub(1, Ordering::Relaxed); // 역방향
                        }
                    }
                    
                    last_a_clone.store(current_a, Ordering::Relaxed);
                }
                Ok(None) => {}
                Err(e) => {
                    println!("인터럽트 에러: {:?}", e);
                    break;
                }
            }
            thread::sleep(Duration::from_micros(10));
        }
    });

    // 모터 활성화
    r_en.set_high();
    l_en.set_high();
    println!("모터 활성화");

    let speeds = vec![0.1, 0.3];
    
    // === 정방향 테스트 ===
    println!("\n=== 정방향 테스트 ===");
    for speed in &speeds {
        let percent = (speed * 100.0) as i32;
        println!("정방향 {}%", percent);
        pulse_count.store(0, Ordering::Relaxed);
        r_pwm.set_duty_cycle(*speed).unwrap();
        l_pwm.set_duty_cycle(0.0).unwrap();
        
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(100));
            let count = pulse_count.load(Ordering::Relaxed);
            let direction = if count >= 0 { "CW" } else { "CCW" };
            print!("\r펄스: {}  방향: {}", count, direction);
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        println!();
    }
    
    // 정지
    r_pwm.set_duty_cycle(0.0).unwrap();
    l_pwm.set_duty_cycle(0.0).unwrap();
    thread::sleep(Duration::from_secs(1));

    // === 역방향 테스트 ===
    println!("\n=== 역방향 테스트 ===");
    for speed in &speeds {
        let percent = (speed * 100.0) as i32;
        println!("역방향 {}%", percent);
        pulse_count.store(0, Ordering::Relaxed);
        r_pwm.set_duty_cycle(0.0).unwrap();
        l_pwm.set_duty_cycle(*speed).unwrap();
        
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(100));
            let count = pulse_count.load(Ordering::Relaxed);
            let direction = if count >= 0 { "CW" } else { "CCW" };
            print!("\r펄스: {}  방향: {}", count, direction);
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        println!();
    }

    // 모터 완전 정지
    r_pwm.set_duty_cycle(0.0).unwrap();
    l_pwm.set_duty_cycle(0.0).unwrap();
    r_en.set_low();
    l_en.set_low();
    println!("\n모터 정지");

    // 모니터 스레드 종료
    drop(monitor_handle);
}
