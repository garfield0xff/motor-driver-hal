
#![cfg_attr(not(feature = "std"), no_std)]

pub mod driver;
pub mod error;
pub mod wrapper;

pub use driver::HBridgeMotorDriver;
pub use error::MotorDriverError;
pub use wrapper::{MotorDriverWrapper, MotorDriverBuilder, EnablePins, PwmChannels, MotorDirection};

#[cfg(feature = "rppal")]
pub use wrapper::rppal::{GpioWrapper, PwmWrapper};

pub trait MotorDriver {
    type Error;
    fn initialize(&mut self) -> Result<(), Self::Error>;
    fn set_speed(&mut self, speed: i16) -> Result<(), Self::Error>;
    fn set_direction(&mut self, forward: bool) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
    fn brake(&mut self) -> Result<(), Self::Error>;
    fn enable(&mut self) -> Result<(), Self::Error>;
    fn disable(&mut self) -> Result<(), Self::Error>;
    fn get_speed(&self) -> Result<i16, Self::Error>;
    fn get_direction(&self) -> Result<bool, Self::Error>;
    fn get_current(&self) -> Result<f32, Self::Error>;
    fn get_voltage(&self) -> Result<f32, Self::Error>;
    fn get_temperature(&self) -> Result<f32, Self::Error>;
    fn get_fault_status(&self) -> Result<u8, Self::Error>;
}