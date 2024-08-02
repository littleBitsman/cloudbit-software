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

pub(self) mod mem {
    /// Reads (page + offset) with read_volatile
    pub fn peek(page: *mut u32, offset: usize) -> u32 {
        unsafe { page.byte_add(offset).read_volatile() }
    }

    /// Sets (page + offset) with write_volatile
    pub fn poke(page: *mut u32, offset: usize, value: u32) {
        unsafe { page.byte_add(offset).write_volatile(value) }
    }
}