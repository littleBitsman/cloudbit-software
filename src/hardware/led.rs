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

use crate::LEDCommand;
use std::process::Command;

/// the raw form of `set_led()`, directly passes `str` to `/usr/local/lb/LEDcolor/bin/setColor` and
/// returns success as a boolean
fn set_raw(str: String) -> bool {
    Command::new("/usr/local/lb/LEDcolor/bin/setColor")
        .arg(str)
        .status()
        .expect("failed to execute /usr/local/lb/LEDcolor/bin/setColor")
        .success()
}

/// set led using [`LEDCommand`]
///
/// returns success as a boolean
pub fn set(arg: LEDCommand) -> bool {
    set_raw(arg.to_string())
}

/// set led using a [`Vec<LEDCommand>`]
///
/// returns success as a boolean
pub fn set_many(arg: Vec<LEDCommand>) -> bool {
    if arg.is_empty() {
        false
    } else {
        let mut str = String::new();
        for item in arg {
            str.push_str(format!("{item} ").as_str())
        }
        set_raw(str)
    }
}
