use std::ptr::{read_volatile, write_volatile};

pub const ADC_PAGE: usize = 0x80050000;
const ADC_SCHED_PAGE: *mut u32 = (ADC_PAGE + 0x0004) as *mut u32;
const ADC_VALUE_PAGE: *const u32 = (ADC_PAGE + 0x0050) as *const u32;
const ADC_CLEAR_PAGE: *mut u32 = (ADC_PAGE + 0x0018) as *mut u32;
static mut ADC_RAW_VALUE: u32 = 0;

/*
pub fn init() {
    if read_to_string("/var/adc_init").is_ok() { // ADC was already initalized previously in cloud_client
        return
    }
    write_file("/var/adc_init", []).unwrap();

    // There isn't actually anything to do AFAIK so uhhh yeah lol
}
*/

/// Gets the raw ADC value, does some math (defined by the original ADC.d program
/// from littleBits), then downscales its range from u32 to u8.
fn read() -> u8 {
    let mut value = unsafe {
        write_volatile(ADC_SCHED_PAGE, 0x1);

        let mut value = read_volatile(ADC_VALUE_PAGE);
        while (value & 0x80000000) == ADC_RAW_VALUE & 0x80000000 {
            value = read_volatile(ADC_VALUE_PAGE);
        }

        ADC_RAW_VALUE = value;

        write_volatile(ADC_CLEAR_PAGE, 0x1);

        value
    } % 0x40000;
    value = if value <= 200 {
        0
    } else {
        (31 * value - 0x1838) / 11
    };

    (value.clamp(0, 4095) / 16) as u8
}

fn main() {
    eprintln!("ADC Value: {}", read())
}