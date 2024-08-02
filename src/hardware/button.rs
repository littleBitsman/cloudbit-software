//! Button wrapper

use crate::hardware::{MAP_SIZE, mem::peek};
use std::{
    io::{Error, Result as IoResult},
    ptr::null_mut, 
    sync::OnceLock
};

use libc::{mmap, munmap, MAP_FAILED, MAP_SHARED, PROT_READ, PROT_WRITE};

const GPIO_PAGE: usize = 0x80018000;
const BUTTON_OFFSET: usize = 0x0610;

static mut GPIO_POINTER: OnceLock<*mut u32> = OnceLock::new();

fn get() -> Option<*mut u32> {
    unsafe { GPIO_POINTER.get().cloned() }
}

pub fn init(fd: i32) -> IoResult<()> {
    if get().is_some() {
        return Ok(());
    }

    let mmaped = unsafe {
        mmap(
            null_mut(),
            MAP_SIZE,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            fd,
            GPIO_PAGE as _,
        )
    };

    if mmaped == MAP_FAILED {
        return Err(Error::last_os_error());
    }

    unsafe {
        GPIO_POINTER.set(mmaped as *mut u32).unwrap();
    }

    Ok(())
}

pub fn read() -> bool {
    if get().is_none() {
        println!("warning: no button page pointer found");
    }
    get().is_some_and(|v| {
        // For some reason this is inverted
        (peek(v, BUTTON_OFFSET) & 0x80) == 0
    })
}

pub fn cleanup() {
    if let Some(pointer) = get() {
        unsafe { munmap(pointer as *mut _, MAP_SIZE) };
    }
}