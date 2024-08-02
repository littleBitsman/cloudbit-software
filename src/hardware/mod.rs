//! Contains all hardware wrappers.

use std::{
    fs::OpenOptions,
    io::Error as IoError, 
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt}
};

pub(self) const MAP_SIZE: usize = 0x1FFF;

pub mod adc;
pub mod led;
pub mod button;

pub fn init_all() -> Result<(), (&'static str, IoError)> {
    let devmem = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(2)
        .open("/dev/mem")
        .map_err(|v| ("failed to open /dev/mem", v))?;

    let fd = devmem.as_raw_fd();

    adc::init(fd).map_err(|v| ("ADC", v))?;
    button::init(fd).map_err(|v| ("Button", v))?;

    Ok(())
}

pub fn cleanup_all() {
    adc::cleanup();
    button::cleanup();
}

/// Memory module containing:
/// - [`peek`] (read memory at `page` offset by `offset`)
/// - [`poke`] (write to memory at `page` offset by `offset`, setting it to `value`)
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