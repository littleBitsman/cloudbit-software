//! ADC wrapper

use super::MAP_SIZE;
use std::{
    io::{Error, Result as IoResult},
    ptr::null_mut,
    sync::OnceLock
};

use libc::{mmap, munmap, open, MAP_FAILED, MAP_SHARED, O_RDWR, PROT_READ, PROT_WRITE};
use tokio::time::{Duration, sleep};

pub const ADC_PAGE: usize = 0x80050000;
pub const ADC_SCHED_OFFSET: usize = 0x0004;
pub const ADC_VALUE_OFFSET: usize = 0x0050;
pub const ADC_CLEAR_OFFSET: usize = 0x0018;

const ADC_POINTER: OnceLock<*mut u32> = OnceLock::new();

pub fn init() -> IoResult<()> {
    if ADC_POINTER.get().is_some() {
        return Ok(());
    }

    let fd = unsafe { open("/dev/mem".as_ptr(), O_RDWR) };

    let mmaped = unsafe {
        mmap(
            null_mut(),
            MAP_SIZE,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            fd,
            ADC_PAGE as _,
        )
    };

    if mmaped == MAP_FAILED {
        return Err(Error::last_os_error());
    }
    ADC_POINTER.into_inner();
    ADC_POINTER.set(mmaped as *mut u32).unwrap();

    Ok(())
}

// I didn't want to put async here but oh well I have to add a delay somehow
pub async fn read() -> u8 {
    if let Some(pointer) = ADC_POINTER.get() {
        let pointer = *pointer;
        let mut value = unsafe {
            pointer
                .byte_add(ADC_SCHED_OFFSET)
                .write_volatile(0x1);

            let value_ptr = pointer.byte_add(ADC_VALUE_OFFSET);

            let curr_high_bit = value_ptr.read_volatile() & 0x80000000;
            while (value_ptr.read_volatile() & 0x80000000) == curr_high_bit {
                sleep(Duration::from_millis(1)).await;
            }

            let value = value_ptr.read_volatile();

            pointer
                .byte_add(ADC_CLEAR_OFFSET)
                .write_volatile(0x1);

            value
        }.clamp(0, 0x3FFFF);

        value = if value <= 200 { 0 } else {
            (31 * value - 0x1838) / 11
        };

        (value.clamp(0, 4095) / 16) as u8
    } else {
        0
    }
}

pub fn cleanup() {
    if let Some(pointer) = ADC_POINTER.into_inner() {
        unsafe { munmap(pointer as *mut _, MAP_SIZE) };
    }
}
