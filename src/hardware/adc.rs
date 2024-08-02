//! ADC wrapper

use crate::hardware::{MAP_SIZE, mem::{peek, poke}};
use std::{
    fs::OpenOptions,
    io::{Error, Result as IoResult},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    ptr::null_mut,
    sync::OnceLock,
};

use libc::{mmap, munmap, MAP_FAILED, MAP_SHARED, O_RDWR, PROT_READ, PROT_WRITE};

pub const ADC_PAGE: usize = 0x80050000;
pub const ADC_SCHED_OFFSET: usize = 0x0004;
pub const ADC_VALUE_OFFSET: usize = 0x0050;
pub const ADC_CLEAR_OFFSET: usize = 0x0018;

static mut ADC_POINTER: OnceLock<*mut u32> = OnceLock::new();

fn get() -> Option<*mut u32> {
    unsafe { ADC_POINTER.get().cloned() }
}

pub fn init() -> IoResult<()> {
    if get().is_some() {
        return Ok(());
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(O_RDWR)
        .open("/dev/mem")?;

    let fd = file.as_raw_fd();

    let mmaped = unsafe {
        mmap(
            null_mut(),
            MAP_SIZE,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            fd,
            ADC_PAGE as _,
        )
    };

    if mmaped == MAP_FAILED {
        return Err(Error::last_os_error());
    }
    
    unsafe {
        ADC_POINTER.set(mmaped as *mut u32).unwrap();
    }

    Ok(())
}

pub fn read() -> u8 {
    if let Some(pointer) = get() {
        poke(pointer, ADC_SCHED_OFFSET, 0x1);

        {
            let has_high_bit = peek(pointer, ADC_VALUE_OFFSET) >= 0x80000000;
            // There isn't a delay here since in C (see https://github.com/Hixie/localbit/blob/master/localbit.c#L346) it works that way so I'll leave it like this in Rust too 
            while (peek(pointer, ADC_VALUE_OFFSET) >= 0x80000000) == has_high_bit { }
        }

        let mut value = peek(pointer, ADC_VALUE_OFFSET) & !0x80000000;
        poke(pointer, ADC_CLEAR_OFFSET, 0x1);
        value = if value <= 200 {
            0
        } else {
            (31 * value - 0x1838) / 11
        };

        // This could panic if value was out of bounds of u8 after dividing by 16, so it gets clamped to avoid that
        (value.clamp(0, 4095) / 16) as u8
    } else {
        println!("warning: no ADC page pointer found");
        0
    }
}

pub fn cleanup() {
    if let Some(pointer) = get() {
        unsafe { munmap(pointer as *mut _, MAP_SIZE) };
    }
}
