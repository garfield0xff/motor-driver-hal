use crate::{MotorDriver, MotorDriverError};
use embedded_hal::digital::{OutputPin, InputPin};
use embedded_hal::pwm::SetDutyCycle;

#[derive(Debug)]
pub struct NoEncoder;

#[derive(Debug)]
pub struct NoEncoderError;

impl embedded_hal::digital::Error for NoEncoderError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

impl embedded_hal::digital::ErrorType for NoEncoder {
    type Error = NoEncoderError;
}

impl InputPin for NoEncoder {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Level {
    Low = 0,
    High = 1,
}

pub struct HBridgeMotorDriver<E1, E2, P1, P2, Enc1, Enc2> {
    enable1: E1,
    enable2: Option<E2>,
    pwm1: P1,
    pwm2: Option<P2>,
    encoder1: Option<Enc1>,
    encoder2: Option<Enc2>,
    max_duty: u16,
    current_speed: i16,
    pulse_count: i32,
    pulse_offset: i32,
    target_pulse: i32,
    ppr: u16,
    last_enc_a: Level,
    last_enc_b: Level,
    direction: bool,
    initialized: bool,
}

const QEM: [i8; 16] = [
     0, -1,  1,  0,
     1,  0,  0, -1,
    -1,  0,  0,  1,
     0,  1, -1,  0,
];

impl<E1, E2, P1, P2> HBridgeMotorDriver<E1, E2, P1, P2, NoEncoder, NoEncoder>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
{   
    pub fn single_pwm(enable: E1, pwm: P1, max_duty: u16) -> Self {
        Self {
            enable1: enable,
            enable2: None,
            pwm1: pwm,
            pwm2: None,
            encoder1: None,
            encoder2: None,
            max_duty,
            current_speed: 0,
            pulse_count: 0,
            pulse_offset: 0,
            target_pulse: 0,
            ppr: 0,
            last_enc_a: Level::Low,
            last_enc_b: Level::Low,
            direction: true,
            initialized: false,
        }
    }

    pub fn dual_pwm(enable1: E1, enable2: E2, pwm1: P1, pwm2: P2, max_duty: u16) -> Self {
        Self {
            enable1,
            enable2: Some(enable2),
            pwm1,
            pwm2: Some(pwm2),
            encoder1: None,
            encoder2: None,
            max_duty,
            current_speed: 0,
            pulse_count: 0,
            pulse_offset: 0,
            target_pulse: 0,
            ppr: 0,
            last_enc_a: Level::Low,
            last_enc_b: Level::Low,
            direction: true,
            initialized: false,
        }
    }
}

impl<E1, E2, P1, P2, Enc1, Enc2> HBridgeMotorDriver<E1, E2, P1, P2, Enc1, Enc2>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
    Enc1: InputPin,
    Enc2: InputPin,
{
    
    pub fn dual_pwm_with_encoder(enable1: E1, enable2: E2, pwm1: P1, pwm2: P2, encoder1: Enc1, encoder2: Enc2, max_duty: u16) -> Self {
        Self { 
            enable1, 
            enable2: Some(enable2), 
            pwm1,
            pwm2: Some(pwm2),
            encoder1: Some(encoder1),
            encoder2: Some(encoder2),
            max_duty,
            current_speed: 0,
            pulse_count: 0,
            pulse_offset: 0,
            target_pulse: 0,
            ppr: 0,
            last_enc_a: Level::Low,
            last_enc_b: Level::Low,
            direction: true,
            initialized: false 
        }
    }

    fn update_pwm(&mut self) -> Result<(), MotorDriverError> {
        let duty = if self.current_speed < 0 {
            (-self.current_speed as u16).min(self.max_duty)
        } else {
            (self.current_speed as u16).min(self.max_duty)
        };

        match (&mut self.pwm2, self.direction) {
            (Some(pwm2), true) => {
                self.pwm1.set_duty_cycle(duty).map_err(|_| MotorDriverError::PwmError)?;
                pwm2.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
            }
            (Some(pwm2), false) => {
                self.pwm1.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
                pwm2.set_duty_cycle(duty).map_err(|_| MotorDriverError::PwmError)?;
            }
            (None, _) => {
                self.pwm1.set_duty_cycle(duty).map_err(|_| MotorDriverError::PwmError)?;
            }
        }
        Ok(())
    }

    pub fn read_encoder(&mut self) -> Result<(), MotorDriverError> {
        if let (Some(ref mut enc_a), Some(ref mut enc_b)) = (&mut self.encoder1, &mut self.encoder2) {
            let level_a = if enc_a.is_high().map_err(|_| MotorDriverError::GpioError)? { 
                Level::High 
            } else { 
                Level::Low 
            };
            let level_b = if enc_b.is_high().map_err(|_| MotorDriverError::GpioError)? { 
                Level::High 
            } else { 
                Level::Low 
            };

            let index = ((self.last_enc_a as u8) << 3)
                      | ((self.last_enc_b as u8) << 2)
                      | ((level_a as u8) << 1)
                      | (level_b as u8);
            
            self.pulse_count += QEM[index as usize] as i32;
            self.last_enc_a = level_a;
            self.last_enc_b = level_b;
            
            Ok(())
        } else {
            Err(MotorDriverError::HardwareFault)
        }
    }

    pub fn get_pulse_count(&self) -> i32 {
        self.pulse_count - self.pulse_offset
    }

    pub fn reset_encoder(&mut self) {
        self.pulse_offset = self.pulse_count;
    }

    pub fn set_target_pulse(&mut self, target: i32) {
        self.target_pulse = target;
    }
}

impl<E1, E2, P1, P2, Enc1, Enc2> MotorDriver for HBridgeMotorDriver<E1, E2, P1, P2, Enc1, Enc2>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
    Enc1: InputPin,
    Enc2: InputPin,
{
    type Error = MotorDriverError;
    
    fn initialize(&mut self) -> Result<(), Self::Error> {
        self.enable1.set_low().map_err(|_| MotorDriverError::GpioError)?;
        if let Some(ref mut enable2) = self.enable2 {
            enable2.set_low().map_err(|_| MotorDriverError::GpioError)?;
        }
        
        self.pwm1.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
        if let Some(ref mut pwm2) = self.pwm2 {
            pwm2.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
        }
        
        self.initialized = true;
        Ok(())
    }

    fn set_speed(&mut self, speed: i16) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        if speed.unsigned_abs() > self.max_duty {
            return Err(MotorDriverError::InvalidSpeed);
        }
        
        self.current_speed = speed;
        self.direction = speed >= 0;
        
        self.update_pwm()
    }

    fn set_direction(&mut self, forward: bool) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.direction = forward;
        self.update_pwm()
    }

    fn stop(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.current_speed = 0;
        self.pwm1.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
        if let Some(ref mut pwm2) = self.pwm2 {
            pwm2.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
        }
        Ok(())
    }

    fn brake(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.current_speed = 0;
        if let Some(ref mut pwm2) = self.pwm2 {
            self.pwm1.set_duty_cycle(self.max_duty).map_err(|_| MotorDriverError::PwmError)?;
            pwm2.set_duty_cycle(self.max_duty).map_err(|_| MotorDriverError::PwmError)?;
        } else {
            self.pwm1.set_duty_cycle(0).map_err(|_| MotorDriverError::PwmError)?;
        }
        Ok(())
    }

    fn enable(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.enable1.set_high().map_err(|_| MotorDriverError::GpioError)?;
        if let Some(ref mut enable2) = self.enable2 {
            enable2.set_high().map_err(|_| MotorDriverError::GpioError)?;
        }
        Ok(())
    }

    fn disable(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.enable1.set_low().map_err(|_| MotorDriverError::GpioError)?;
        if let Some(ref mut enable2) = self.enable2 {
            enable2.set_low().map_err(|_| MotorDriverError::GpioError)?;
        }
        Ok(())
    }

    fn get_speed(&self) -> Result<i16, Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        Ok(self.current_speed)
    }

    fn get_direction(&self) -> Result<bool, Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        Ok(self.direction)
    }

    fn set_ppr(&mut self, ppr: i16) -> Result<bool, Self::Error> {        
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        if ppr <= 0 {
            return Err(MotorDriverError::InvalidSpeed);
        }
        self.ppr = ppr as u16;
        Ok(true)
    }

    fn check_ppr(&mut self) -> Result<(), Self::Error> {
        if self.ppr == 0 {
            return Err(MotorDriverError::NotInitialized);
        }
        
        self.read_encoder()?;
        
        let current_pulse = self.get_pulse_count();
        let current_rotation_pulse = current_pulse % (self.ppr as i32);
        let target_rotation_pulse = self.target_pulse % (self.ppr as i32);
        
        if current_rotation_pulse == target_rotation_pulse {
            Ok(())
        } else {
            Err(MotorDriverError::HardwareFault)
        }
    }


    fn get_current(&self) -> Result<f32, Self::Error> {
        Err(MotorDriverError::HardwareFault)
    }

    fn get_voltage(&self) -> Result<f32, Self::Error> {
        Err(MotorDriverError::HardwareFault)
    }

    fn get_temperature(&self) -> Result<f32, Self::Error> {
        Err(MotorDriverError::HardwareFault)
    }

    fn get_fault_status(&self) -> Result<u8, Self::Error> {
        if !self.initialized {
            return Err(MotorDriverError::NotInitialized);
        }
        Ok(0)
    }
}

