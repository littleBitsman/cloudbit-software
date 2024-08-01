//! Contains all hardware wrappers.

use std::io::Error as IoError;

pub(self) const MAP_SIZE: usize = 0x1FFF;

pub mod adc;
pub mod led;
pub mod button;

pub fn init_all() -> Result<(), (&'static str, IoError)> {
    adc::init().map_err(|v| ("ADC", v))?;
    button::init().map_err(|v| ("Button", v))?;

    Ok(())
}

pub fn cleanup_all() {
    adc::cleanup();
    button::cleanup();
}