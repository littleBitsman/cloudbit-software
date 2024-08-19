// This file is part of cloudbit-software.
//
// cloudbit-software - an alternative software for the littleBits cloudBit.
//
// Copyright (C) 2024 littleBitsman
//
// cloudbit-software is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// cloudbit-software is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program. If not, see https://www.gnu.org/licenses/.

//! Contains all hardware wrappers.

use std::{
    fs::OpenOptions,
    io::Error as IoError,
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
};

pub mod adc;
pub mod button;
pub mod led;

pub fn init_all() -> Result<(), (&'static str, IoError)> {
    let devmem = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(2) // O_RDWR = 2
        .open("/dev/mem")
        .map_err(|v| ("failed to open /dev/mem", v))?;

    let fd = devmem.as_raw_fd();

    adc::init(fd).map_err(|v| ("ADC", v))?;
    button::init(fd).map_err(|v| ("Button", v))?;

    Ok(())
}

/// Memory module containing:
/// - [`peek`] (read memory at `page` offset by `offset`)
/// - [`poke`] (write to memory at `page` offset by `offset`, setting it to `value`)
mod mem {
    use std::{io::{Error as IoError, Result as IoResult}, ptr::null_mut};
    use libc::{mmap, MAP_FAILED, MAP_SHARED, PROT_READ, PROT_WRITE};

    pub const MAP_SIZE: usize = 0x1FFF;

    /// Reads memory at (page + offset) with [`std::ptr::read_volatile`]
    pub fn peek(page: *mut u32, offset: usize) -> u32 {
        // TODO: make this safety comment better
        // SAFETY: it is up to the caller to ensure that the pointer is valid
        // (see std::ptr::read_volatile)
        unsafe { page.byte_add(offset).read_volatile() }
    }

    /// Sets memory at (page + offset) with [`std::ptr::write_volatile`]
    pub fn poke(page: *mut u32, offset: usize, value: u32) {
        // TODO: make this safety comment better
        // SAFETY: it is up to the caller to ensure that the pointer is valid
        // (see std::ptr::write_volatile)
        unsafe { page.byte_add(offset).write_volatile(value) }
    }

    pub fn map<T>(fd: i32, offset: i64) -> IoResult<*mut T> {
        // SAFETY: FFI functions are marked unsafe since the compiler cannot verify
        // behavior, but mmap is OK (or should be)
        let ptr = unsafe {
            mmap(
                null_mut(),
                MAP_SIZE,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd,
                offset,
            )
        };

        if ptr == MAP_FAILED {
            Err(IoError::last_os_error())
        } else {
            Ok(ptr as *mut T)
        }
    }
}
