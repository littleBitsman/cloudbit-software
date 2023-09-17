extern crate libc;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::ptr;
use std::mem;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use std::io::{Error, ErrorKind};
use std::fs::File;
use std::io::Read;

// SYSTEM

fn print_log(message: &str) {
    let current_time = SystemTime::now();
    let timestamp = current_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
    println!("{}.{} cloudbit: {}", timestamp, current_time.subsec_nanos() / 1_000_000, message);
}

fn print_error(message: &str) {
    print_log(&format!("{}: {}", message, Error::last_os_error()));
}

fn fill_time_spec(ts: &mut libc::timespec, seconds: f64) {
    let integral = seconds.floor();
    let fraction = seconds - integral;
    ts.tv_sec = integral as i64;
    ts.tv_nsec = (fraction * 1_000_000_000.0) as i64;
}

fn delay(seconds: f64) {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    fill_time_spec(&mut ts, seconds);
    unsafe {
        libc::nanosleep(&ts, ptr::null_mut());
    }
}

fn socket_has_message(socket: c_uint, seconds: f64) -> bool {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    fill_time_spec(&mut ts, seconds);
    let mut descriptors: libc::fd_set = mem::zeroed();
    libc::FD_ZERO(&mut descriptors);
    libc::FD_SET(socket as i32, &mut descriptors);
    let result = unsafe {
        libc::pselect(socket as i32 + 1, &mut descriptors, ptr::null_mut(), ptr::null_mut(), &ts, ptr::null_mut())
    };
    result > 0 && libc::FD_ISSET(socket as i32, &mut descriptors)
}

fn read_file(filename: &str, buffer: &mut Vec<u8>, size: &mut c_uint, null_terminate: bool) -> bool {
    let c_filename = CString::new(filename).unwrap();
    let c_mode = CString::new("rb").unwrap();
    let file = unsafe { libc::fopen(c_filename.as_ptr(), c_mode.as_ptr()) };
    if file.is_null() {
        print_error("Failed to read file during fopen");
        *size = 0;
        *buffer = Vec::new();
        return false;
    }
    if unsafe { libc::fseek(file, 0, libc::SEEK_END) } != 0 {
        print_error("Failed to read file during fseek to end");
        unsafe { libc::fclose(file) };
        *size = 0;
        *buffer = Vec::new();
        return false;
    }
    let length = unsafe { libc::ftell(file) };
    if unsafe { libc::fseek(file, 0, libc::SEEK_SET) } != 0 {
        print_error("Failed to read file during fseek to start");
        unsafe { libc::fclose(file) };
        *size = 0;
        *buffer = Vec::new();
        return false;
    }
    if *size == 0 {
        *size = length as c_uint;
    } else if *size > length as c_uint {
        print_log(&format!(
            "Failed to read file: insufficient data in file (file is {} bytes, need {} bytes)",
            length,
            *size
        ));
        unsafe { libc::fclose(file) };
        *size = 0;
        *buffer = Vec::new();
        return false;
    }
    let buffer_size = *size as usize + if null_terminate { 1 } else { 0 };
    *buffer = vec![0u8; buffer_size];
    let length = unsafe { libc::fread(buffer.as_mut_ptr() as *mut libc::c_void, 1, *size as usize, file) };
    if length != *size as usize {
        print_error("Failed to read file during fread");
        *buffer = Vec::new();
        *size = 0;
        unsafe { libc::fclose(file) };
        return false;
    }
    if unsafe { libc::fclose(file) } != 0 {
        print_error("Failed to read file during fclose");
        *buffer = Vec::new();
        *size = 0;
        return false;
    }
    if null_terminate {
        buffer[length] = 0;
    }
    true
}

fn decode_hex_bytes(text_buffer: &str, byte_buffer: &mut Vec<u8>, length: c_uint) -> bool {
    let text_bytes = text_buffer.as_bytes();
    let mut index = 0;
    while index < length {
        let mut byte_value: u8 = 0;
        let mut high_byte = false;
        while !high_byte {
            let digit = text_bytes[index as usize * 2 + if high_byte { 0 } else { 1 }];
            let value = match digit {
                b'0' => 0,
                b'1' => 1,
                b'2' => 2,
                b'3' => 3,
                b'4' => 4,
                b'5' => 5,
                b'6' => 6,
                b'7' => 7,
                b'8' => 8,
                b'9' => 9,
                b'A' | b'a' => 10,
                b'B' | b'b' => 11,
                b'C' | b'c' => 12,
                b'D' | b'd' => 13,
                b'E' | b'e' => 14,
                b'F' | b'f' => 15,
                _ => return false,
            };
            if high_byte {
                byte_value <<= 4;
            }
            byte_value += value;
            high_byte = !high_byte;
        }
        byte_buffer.push(byte_value);
        index += 1;
    }
    true
}

// NETWORK

const MAC_ADDRESS_SIZE: usize = 6;

struct Network {
    server_address: libc::sockaddr_in6,
    local_address: libc::sockaddr_in6,
    send_socket: c_int,
    receive_socket: c_int,
    local_mac_address: [u8; MAC_ADDRESS_SIZE],
}

impl Network {
    fn new(server_name: &str, send_port: u16, receive_port: u16, local_mac_address: [u8; MAC_ADDRESS_SIZE]) -> Network {
        let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
        hints.ai_flags = libc::AI_ADDRCONFIG | libc::AI_V4MAPPED;
        hints.ai_socktype = libc::SOCK_DGRAM;
        let c_server_name = CString::new(server_name).unwrap();
        let mut server_info: *mut libc::addrinfo = ptr::null_mut();
        let c_send_port = CString::new(format!("{}", send_port)).unwrap();
        let c_receive_port = CString::new(format!("{}", receive_port)).unwrap();
        let server_result = unsafe { libc::getaddrinfo(c_server_name.as_ptr(), c_send_port.as_ptr(), &hints, &mut server_info) };
        if server_result != 0 {
            print_error("Failed to get server address information");
            panic!("getaddrinfo failed with error: {}", server_result);
        }
        let mut local_info: *mut libc::addrinfo = ptr::null_mut();
        let local_result = unsafe { libc::getaddrinfo(ptr::null(), c_receive_port.as_ptr(), &hints, &mut local_info) };
        if local_result != 0 {
            print_error("Failed to get local address information");
            panic!("getaddrinfo failed with error: {}", local_result);
        }
        let server_addr = unsafe { (*server_info).ai_addr as *mut libc::sockaddr_in6 };
        let local_addr = unsafe { (*local_info).ai_addr as *mut libc::sockaddr_in6 };
        Network {
            server_address: *server_addr,
            local_address: *local_addr,
            send_socket: -1,
            receive_socket: -1,
            local_mac_address,
        }
    }

    fn send(&self, value: u16, button: bool) -> bool {
        let mut buffer: Vec<u8> = Vec::new();
        let button_byte = if button { 1 } else { 0 };
        let high_byte = ((value >> 8) & 0xFF) as u8;
        let low_byte = (value & 0xFF) as u8;
        buffer.extend_from_slice(&self.local_mac_address);
        buffer.push(button_byte);
        buffer.push(0);
        buffer.push(high_byte);
        buffer.push(low_byte);
        let result = unsafe {
            libc::sendto(
                self.send_socket,
                buffer.as_ptr() as *const libc::c_void,
                buffer.len(),
                0,
                &self.server_address as *const libc::sockaddr_in6 as *const libc::sockaddr,
                mem::size_of::<libc::sockaddr_in6>() as libc::socklen_t,
            )
        };
        if result == -1 {
            print_error("Failed to send data");
            return false;
        }
        true
    }

    fn receive(&self) -> (bool, Color, bool, u16) {
        let mut buffer: [u8; 10] = [0; 10];
        let result = unsafe {
            libc::recv(
                self.receive_socket,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
                0,
            )
        };
        if result == -1 {
            print_error("Failed to receive data");
            return (false, Color::Black, false, 0);
        }
        if result != buffer.len() as isize {
            print_log("Received incomplete data");
            return (false, Color::Black, false, 0);
        }
        if &buffer[0..6] != &self.local_mac_address {
            print_log("Received packet intended for another cloudbit");
            return (false, Color::Black, false, 0);
        }
        let set_led = (buffer[6] & 0x80) > 0;
        let color = if set_led { Color::from(buffer[6] & 0x07) } else { Color::Black };
        let set_output = (buffer[7] & 0x80) > 0;
        let value = if set_output { ((buffer[8] as u16) << 8) | buffer[9] as u16 } else { 0xFFFF };
        (set_led, color, set_output, value)
    }
}

enum Color {
    Black,
    Blue,
    Red,
    Purple,
    Green,
    Teal,
    Yellow,
    White,
}

impl Color {
    fn from(value: u8) -> Color {
        match value {
            0 => Color::Black,
            1 => Color::Blue,
            2 => Color::Red,
            3 => Color::Purple,
            4 => Color::Green,
            5 => Color::Teal,
            6 => Color::Yellow,
            7 => Color::White,
            _ => Color::Black,
        }
    }
}


fn main() {
    println!("hi")
}