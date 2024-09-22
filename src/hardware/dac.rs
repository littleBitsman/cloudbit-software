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

//! DAC wrapper

use crate::hardware::mem::{map, peek, poke};
use std::{
    io::Result as IoResult,
    sync::{
        atomic::{AtomicU32, Ordering::SeqCst},
        OnceLock,
    },
};

pub const DAC_PAGE: usize = 0x80048000;
pub const DAC_STATE_OFFSET: usize = 0x40;
pub const DAC_VALUE_OFFSET: usize = 0xF0;

static LAST_DAC_READY_FLAG: AtomicU32 = AtomicU32::new(0);
static mut DAC_POINTER: OnceLock<*mut u32> = OnceLock::new();

fn get() -> Option<*mut u32> {
    unsafe { DAC_POINTER.get().copied() }
}

/// Initalizes DAC memory
fn mem_init(page: *mut u32) {
    let _ = page;
    // This will be commented out for now
    /*
    // This sequence based on DAC_init
    poke(page, 0x08, -0x40000000_i32 as u32); // SFTRST and CLKGATE = 0
    poke(page, 0x78, 0x1001); // HW_AUDIOOUT_PWRDN HEADPHONE and DAC = 0
    poke(page, 0x28, 0x7000000); // HW_AUDIOOUT_DACSRR SRC_HOLD = 0
    poke(page, 0x24, 0x13FF); // HW_AUDIOOUT_DACSRR SRC_FRAC = 0x13FF
    poke(page, 0x38, 0x1000000); // HW_AUDIOOUT_DACVOLUME MUTE_LEFT = 0
    poke(page, 0x58, 0x1007F7F); // HW_AUDIOOUT_HPVOL VOL_LEFT, VOL_RIGHT, MUTE = 0
    poke(page, 0x54, 0x087F); // HW_AUDIOOUT_HPVOL VOL_RIGHT = 0x7F, VOL_LEFT = 0x8
    poke(page, 0x84, 0x1074); // HW_AUDIOOUT_REFCTRL DAC_ADJ = 0x4, VAG_VAL = 0x7, ADJ_VAG = 0x1
    poke(page, 0x94, 0x20); // HW_AUDIOOUT_ANACTRL HP_HOLD_GND = 1
    poke(page, 0xE8, -0x80000000_i32 as u32); // HW_AUDIOOUT_ANACLKCTRL CLKGATE = 0
    poke(page, 0x04, 0x1); // HW_AUDIOOUT_CTRL RUN = 1
    */
}

pub fn init(fd: i32) -> IoResult<()> {
    if get().is_some() {
        return Ok(());
    }

    let mmaped = map(fd, DAC_PAGE as i64)?;
    mem_init(mmaped);
    unsafe { DAC_POINTER.set(mmaped).unwrap() }

    set_ready_flag(peek(mmaped, DAC_STATE_OFFSET) ^ 2);

    Ok(())
}

fn get_ready_flag() -> u32 {
    LAST_DAC_READY_FLAG.load(SeqCst)
}
fn set_ready_flag(v: u32) {
    LAST_DAC_READY_FLAG.store(v, SeqCst)
}

/// Set output
pub fn set(value: u16) {
    if let Some(ptr) = get() {
        let converted = (value ^ 0x8000) as u32;
        let packed = (converted << 16) | converted;

        let mut curr_state;
        let mut state = get_ready_flag();

        for _ in 0..20 {
            curr_state = peek(ptr, DAC_STATE_OFFSET);
            if ((curr_state ^ state) & 2) != 0 {
                poke(ptr, DAC_VALUE_OFFSET, packed);
                state = curr_state;
            }
        }

        set_ready_flag(state)
    } else {
        println!("warning: no DAC page pointer found")
    }
}
