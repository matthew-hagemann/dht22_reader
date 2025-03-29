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
const TIMEOUT: u128 = 100;

mod gpiod;

use gpiod::{cleanup, Gpiod, GpiodError, IGpiod, OFFSET};
use std::{ffi::CString, time::Instant};

fn main() {
    let path = CString::new(GPIO_CHIP_PATH).expect("CString::new failed");
    let path_ptr = path.as_ptr();

    let gpiod = Gpiod {};

    let chip = match gpiod.chip(path_ptr) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error creating chip: {}", e);
            return;
        }
    };

    let info = match gpiod.info(chip) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Error obtaining chip info: {}", e);
            cleanup(Some(chip), None, None, None);
            return;
        }
    };

    let name = match gpiod.name(info) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error obtaining chip name: {}", e);
            cleanup(Some(chip), Some(info), None, None);
            return;
        }
    };
    println!("{}", name);

    let settings = match gpiod.settings() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error creating new settings object: {}", e);
            cleanup(Some(chip), Some(info), None, None);
            return;
        }
    };

    // The DHT22 protocol initiate reading data by setting the pin to a pull up bias. Then, we pull
    // low for between 1~10ms. We then pull up for 20-40us (will let the bias take care of that)
    // and await a response from the sensor.
    match gpiod.settings_set_direction(settings, gpiod_line_direction_GPIOD_LINE_DIRECTION_OUTPUT) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error setting direction: {}", e);
            cleanup(Some(chip), Some(info), Some(settings), None);
            return;
        }
    }
    match gpiod.settings_set_bias(settings, gpiod_line_bias_GPIOD_LINE_BIAS_PULL_UP) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error setting bias: {}", e);
            cleanup(Some(chip), Some(info), Some(settings), None);
            return;
        }
    }

    let config = match gpiod.config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error creating new config object: {}", e);
            cleanup(Some(chip), Some(info), Some(settings), None);
            return;
        }
    };

    match gpiod.config_add_settings(config, settings) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error adding settings to config: {}", e);
            cleanup(Some(chip), Some(info), Some(settings), Some(config));
            return;
        }
    }

    // Wait 1ms before pulling low
    std::thread::sleep(std::time::Duration::from_millis(1));

    let request = match gpiod.chip_request_lines(chip, config) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error requesting line: {}", e);
            cleanup(Some(chip), Some(info), Some(settings), Some(config));
            return;
        }
    };

    // Pull high for 40us
    match gpiod.line_request_set_value(request, OFFSET, 1) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error setting line value: {}", e);
            cleanup(Some(chip), Some(info), None, None);
            return;
        }
    }
    std::thread::sleep(std::time::Duration::from_micros(40));

    // Now reconfigure the line to read and wait input from the DHT22 sensor.
    // New settings object for the same line
    let settings = match gpiod.settings() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error creating new settings object: {}", e);
            cleanup(Some(chip), Some(info), None, None);
            return;
        }
    };
    gpiod
        .settings_set_direction(settings, gpiod_line_direction_GPIOD_LINE_DIRECTION_INPUT)
        .unwrap();

    // Create config using the settings object
    let config = match gpiod.config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error creating new config object: {}", e);
            return;
        }
    };

    match gpiod.line_request_reconfigure_lines(request, config) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error reconfiguring line: {}", e);
            return;
        }
    }

    // Once we have the request object, we can clean the rest up.
    cleanup(Some(chip), Some(info), Some(settings), Some(config));

    // Now we expect the sensor to pull low for 80us, then high for 80us as an ack:
    let pulse = expect_pulse(false, request).unwrap();
    println!("Pulse low: {}us", pulse);
    let pulse = expect_pulse(true, request).unwrap();
    println!("Pulse high: {}us", pulse);
}

fn expect_pulse(value: bool, request: *mut gpiod::gpiod_line_request) -> Result<u128, GpiodError> {
    let start = Instant::now();

    while (Gpiod {}.line_request_get_value(request, OFFSET).unwrap() == value) {
        if start.elapsed().as_micros() > TIMEOUT {
            return Err(GpiodError::Timeout);
        }
    }

    Ok(start.elapsed().as_micros())
}
