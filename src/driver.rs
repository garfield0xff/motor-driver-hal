use crate::{MotorDriver, MotorDriverError};
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;
pub struct HBridgeMotorDriver<E1, E2, P1, P2, Enc1, Enc2> {
    enable1: E1,
    enable2: Option<E2>,
    pwm1: P1,
    pwm2: Option<P2>,
    encoder1: Option<Enc1>,
    encoder2: Option<Enc2>,
    max_duty: u16,
    current_speed: i16,
    direction: bool,
    initialized: bool,
}

impl<E1, E2, P1, P2, Enc1, Enc2> HBridgeMotorDriver<E1, E2, P1, P2, Enc1, Enc2>
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
            direction: true,
            initialized: false,
        }
    }
    
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
}

impl<E1, E2, P1, P2, Enc1, Enc2> MotorDriver for HBridgeMotorDriver<E1, E2, P1, P2, Enc1, Enc2>
where
    E1: OutputPin,
    E2: OutputPin,
    P1: SetDutyCycle,
    P2: SetDutyCycle,
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