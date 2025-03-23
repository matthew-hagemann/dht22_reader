#![allow(improper_ctypes)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
// I'm checking for null ptr derefs already
#![allow(clippy::not_unsafe_ptr_arg_deref)]

include!("bindings/bindings.rs");

// FIXME: These should ultimately be configurable, but will hardcode them for my board setup for
// now.
// Chip: A chip with pins on it. RPI's just have the 1, which will be at index 0
const GPIO_CHIP_PATH: &str = "/dev/gpiochip0";

mod gpiod;

use gpiod::{cleanup, Gpiod, IGpiod, OFFSET};
use std::ffi::CString;

fn main() {
    let path = CString::new(GPIO_CHIP_PATH).expect("CString::new failed");
    let path_ptr = path.as_ptr();

    let gpiod = Gpiod {};

    let chip = match gpiod.chip(path_ptr) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let info = match gpiod.info(chip) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("{}", e);
            cleanup(Some(chip), None, None, None);
            return;
        }
    };

    let name = match gpiod.name(info) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{}", e);
            cleanup(Some(chip), Some(info), None, None);
            return;
        }
    };
    println!("{}", name);

    let settings = match gpiod.settings() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            cleanup(Some(chip), Some(info), None, None);
            return;
        }
    };

    // The DHT22 protocol initiate reading data by setting the pin to a pull up bias. Then, we pull
    // low for between 1~10ms. We then pull up for 20-40us (will let the bias take care of that)
    // and await a response from the sensor.
    gpiod
        .settings_set_direction(settings, gpiod_line_direction_GPIOD_LINE_DIRECTION_OUTPUT)
        .unwrap();
    gpiod
        .settings_set_drive(settings, gpiod_line_bias_GPIOD_LINE_BIAS_PULL_UP)
        .unwrap();

    let config = match gpiod.config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            cleanup(Some(chip), Some(info), Some(settings), None);
            return;
        }
    };

    gpiod.config_add_settings(config, settings).unwrap();

    // Wait 1ms before pulling low
    std::thread::sleep(std::time::Duration::from_millis(1));

    let request = gpiod.chip_request_lines(chip, config).unwrap();

    // Pull high for 40us
    gpiod.line_request_set_value(request, OFFSET, 1).unwrap();
    std::thread::sleep(std::time::Duration::from_micros(40));

    cleanup(Some(chip), Some(info), Some(settings), Some(config));
}
