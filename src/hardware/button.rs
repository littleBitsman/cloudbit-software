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