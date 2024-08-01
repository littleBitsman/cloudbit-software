//! LED wrapper
//!
//! Note that this does NOT use memory mapping and volatile reads/writes,
//! rather it uses the littleBits-provided commands
//! (specifically `/usr/local/lb/LEDcolor/bin/setColor`) mainly due to the
//! fact that LED commands are not used often (unless there is a bad config).
//! 
//! (also blink would suck to make xd)

use crate::LEDCommand;
use execute::Execute;
use std::process::Command;

/// the raw form of `set_led()`, directly passes `str` to `/usr/local/lb/LEDcolor/bin/setColor` and
/// returns success as a boolean
fn set_raw(str: String) -> bool {
    Command::new("/usr/local/lb/LEDcolor/bin/setColor")
        .arg(str)
        .execute_check_exit_status_code(0)
        .is_ok()
}

/// set led using [`LEDCommand`]
///
/// returns success as a boolean
pub fn set(arg: LEDCommand) -> bool {
    set_raw(arg.to_string())
}

/// set led using [`LEDCommandChain`]
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
