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

//! LED wrapper
//!
//! Note that this does NOT use memory mapping and volatile reads/writes,
//! rather it uses the littleBits-provided commands
//! (specifically `/usr/local/lb/LEDcolor/bin/setColor`) mainly due to the
//! fact that LED commands are not used often (unless there is a bad config).
//!
//! (also blink would suck to make xd)

use crate::{
    hardware::mem::{map, poke},
    LEDCommand,
};
use std::{
    io::Result as IoResult,
    sync::{
        mpsc::{channel, Sender, TryRecvError},
        OnceLock,
    },
    thread::{sleep, spawn},
    time::Duration,
};

const GPIO_PAGE: usize = 0x80018000;
const SLEEP_DUR: Duration = Duration::from_millis(500);

static LED_CMD_SENDER: OnceLock<Sender<LEDCommand>> = OnceLock::new();
static mut GPIO_POINTER: OnceLock<*mut u32> = OnceLock::new();

fn get() -> Option<*mut u32> {
    unsafe { GPIO_POINTER.get().copied() }
}

fn mem_init(page: *mut u32) {
    poke(page, 0x0114, 0xF0000000); // HW_PINCTRL_MUXSEL1 BANK0_PIN31 and BANK0_PIN30 = 0b11
    poke(page, 0x0134, 0x03000000); // HW_PINCTRL_MUXSEL3 BANK1_PIN28 = 0b11
    poke(page, 0x0704, 0x40000000); // HW_PINCTRL_DOE0 bit 30 = 1
    poke(page, 0x0714, 0x10000000); // HW_PINCTRL_DOE1 bit 28 = 1
}

pub fn init(fd: i32) -> IoResult<()> {
    if LED_CMD_SENDER.get().is_some() || get().is_some() {
        return Ok(());
    }

    let mmaped = map(fd, GPIO_PAGE as i64)?;
    mem_init(mmaped);
    unsafe { GPIO_POINTER.set(mmaped).unwrap() }

    let (send, recv) = channel();
    LED_CMD_SENDER.set(send).unwrap();
    spawn(move || {
        let ptr = get().unwrap();

        let mut color = LEDCommand::White;
        let mut state = LEDCommand::Off;
        let mut is_on = false;
        let mut color_changed = false;
        loop {
            match recv.try_recv() {
                Ok(msg) => match msg {
                    LEDCommand::Off | LEDCommand::Blink | LEDCommand::Hold => state = msg,
                    _ => {
                        color_changed = color != msg;
                        color = msg;
                    }
                },
                Err(ty) => {
                    if ty == TryRecvError::Disconnected {
                        break;
                    }
                }
            }
            is_on = match state {
                LEDCommand::Off => false,
                LEDCommand::Hold => true,
                LEDCommand::Blink => !is_on,
                _ => unreachable!(), // state will only be Off, Hold, or Blink
            };

            if is_on {
                // Handle the current color
                if color_changed {
                    // ONLY if the color changed, write to memory again
                    let bitmask: u8 = match color {
                        LEDCommand::Red => 0b100,
                        LEDCommand::Green => 0b010,
                        LEDCommand::Blue => 0b001,
                        LEDCommand::Purple | LEDCommand::Violet => 0b101,
                        LEDCommand::Teal => 0b011,
                        LEDCommand::Yellow => 0b110,
                        LEDCommand::White => 0b111,
                        LEDCommand::Clownbarf => 0b111,
                        _ => unreachable!(),
                    };

                    poke(
                        ptr,
                        if (bitmask & 0b100) > 0 {
                            0x0508
                        } else {
                            0x0504
                        },
                        0x80000000,
                    );
                    poke(
                        ptr,
                        if (bitmask & 0b010) > 0 {
                            0x0508
                        } else {
                            0x0504
                        },
                        0x40000000,
                    );
                    poke(
                        ptr,
                        if (bitmask & 0b001) > 0 {
                            0x0518
                        } else {
                            0x0514
                        },
                        0x10000000,
                    );
                }
            }

            sleep(SLEEP_DUR)
        }
    });
    Ok(())
}

/// set led using [`LEDCommand`]
///
/// returns success as a boolean
pub fn set(arg: LEDCommand) -> bool {
    if let Some(sender) = LED_CMD_SENDER.get() {
        sender.send(arg).is_ok()
    } else {
        false
    }
}

/// set led using a [`Vec<LEDCommand>`]
///
/// returns success as a boolean
pub fn set_many(arg: Vec<LEDCommand>) -> bool {
    if arg.is_empty() {
        false
    } else {
        let mut combined = true;
        for item in arg {
            combined &= set(item);
        }
        combined
    }
}
