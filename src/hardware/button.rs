//! Button wrapper

use super::MAP_SIZE;
use std::{
    fs::OpenOptions,
    io::{Error, ErrorKind, Result as IoResult},
    os::unix::{fs::OpenOptionsExt, io::AsRawFd},
    ptr::null_mut,
    sync::OnceLock
};

use libc::{mmap64, munmap, MAP_FAILED, MAP_SHARED, O_RDWR, PROT_READ, PROT_WRITE};

const GPIO_PAGE: usize = 0x80018000;
const BUTTON_OFFSET: usize = 0x0610;

const GPIO_POINTER: OnceLock<*mut u32> = OnceLock::new();

pub fn init() -> IoResult<()> {
    if GPIO_POINTER.get().is_some() {
        return Ok(());
    }

    let fd = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(O_RDWR)
        .open("/dev/mem")?
        .as_raw_fd();

    let mmaped = unsafe {
        mmap64(
            null_mut(),
            MAP_SIZE,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            fd,
            GPIO_PAGE as _,
        )
    };

    if mmaped == MAP_FAILED {
        return Err(Error::new(ErrorKind::Other, "mmap failed"));
    }
    GPIO_POINTER.into_inner();
    GPIO_POINTER.set(mmaped as *mut u32).unwrap();

    Ok(())
}

pub fn get_state() -> bool {
    GPIO_POINTER.get().is_some_and(|v| {
        (unsafe { (*v).byte_add(BUTTON_OFFSET).read_volatile() } & 0x80) > 0
    })
}

pub fn cleanup() {
    if let Some(pointer) = GPIO_POINTER.into_inner() {
        unsafe { munmap(pointer as *mut _, MAP_SIZE) };
    }
}