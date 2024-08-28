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

use crate::hardware::mem::{map, peek};
use std::{io::Result as IoResult, sync::OnceLock};

const GPIO_PAGE: usize = 0x80018000;
const BUTTON_OFFSET: usize = 0x0610;

static mut GPIO_POINTER: OnceLock<*mut u32> = OnceLock::new();

fn get() -> Option<*mut u32> {
    unsafe { GPIO_POINTER.get().copied() }
}

/// Initalizes button memory
fn mem_init(page: *mut u32) {
    // See hardware/adc.rs at line 40 for more info on why this is commented out (FOR NOW.)
    // poke(page, 0x0124, 0x0000C000);
    // poke(page, 0x0718, 0x00000080);
    let _ = page; // this makes page used so it doesn't create build warnings
}

pub fn init(fd: i32) -> IoResult<()> {
    if get().is_some() {
        return Ok(());
    }

    let mmaped = map(fd, GPIO_PAGE as i64)?;

    unsafe { GPIO_POINTER.set(mmaped).unwrap() }

    mem_init(mmaped);

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
