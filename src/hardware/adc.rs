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

use crate::hardware::mem::{map, peek, poke, StaticPtr};
use std::io::Result as IoResult;

pub const ADC_PAGE: usize = 0x80050000;
pub const ADC_SCHED_OFFSET: usize = 0x0004;
pub const ADC_VALUE_OFFSET: usize = 0x0050;
pub const ADC_CLEAR_OFFSET: usize = 0x0018;

static ADC_POINTER: StaticPtr<u32> = StaticPtr::new();

fn get() -> Option<*mut u32> {
    ADC_POINTER.get()
}

/// Initalizes ADC memory
fn mem_init(page: *mut u32) {
    // I've left this commented out since ADC.d already does this for me.
    // In the future this will be executed and the pre-existing software 
    // for the ADC will be disabled (same with the button and possibly LED).
    // The DAC is more complex, so that's a problem for later.
    /*
    poke(page, 0x0008, 0x40000000);
    poke(page, 0x0004, 0x00000001);
    poke(page, 0x0028, 0x01000000);
    poke(page, 0x0014, 0x00010000);
    poke(page, 0x0034, 0x00000001);
    poke(page, 0x0024, 0x01000000);
    */
    poke(page, 0x0144, 0x00000980); // Sets the last 12 bits like this: 0b1001_1000_0000
}

pub fn init(fd: i32) -> IoResult<()> {
    if get().is_some() {
        return Ok(())
    }

    let mmaped = map(fd, ADC_PAGE as i64)?;
    
    ADC_POINTER.set(mmaped);

    mem_init(mmaped);

    Ok(())
}

/// Reads the ADC (also known as the *LR*ADC, or ***L***ow-***R***esolution **A**nalog to **D**igital **C**onverter)
pub fn read() -> u8 {
    if let Some(pointer) = get() {
        poke(pointer, ADC_SCHED_OFFSET, 0x1);

        {
            let has_high_bit = peek(pointer, ADC_VALUE_OFFSET) >= 0x80000000;
            // There isn't a delay here since in C 
            // (see https://github.com/Hixie/localbit/blob/master/localbit.c#L346) 
            // it works that way so I'll leave it like this in Rust too 
            while (peek(pointer, ADC_VALUE_OFFSET) >= 0x80000000) == has_high_bit { }
        }

        let mut value = peek(pointer, ADC_VALUE_OFFSET) & !0x80000000;
        poke(pointer, ADC_CLEAR_OFFSET, 0x1);
        value = if value <= 200 {
            0
        } else {
            (31 * value - 0x1838) / 11
        };

        // That comment before was a lie
        // the `as` keyword clamps automatically
        (value / 16) as u8
    } else {
        println!("warning: no ADC page pointer found");
        0
    }
}