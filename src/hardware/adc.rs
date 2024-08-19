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

//! ADC wrapper

use crate::hardware::mem::{map, peek, poke};
use std::{
    io::Result as IoResult,
    sync::OnceLock,
};

pub const ADC_PAGE: usize = 0x80050000;
pub const ADC_SCHED_OFFSET: usize = 0x0004;
pub const ADC_VALUE_OFFSET: usize = 0x0050;
pub const ADC_CLEAR_OFFSET: usize = 0x0018;

static mut ADC_POINTER: OnceLock<*mut u32> = OnceLock::new();

fn get() -> Option<*mut u32> {
    // SAFETY: TODO
    unsafe { ADC_POINTER.get().cloned() }
}

pub fn init(fd: i32) -> IoResult<()> {
    if get().is_some() {
        return Ok(())
    }

    let mmaped = map(fd, ADC_PAGE as i64)?;
    
    // SAFETY: TODO
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