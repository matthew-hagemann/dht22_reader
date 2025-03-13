#![allow(improper_ctypes)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CString;

include!("bindings/bindings.rs");

// FIXME: These should ultimately be configurable, but will hardcode them for my board setup for
// now.
// Chip: A chip with pins on it. RPI's just have the 1, which will be at index 0
const GPIO_CHIP_PATH: &str = "/dev/gpiochip0";
// The pin/line. Refered to as offsets in documentation as when you have multiple chips and want to
// refer to a specific pin, you refer to it by its offset from its chip index.
const OFFSET: i64 = 21;

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
    let info: *mut gpiod_chip_info = unsafe { gpiod_chip_get_info(chip) };
    if info.is_null() {
        eprintln!("Failed to get chip info");
        // SAFETY: questionable at best, there should be a smarter way of doing this...
        unsafe { gpiod_chip_close(chip) };
        return;
    }

    // SAFETY: Yet to be determined
    let name: *const i8 = unsafe { gpiod_chip_info_get_name(info) };
    println!("{}", unsafe {
        std::ffi::CStr::from_ptr(name).to_string_lossy()
    });

    // Create a settings object that will be used to configure the line
    // SAFETY: settings must be freed using gpiod_line_settings_free()
    let settings = unsafe { gpiod_line_settings_new() };

    // The DHT22 protocol initiate reading data by setting the pin to a pull up bias. Then, we pull
    // low for between 1~10ms. We then pull up for 20-40us (will let the bias take care of that)
    // and await a response from the sensor.
    //
    // SAFETY: settings must be freed using gpiod_line_settings_free()
    unsafe {
        gpiod_line_settings_set_direction(
            settings,
            gpiod_line_direction_GPIOD_LINE_DIRECTION_INPUT,
        );
        gpiod_line_settings_set_drive(settings, gpiod_line_bias_GPIOD_LINE_BIAS_PULL_UP);
    };

    // SAFETY: config must be explicitly freed when we are done with it.
    let config = unsafe { gpiod_line_config_new() };

    // SAFETY: We explicitly checked info is not null when it was returned by gpiod_chip_get_info()
    unsafe { gpiod_chip_info_free(info) };
    // SAFETY: We explicitly checked chip is not null when it was returned by gpiod_chip_open()
    unsafe { gpiod_chip_close(chip) };
}
