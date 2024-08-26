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
use std::{io::Result as IoResult, sync::OnceLock};

pub const ADC_PAGE: usize = 0x80050000;
pub const ADC_SCHED_OFFSET: usize = 0x0004;
pub const ADC_VALUE_OFFSET: usize = 0x0050;
pub const ADC_CLEAR_OFFSET: usize = 0x0018;

static mut ADC_POINTER: OnceLock<*mut u32> = OnceLock::new();

fn get() -> Option<*mut u32> {
    unsafe { ADC_POINTER.get().copied() }
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
        return Ok(());
    }

    let mmaped = map(fd, ADC_PAGE as i64)?;

    unsafe { ADC_POINTER.set(mmaped).unwrap() }

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
            while (peek(pointer, ADC_VALUE_OFFSET) >= 0x80000000) == has_high_bit {}

            // wait for the LRADC0_IRQ bit to become 1 (happens after a conversion completes)
            // this might solve issue #7
            while (peek(pointer, 0x0010) & 0x1) == 0 {}
        }

        let mut value = peek(pointer, ADC_VALUE_OFFSET) & !0x80000000;
        poke(pointer, ADC_CLEAR_OFFSET, 0x1); // clears the LRADC0_IRQ bit in HW_LRADC_CTRL1
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

/// Gets the CPU die temperature, in Kelvin.
pub fn read_temp() -> f32 {
    if let Some(ptr) = get() {
        // Channel 1 is converted from channel 8 (PMOS THIN)
        // Channel 2 is converted from channel 9 (NMOS THIN)

        // Await conversion of channel 1
        let pmos_thin = {
            // Schedule conversion
            poke(ptr, ADC_SCHED_OFFSET, 0x2);
            let has_high_bit = peek(ptr, 0x0060) >= 0x80000000;
            while (peek(ptr, 0x0060) >= 0x80000000) == has_high_bit {}
            peek(ptr, 0x0060) & 0xFFF // mask to the low 12 bits
        } as f32;

        // Await conversion of channel 2
        let nmos_thin = {
            // Schedule conversion
            poke(ptr, ADC_SCHED_OFFSET, 0x4);
            let has_high_bit = peek(ptr, 0x0060) >= 0x80000000;
            while (peek(ptr, 0x0070) >= 0x80000000) == has_high_bit {}
            peek(ptr, 0x0070) & 0xFFF // mask to the low 12 bits
        } as f32;
        // (channel9 - channel8) * 1.012 / 4

        // Clear
        poke(ptr, ADC_CLEAR_OFFSET, 0x6);

        (nmos_thin - pmos_thin) * 1.012 / 4.0
    } else {
        println!("warning: no ADC page pointer found");
        f32::NAN
    }
}
