#![allow(improper_ctypes)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CString;

include!("bindings/bindings.rs");

const GPIO_CHIP_PATH: &str = "/dev/gpiochip0";

fn main() {
    let path = CString::new(GPIO_CHIP_PATH).expect("CString::new failed");
    let path_ptr = path.as_ptr();

    // SAFETY: Closed at the end of the program to ensure all resources are released
    let chip: *mut gpiod_chip = unsafe { gpiod_chip_open(path_ptr) };

    // Null check chip. Null is returned if an error occured.
    if chip.is_null() {
        eprintln!("Failed to open GPIO chip");
        return;
    }

    // SAFETY: Must be explicitly freed using gpiod_chip_info_free()
    let info: *mut gpiod_chip_info = unsafe { gpiod_chip_get_info(chip)};
    if info.is_null() {
        eprintln!("Failed to get chip info");
        // SAFETY: questionable at best, there should be a smarter way of doing this...
        unsafe { gpiod_chip_close(chip) };
        return;
    }

    // SAFETY: Yet to be determined
    let name: *const i8 = unsafe { gpiod_chip_info_get_name(info) };
    println!("{}", unsafe { std::ffi::CStr::from_ptr(name).to_string_lossy() });

    // SAFETY: We explicitly checked info is not null when it was returned by gpiod_chip_get_info()
    unsafe { gpiod_chip_info_free(info) };
    // SAFETY: We explicitly checked chip is not null when it was returned by gpiod_chip_open()
    unsafe { gpiod_chip_close(chip) };
}
