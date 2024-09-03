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
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt}
};

pub mod adc;
pub mod button;
pub mod dac;
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
    dac::init(fd).map_err(|v| ("DAC", v))?;
    led::init(fd).map_err(|v| ("DAC", v))?;

    Ok(())
}

/// Memory module containing:
/// - [`peek`] (read memory at `page` offset by `offset`)
/// - [`poke`] (write to memory at `page` offset by `offset`, setting it to `value`)
/// - [`map`] (wrapper for [`libc::mmap`])
mod mem {
    use libc::{mmap, MAP_FAILED, MAP_SHARED, PROT_READ, PROT_WRITE};
    use std::{
        io::{Error as IoError, Result as IoResult},
        ptr::null_mut
    };

    pub const MAP_SIZE: usize = 0x1FFF;

    /// Reads memory at (page + offset) with [`std::ptr::read_volatile`].
    ///
    /// The equivalent of this function in C is as follows:
    /// ```c
    /// uint32_t value = *(volatile uint32_t *)(base_address + offset);
    /// ```
    ///
    /// This function is not marked as `unsafe` to avoid requiring `unsafe` blocks
    /// every time it is used. However, it does involve `unsafe` operations internally.
    ///
    /// # Safety
    /// - The `page` pointer must be non-null, properly aligned, and point to a valid memory-mapped I/O region.
    /// - The `offset` must be within the bounds of the mapped memory region.
    /// - The caller must ensure that reading from this memory address does not cause any unintended side effects
    ///   or undefined behavior, particularly in the context of hardware interaction.
    pub fn peek(page: *mut u32, offset: usize) -> u32 {
        // SAFETY: The caller must guarantee that `page` is a valid, non-null pointer
        // pointing to a memory region that can be safely read from, and that `offset`
        // is within the bounds of that memory region. The operation will perform a
        // volatile read to ensure the read is not optimized away by the compiler.
        unsafe { page.byte_add(offset).read_volatile() }
    }

    /// Sets memory at (page + offset) with [`std::ptr::write_volatile`].
    ///
    /// The C counterpart to this function is as follows:
    /// ```c
    /// *(volatile uint32_t *)(base_address + offset) = value;
    /// ```
    ///
    /// This function is not marked as `unsafe` to avoid requiring `unsafe` blocks
    /// every time it is used. However, it does involve `unsafe` operations internally.
    ///
    /// # Safety
    /// - The `page` pointer must be non-null, properly aligned, and point to a valid memory-mapped I/O region.
    /// - The `offset` must be within the bounds of the mapped memory region.
    /// - The caller must ensure that writing to this memory address does not cause any unintended side effects
    ///   or undefined behavior, particularly in the context of hardware interaction.
    pub fn poke(page: *mut u32, offset: usize, value: u32) {
        // SAFETY: The caller must guarantee that `page` is a valid, non-null pointer
        // pointing to a memory region that can be safely written to, and that `offset`
        // is within the bounds of that memory region. The operation will perform a
        // volatile write to ensure the write is not optimized away by the compiler.
        unsafe { page.byte_add(offset).write_volatile(value) }
    }

    /// Maps a file (defined by the file descriptor `fd`) into memory,
    /// at offset `offset`.
    ///
    /// This function is not marked as `unsafe` to avoid requiring `unsafe` blocks
    /// every time it is used. However, it does involve `unsafe` operations internally.
    ///
    /// # Safety
    /// The `offset` must be aligned and within the bounds of the file defined
    /// by the file descriptor.
    pub fn map<T>(fd: i32, offset: i64) -> IoResult<*mut T> {
        // SAFETY: FFI functions are marked unsafe since the compiler cannot verify
        // behavior, but mmap is OK (or should be).
        // This function assumes that `offset` is aligned and is valid.
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
            Ok(ptr.cast())
        }
    }
}
