#![allow(improper_ctypes)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CString;

include!("bindings/bindings.rs");
pub trait IGpiod {
    /// Opens a GPIO chip.
    ///
    /// # Safety
    /// - The caller must ensure that `ptr` is a valid, null-terminated C string
    ///   representing a valid GPIO chip path.
    /// - The returned `gpiod_chip` pointer must be freed properly.
    unsafe fn chip(&self, ptr: *const i8) -> *mut gpiod_chip;

    /// Retrieves chip information.
    ///
    /// # Safety
    /// - `chip` must be a valid, non-null pointer to an open `gpiod_chip` instance.
    /// - The returned `gpiod_chip_info` pointer must be freed properly.
    unsafe fn info(&self, chip: *mut gpiod_chip) -> *mut gpiod_chip_info;

    /// Retrieves the name of a GPIO chip.
    ///
    /// # Safety
    /// - `info` must be a valid, non-null pointer to a `gpiod_chip_info` instance.
    unsafe fn name(&self, info: *mut gpiod_chip_info) -> *const i8;

    /// Creates a new GPIO line settings object.
    ///
    /// # Safety
    /// - The caller must ensure that the returned pointer is freed using `gpiod_line_settings_free()`.
    unsafe fn settings(&self) -> *mut gpiod_line_settings;

    /// Sets the drive bias for a GPIO line.
    ///
    /// # Safety
    /// - `settings` must be a valid, non-null pointer to a `gpiod_line_settings` instance.
    unsafe fn settings_set_drive(&self, settings: *mut gpiod_line_settings, bias: gpiod_line_bias);

    /// Sets the direction of a GPIO line.
    ///
    /// # Safety
    /// - `settings` must be a valid, non-null pointer to a `gpiod_line_settings` instance.
    unsafe fn settings_set_direction(
        &self,
        settings: *mut gpiod_line_settings,
        direction: gpiod_line_direction,
    );

    /// Creates a new GPIO line configuration object.
    ///
    /// # Safety
    /// - The caller must ensure that the returned pointer is freed using `gpiod_line_config_free()`.
    unsafe fn config(&self) -> *mut gpiod_line_config;

    /// Adds a line setting to a configuration object.
    ///
    /// # Safety
    /// - `config` must be a valid, non-null pointer to a `gpiod_line_config` instance.
    /// - `settings` must be a valid, non-null pointer to a `gpiod_line_settings` instance.
    unsafe fn config_add_settings(
        &self,
        config: *mut gpiod_line_config,
        settings: *mut gpiod_line_settings,
    ) -> ::std::os::raw::c_int;
}

pub struct Gpiod {}

impl IGpiod for Gpiod {
    unsafe fn chip(&self, ptr: *const i8) -> *mut gpiod_chip {
        unsafe { gpiod_chip_open(ptr) }
    }
    unsafe fn info(&self, chip: *mut gpiod_chip) -> *mut gpiod_chip_info {
        unsafe { gpiod_chip_get_info(chip) }
    }
    unsafe fn name(&self, info: *mut gpiod_chip_info) -> *const i8 {
        unsafe { gpiod_chip_info_get_name(info) }
    }
    unsafe fn settings(&self) -> *mut gpiod_line_settings {
        unsafe { gpiod_line_settings_new() }
    }
    unsafe fn settings_set_drive(&self, settings: *mut gpiod_line_settings, bias: gpiod_line_bias) {
        unsafe {
            gpiod_line_settings_set_drive(settings, bias);
        };
    }
    unsafe fn settings_set_direction(
        &self,
        settings: *mut gpiod_line_settings,
        direction: gpiod_line_direction,
    ) {
        unsafe {
            gpiod_line_settings_set_direction(settings, direction);
        }
    }
    unsafe fn config(&self) -> *mut gpiod_line_config {
        unsafe { gpiod_line_config_new() }
    }

    unsafe fn config_add_settings(
        &self,
        config: *mut gpiod_line_config,
        settings: *mut gpiod_line_settings,
    ) -> ::std::os::raw::c_int {
        unsafe { gpiod_line_config_add_line_settings(config, &OFFSET, 1, settings) }
    }
}

// FIXME: These should ultimately be configurable, but will hardcode them for my board setup for
// now.
// Chip: A chip with pins on it. RPI's just have the 1, which will be at index 0
const GPIO_CHIP_PATH: &str = "/dev/gpiochip0";
// The pin/line. Refered to as offsets in documentation as when you have multiple chips and want to
// refer to a specific pin, you refer to it by its offset from its chip index.
const OFFSET: std::os::raw::c_uint = 21;

fn main() {
    let path = CString::new(GPIO_CHIP_PATH).expect("CString::new failed");
    let path_ptr = path.as_ptr();

    let gpiod = Gpiod {};

    // SAFETY: Closed at the end of the program to ensure all resources are released
    // SAFTEY: path_ptr can't be NULL, we just set it based on a const
    let chip: *mut gpiod_chip = unsafe { gpiod.chip(path_ptr) };

    // Null check chip. Null is returned if an error occured.
    if chip.is_null() {
        eprintln!("Failed to open GPIO chip");
        return;
    }

    // SAFETY: Must be explicitly freed using gpiod_chip_info_free()
    let info: *mut gpiod_chip_info = unsafe { gpiod.info(chip) };
    if info.is_null() {
        eprintln!("Failed to get chip info");
        // SAFETY: questionable at best, there should be a smarter way of doing this...
        cleanup(Some(chip), None, None, None);
        return;
    }

    // SAFETY: Yet to be determined
    let name: *const i8 = unsafe { gpiod.name(info) };
    println!("{}", unsafe {
        std::ffi::CStr::from_ptr(name).to_string_lossy()
    });

    // Create a settings object that will be used to configure the line
    // SAFETY: settings must be freed using gpiod_line_settings_free()
    let settings = unsafe { gpiod.settings() };
    if settings.is_null() {
        eprintln!("Failed to create GPIO settings object");
        cleanup(Some(chip), Some(info), None, None);
        return;
    }

    // The DHT22 protocol initiate reading data by setting the pin to a pull up bias. Then, we pull
    // low for between 1~10ms. We then pull up for 20-40us (will let the bias take care of that)
    // and await a response from the sensor.
    unsafe {
        gpiod.settings_set_direction(settings, gpiod_line_direction_GPIOD_LINE_DIRECTION_INPUT)
    };
    unsafe { gpiod.settings_set_drive(settings, gpiod_line_bias_GPIOD_LINE_BIAS_PULL_UP) };

    // SAFETY: config must be explicitly freed when we are done with it.
    let config = unsafe { gpiod.config() };
    if config.is_null() {
        eprintln!("Failed to create GPIO config object");
        cleanup(Some(chip), Some(info), Some(settings), None);
        return;
    }

    let result = unsafe { gpiod.config_add_settings(config, settings) };

    if result != 0 {
        eprintln!("Failed to add line settings to config");
        return;
    }

    // Wait 1ms before pulling low
    std::thread::sleep(std::time::Duration::from_millis(1));

    cleanup(Some(chip), Some(info), Some(settings), Some(config));
}

fn cleanup(
    chip: Option<*mut gpiod_chip>,
    info: Option<*mut gpiod_chip_info>,
    settings: Option<*mut gpiod_line_settings>,
    config: Option<*mut gpiod_line_config>,
) {
    if let Some(cf) = config {
        // SAFETY: We explicitly checked config is not null when it was returned by
        // gpiod_line_config_new()
        unsafe { gpiod_line_config_free(cf) };
    }
    if let Some(s) = settings {
        // SAFETY: We explicitly checked settigns is not null when it was returned by
        // gpiod_line_settings_new()
        unsafe { gpiod_line_settings_free(s) };
    }
    if let Some(i) = info {
        // SAFETY: We explicitly checked chip is not null when it was returned by gpiod_chip_open()
        unsafe { gpiod_chip_info_free(i) };
    }
    if let Some(c) = chip {
        // SAFETY: We explicitly checked info is not null when it was returned by gpiod_chip_get_info()
        unsafe { gpiod_chip_close(c) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Counters to verify each free function is called.
    static CONFIG_FREED: AtomicUsize = AtomicUsize::new(0);
    static SETTINGS_FREED: AtomicUsize = AtomicUsize::new(0);
    static INFO_FREED: AtomicUsize = AtomicUsize::new(0);
    static CHIP_FREED: AtomicUsize = AtomicUsize::new(0);

    // Override external functions provided by bindgen.
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_line_config_free(_ptr: *mut gpiod_line_config) {
        CONFIG_FREED.fetch_add(1, Ordering::SeqCst);
    }
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_line_settings_free(_ptr: *mut gpiod_line_settings) {
        SETTINGS_FREED.fetch_add(1, Ordering::SeqCst);
    }
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_chip_info_free(_ptr: *mut gpiod_chip_info) {
        INFO_FREED.fetch_add(1, Ordering::SeqCst);
    }
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_chip_close(_ptr: *mut gpiod_chip) {
        CHIP_FREED.fetch_add(1, Ordering::SeqCst);
    }

    // TODO: create a trait that wraps the bindgen code and create a mock based on that trait for
    // these functions
    #[test]
    fn test_cleanup_invokes_all_free_functions() {
        // Reset counters.
        CONFIG_FREED.store(0, Ordering::SeqCst);
        SETTINGS_FREED.store(0, Ordering::SeqCst);
        INFO_FREED.store(0, Ordering::SeqCst);
        CHIP_FREED.store(0, Ordering::SeqCst);

        // Pass dummy non-null pointers.
        cleanup(
            Some(1 as *mut gpiod_chip),
            Some(1 as *mut gpiod_chip_info),
            Some(1 as *mut gpiod_line_settings),
            Some(1 as *mut gpiod_line_config),
        );

        assert_eq!(CONFIG_FREED.load(Ordering::SeqCst), 1);
        assert_eq!(SETTINGS_FREED.load(Ordering::SeqCst), 1);
        assert_eq!(INFO_FREED.load(Ordering::SeqCst), 1);
        assert_eq!(CHIP_FREED.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_cleanup_handles_none() {
        // Reset counters.
        CONFIG_FREED.store(0, Ordering::SeqCst);
        SETTINGS_FREED.store(0, Ordering::SeqCst);
        INFO_FREED.store(0, Ordering::SeqCst);
        CHIP_FREED.store(0, Ordering::SeqCst);

        // Call cleanup with None for all pointers.
        cleanup(None, None, None, None);

        assert_eq!(CONFIG_FREED.load(Ordering::SeqCst), 0);
        assert_eq!(SETTINGS_FREED.load(Ordering::SeqCst), 0);
        assert_eq!(INFO_FREED.load(Ordering::SeqCst), 0);
        assert_eq!(CHIP_FREED.load(Ordering::SeqCst), 0);
    }
}
