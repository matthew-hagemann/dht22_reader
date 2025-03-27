#![allow(improper_ctypes)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
// I'm checking for null ptr derefs already
#![allow(clippy::not_unsafe_ptr_arg_deref)]

include!("bindings/bindings.rs");

use std::ptr;

use thiserror::Error;

// The pin/line. Refered to as offsets in documentation as when you have multiple chips and want to
// refer to a specific pin, you refer to it by its offset from its chip index.
pub const OFFSET: std::os::raw::c_uint = 21;

#[derive(Error, Debug)]
pub enum GpiodError {
    #[error("Failed to open GPIO chip")]
    OpenChip,
    #[error("Failed to get chip info")]
    GetChipInfo,
    #[error("Failed to get chip name")]
    GetChipName,
    #[error("Failed to create GPIO settings object")]
    CreateSettings,
    #[error("Failed to set bias on settings object with bias {0}")]
    SetBias(gpiod_line_bias),
    #[error("Failed to set direction on settings object with direction {0}")]
    SetDirection(gpiod_line_direction),
    #[error("Failed to create GPIO config object")]
    CreateConfig,
    #[error("Encountered an unexpected null pointer")]
    NullPtr,
    #[error("Failed to create line request")]
    LineRequest,
    #[error("Failed to set line request value")]
    LineRequestSetValue,
    #[error("Failed to get line request value")]
    LineRequestGetValue,
    #[error("Timeout waiting for line request value")]
    Timeout,
}

// FIXME: this is the wrong abstraction, as ideally we want to make use of implementing drop on
// structs in order for Rust to handle memory for us. IE, we should end up heading for:
//
// struct Chip {
//     ptr: *mut gpiod_chip,
// }
// impl Drop for Chip {
//     fn drop(&mut self) {
//         if !self.ptr.is_null() {
//             unsafe { gpiod_chip_close(self.ptr) }
//         }
//     }
// }
pub trait IGpiod {
    fn chip(&self, ptr: *const i8) -> Result<*mut gpiod_chip, GpiodError>;

    fn info(&self, chip: *mut gpiod_chip) -> Result<*mut gpiod_chip_info, GpiodError>;

    fn name(&self, info: *mut gpiod_chip_info) -> Result<String, GpiodError>;

    fn settings(&self) -> Result<*mut gpiod_line_settings, GpiodError>;

    fn settings_set_drive(
        &self,
        settings: *mut gpiod_line_settings,
        bias: gpiod_line_bias,
    ) -> Result<(), GpiodError>;

    fn settings_set_direction(
        &self,
        settings: *mut gpiod_line_settings,
        direction: gpiod_line_direction,
    ) -> Result<(), GpiodError>;

    fn config(&self) -> Result<*mut gpiod_line_config, GpiodError>;

    fn config_add_settings(
        &self,
        config: *mut gpiod_line_config,
        settings: *mut gpiod_line_settings,
    ) -> Result<::std::os::raw::c_int, GpiodError>;

    fn chip_request_lines(
        &self,
        chip: *mut gpiod_chip,
        line_cfg: *mut gpiod_line_config,
    ) -> Result<*mut gpiod_line_request, GpiodError>;

    fn line_request_set_value(
        &self,
        request: *mut gpiod_line_request,
        offset: ::std::os::raw::c_uint,
        value: gpiod_line_value,
    ) -> Result<(), GpiodError>;

    fn line_request_reconfigure_lines(
        &self,
        request: *mut gpiod_line_request,
        config: *mut gpiod_line_config,
    ) -> Result<(), GpiodError>;

    fn line_request_get_value(
        &self,
        request: *mut gpiod_line_request,
        offset: ::std::os::raw::c_uint,
    ) -> Result<bool, GpiodError>;
}

/// Concrete implementation of the GPIO device.
pub struct Gpiod {}

impl IGpiod for Gpiod {
    /// Opens a GPIO chip.
    ///
    /// # Safety
    /// - The returned `gpiod_chip` pointer must be freed properly.
    fn chip(&self, ptr: *const i8) -> Result<*mut gpiod_chip, GpiodError> {
        let result = unsafe { gpiod_chip_open(ptr) };
        if result.is_null() {
            return Err(GpiodError::OpenChip);
        }
        Ok(result)
    }
    /// Retrieves chip information.
    ///
    /// # Safety
    /// - `chip` must be a valid, non-null pointer to an open `gpiod_chip` instance.
    /// - The returned `gpiod_chip_info` pointer must be freed properly.
    fn info(&self, chip: *mut gpiod_chip) -> Result<*mut gpiod_chip_info, GpiodError> {
        if chip.is_null() {
            return Err(GpiodError::NullPtr);
        }
        let result = unsafe { gpiod_chip_get_info(chip) };
        if result.is_null() {
            return Err(GpiodError::GetChipInfo);
        }
        Ok(result)
    }
    /// Retrieves the name of a GPIO chip.
    ///
    /// # Safety
    /// - `info` must be a valid, non-null pointer to a `gpiod_chip_info` instance.
    fn name(&self, info: *mut gpiod_chip_info) -> Result<String, GpiodError> {
        if info.is_null() {
            return Err(GpiodError::NullPtr);
        }
        let result = unsafe { gpiod_chip_info_get_name(info) };
        if result.is_null() {
            return Err(GpiodError::GetChipName);
        }
        // Safety: We checked that result is not null
        Ok(unsafe {
            std::ffi::CStr::from_ptr(result)
                .to_string_lossy()
                .to_string()
        })
    }
    /// Creates a new GPIO line settings object.
    ///
    /// # Safety
    /// - The caller must ensure that the returned pointer is freed using `gpiod_line_settings_free()`.
    fn settings(&self) -> Result<*mut gpiod_line_settings, GpiodError> {
        let result = unsafe { gpiod_line_settings_new() };
        if result.is_null() {
            return Err(GpiodError::CreateSettings);
        }
        Ok(result)
    }
    /// Sets the drive bias for a GPIO line.
    ///
    /// # Safety
    /// - `settings` must be a valid, non-null pointer to a `gpiod_line_settings` instance.
    fn settings_set_drive(
        &self,
        settings: *mut gpiod_line_settings,
        bias: gpiod_line_bias,
    ) -> Result<(), GpiodError> {
        if settings.is_null() {
            return Err(GpiodError::NullPtr);
        }
        let result = unsafe { gpiod_line_settings_set_drive(settings, bias) };
        if result != 0 {
            return Err(GpiodError::SetBias(bias));
        }
        Ok(())
    }
    /// Sets the direction of a GPIO line.
    ///
    /// # Safety
    /// - `settings` must be a valid, non-null pointer to a `gpiod_line_settings` instance.
    fn settings_set_direction(
        &self,
        settings: *mut gpiod_line_settings,
        direction: gpiod_line_direction,
    ) -> Result<(), GpiodError> {
        if settings.is_null() {
            return Err(GpiodError::NullPtr);
        }
        let result = unsafe { gpiod_line_settings_set_direction(settings, direction) };
        if result != 0 {
            return Err(GpiodError::SetDirection(direction));
        }
        Ok(())
    }
    /// Creates a new GPIO line configuration object.
    ///
    /// # Safety
    /// - The caller must ensure that the returned pointer is freed using `gpiod_line_config_free()`.
    fn config(&self) -> Result<*mut gpiod_line_config, GpiodError> {
        let result = unsafe { gpiod_line_config_new() };
        if result.is_null() {
            return Err(GpiodError::CreateConfig);
        }
        Ok(result)
    }

    /// Adds a line setting to a configuration object.
    ///
    /// # Safety
    /// - `config` must be a valid, non-null pointer to a `gpiod_line_config` instance.
    /// - `settings` must be a valid, non-null pointer to a `gpiod_line_settings` instance.
    fn config_add_settings(
        &self,
        config: *mut gpiod_line_config,
        settings: *mut gpiod_line_settings,
    ) -> Result<::std::os::raw::c_int, GpiodError> {
        if config.is_null() || settings.is_null() {
            return Err(GpiodError::NullPtr);
        }
        let result = unsafe { gpiod_line_config_add_line_settings(config, &OFFSET, 1, settings) };
        if result != 0 {
            return Err(GpiodError::CreateConfig);
        }
        Ok(result)
    }

    /// Requests a GPIO line.
    ///
    /// # Safety
    /// - `chip` must be a valid, non-null pointer to a `gpiod_chip` instance.
    /// - `line_cfg` must be a valid, non-null pointer to a `gpiod_line_config` instance.
    /// - The returned `gpiod_line_request` pointer must be freed properly.
    fn chip_request_lines(
        &self,
        chip: *mut gpiod_chip,
        line_cfg: *mut gpiod_line_config,
    ) -> Result<*mut gpiod_line_request, GpiodError> {
        if chip.is_null() || line_cfg.is_null() {
            return Err(GpiodError::NullPtr);
        }
        let result = unsafe { gpiod_chip_request_lines(chip, ptr::null_mut(), line_cfg) };
        if result.is_null() {
            return Err(GpiodError::LineRequest);
        }
        Ok(result)
    }

    /// Sets the value of a GPIO line request.
    ///
    /// # Safety
    /// - `request` must be a valid, non-null pointer to a `gpiod_line_request` instance.
    fn line_request_set_value(
        &self,
        request: *mut gpiod_line_request,
        offset: ::std::os::raw::c_uint,
        value: gpiod_line_value,
    ) -> Result<(), GpiodError> {
        if request.is_null() {
            return Err(GpiodError::NullPtr);
        }
        let result = unsafe { gpiod_line_request_set_value(request, offset, value) };
        if result != 0 {
            return Err(GpiodError::LineRequestSetValue);
        }
        Ok(())
    }

    /// Reconfigures a line request.
    ///
    /// # Safety
    /// - `request` must be a valid, non-null pointer to a `gpiod_line_request` instance.
    /// - `config` must be a valid, non-null pointer to a `gpiod_line_config` instance.
    fn line_request_reconfigure_lines(
        &self,
        request: *mut gpiod_line_request,
        config: *mut gpiod_line_config,
    ) -> Result<(), GpiodError> {
        if request.is_null() || config.is_null() {
            return Err(GpiodError::NullPtr);
        }
        let result = unsafe { gpiod_line_request_reconfigure_lines(request, config) };
        if result != 0 {
            return Err(GpiodError::LineRequestSetValue);
        }
        Ok(())
    }

    /// Gets the value of a GPIO line request.
    ///
    /// # Safety
    /// - `request` must be a valid, non-null pointer to a `gpiod_line_request` instance.
    fn line_request_get_value(
        &self,
        request: *mut gpiod_line_request,
        offset: ::std::os::raw::c_uint,
    ) -> Result<bool, GpiodError> {
        if request.is_null() {
            return Err(GpiodError::NullPtr);
        }
        let result = unsafe { gpiod_line_request_get_value(request, offset) };
        if result == -1 {
            return Err(GpiodError::LineRequestGetValue);
        }
        Ok(result == 1)
    }
}

// FIXME: Can this move into a Drop implementation?
pub fn cleanup(
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
    #![allow(clippy::manual_c_str_literals)]

    use super::*;
    use simple_test_case::test_case;
    use std::ptr;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    static CONFIG_FREED: AtomicUsize = AtomicUsize::new(0);
    static SETTINGS_FREED: AtomicUsize = AtomicUsize::new(0);
    static INFO_FREED: AtomicUsize = AtomicUsize::new(0);
    static CHIP_FREED: AtomicUsize = AtomicUsize::new(0);

    // Override external functions provided by bindgen.
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_chip_open(ptr: *const i8) -> *mut gpiod_chip {
        if ptr.is_null() {
            return ptr::null_mut();
        }
        1 as *mut gpiod_chip
    }

    // Mock result for gpiod_chip_get_info
    static GPIOD_CHIP_GET_INFO_RESULT: AtomicBool = AtomicBool::new(false);
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_chip_get_info(_: *mut gpiod_chip) -> *mut gpiod_chip_info {
        if GPIOD_CHIP_GET_INFO_RESULT.load(Ordering::SeqCst) {
            return 1 as *mut gpiod_chip_info;
        }
        ptr::null_mut()
    }

    // Mock result for gpiod_chip_info_get_name
    static GPIOD_CHIP_GET_NAME_RESULT: AtomicBool = AtomicBool::new(false);
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_chip_info_get_name(_: *mut gpiod_chip_info) -> *const i8 {
        if GPIOD_CHIP_GET_NAME_RESULT.load(Ordering::SeqCst) {
            return b"dummy_chip\0".as_ptr() as *const i8;
        }
        ptr::null()
    }

    // Mock result for gpiod_line_settings_new
    static GPIOD_SETTINGS_CREATED: AtomicBool = AtomicBool::new(false);
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_line_settings_new() -> *mut gpiod_line_settings {
        if GPIOD_SETTINGS_CREATED.load(Ordering::SeqCst) {
            return 1 as *mut gpiod_line_settings;
        }
        ptr::null_mut()
    }

    // Mock result for gpiod_line_settings_set_drive
    static GPIOD_SETTINGS_DRIVE_SET: AtomicBool = AtomicBool::new(false);
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_line_settings_set_drive(
        _: *mut gpiod_line_settings,
        _: gpiod_line_bias,
    ) -> i32 {
        if GPIOD_SETTINGS_DRIVE_SET.load(Ordering::SeqCst) {
            return 0;
        }
        -1
    }

    // Mock result for gpiod_line_settings_set_direction
    static GPIOD_SETTINGS_DIRECTION_SET: AtomicBool = AtomicBool::new(false);
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_line_settings_set_direction(
        _: *mut gpiod_line_settings,
        _: gpiod_line_direction,
    ) -> i32 {
        if GPIOD_SETTINGS_DIRECTION_SET.load(Ordering::SeqCst) {
            return 0;
        }
        -1
    }

    // Mock result for gpiod_line_config_new
    static GPIOD_CONFIG_CREATED: AtomicBool = AtomicBool::new(false);
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_line_config_new() -> *mut gpiod_line_config {
        if GPIOD_CONFIG_CREATED.load(Ordering::SeqCst) {
            return 1 as *mut gpiod_line_config;
        }
        ptr::null_mut()
    }

    static GPIOD_CONFIG_ADD_SETTINGS_RESULT: AtomicBool = AtomicBool::new(false);
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_line_config_add_line_settings(
        _: *mut gpiod_line_config,
        _: *const std::os::raw::c_uint,
        _: i32,
        _: *mut gpiod_line_settings,
    ) -> i32 {
        if GPIOD_CONFIG_ADD_SETTINGS_RESULT.load(Ordering::SeqCst) {
            return 0;
        }
        -1
    }

    static GPIOD_CHIP_REQUEST_LINES_RESULT: AtomicBool = AtomicBool::new(false);
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_chip_request_lines(
        _: *mut gpiod_chip,
        _: *mut gpiod_request_config,
        _: *mut gpiod_line_config,
    ) -> *mut gpiod_line_request {
        if GPIOD_CHIP_REQUEST_LINES_RESULT.load(Ordering::SeqCst) {
            return 1 as *mut gpiod_line_request;
        }
        ptr::null_mut()
    }

    static GPIOD_LINE_REQUEST_SET_VALUE_RESULT: AtomicBool = AtomicBool::new(false);
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_line_request_set_value(
        _: *mut gpiod_line_request,
        _: std::os::raw::c_uint,
        _: gpiod_line_value,
    ) -> i32 {
        if GPIOD_LINE_REQUEST_SET_VALUE_RESULT.load(Ordering::SeqCst) {
            return 0;
        }
        -1
    }

    static GPIOD_LINE_REQUEST_RECONFIGURE_LINES_RESULT: AtomicBool = AtomicBool::new(false);
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_line_request_reconfigure_lines(
        _: *mut gpiod_line_request,
        _: *mut gpiod_line_config,
    ) -> i32 {
        if GPIOD_LINE_REQUEST_RECONFIGURE_LINES_RESULT.load(Ordering::SeqCst) {
            return 0;
        }
        -1
    }

    static GPIOD_LINE_REQUEST_GET_VALUE_RESULT: AtomicBool = AtomicBool::new(false);
    #[no_mangle]
    pub unsafe extern "C" fn gpiod_line_request_get_value(
        _: *mut gpiod_line_request,
        _: std::os::raw::c_uint,
    ) -> i32 {
        if GPIOD_LINE_REQUEST_GET_VALUE_RESULT.load(Ordering::SeqCst) {
            return 1;
        }
        -1
    }

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

    #[test_case(b"dummy\0".as_ptr() as *const i8; "create chip")]
    #[test_case(ptr::null_mut(); "fail to create chip")]
    #[test]
    fn test_gpio_chip_open(ptr: *const i8) {
        let chip = Gpiod {}.chip(ptr);
        if ptr.is_null() {
            assert!(chip.is_err());
        } else {
            assert!(chip.is_ok());
        }
    }

    #[test_case(ptr::null_mut(), false; "fail on null ptr input")]
    #[test_case(1 as *mut gpiod_chip, false; "fail to get chip info")]
    #[test_case(1 as *mut gpiod_chip, true; "get chip info")]
    #[test]
    fn test_gpio_get_chip_info(chip: *mut gpiod_chip, desired: bool) {
        GPIOD_CHIP_GET_INFO_RESULT.store(desired, Ordering::SeqCst);
        let info = Gpiod {}.info(chip);
        assert_eq!(info.is_err(), !desired);
    }

    #[test_case(ptr::null_mut(), false; "fail on null ptr input")]
    #[test_case(1 as *mut gpiod_chip_info, false; "fail to get chip name")]
    #[test_case(1 as *mut gpiod_chip_info, true; "get chip name")]
    #[test]
    fn test_gpio_get_chip_name(info: *mut gpiod_chip_info, desired: bool) {
        GPIOD_CHIP_GET_NAME_RESULT.store(desired, Ordering::SeqCst);
        let name = Gpiod {}.name(info);
        assert_eq!(name.is_err(), !desired);
    }

    #[test_case(false; "fail to create settings")]
    #[test_case(true; "create settings")]
    #[test]
    fn test_gpio_create_settings(desired: bool) {
        GPIOD_SETTINGS_CREATED.store(desired, Ordering::SeqCst);
        let settings = Gpiod {}.settings();
        assert_eq!(settings.is_err(), !desired);
    }

    #[test_case(ptr::null_mut(), false; "fail on null ptr input")]
    #[test_case(1 as *mut gpiod_line_settings, false; "fail to set drive")]
    #[test_case(1 as *mut gpiod_line_settings, true; "set drive")]
    #[test]
    fn test_gpio_set_drive(settings: *mut gpiod_line_settings, desired: bool) {
        GPIOD_SETTINGS_DRIVE_SET.store(desired, Ordering::SeqCst);
        let result = Gpiod {}.settings_set_drive(settings, gpiod_line_bias_GPIOD_LINE_BIAS_PULL_UP);
        assert_eq!(result.is_err(), !desired);
    }

    #[test_case(ptr::null_mut(), false; "fail on null ptr input")]
    #[test_case(1 as *mut gpiod_line_settings, false; "fail to set direction")]
    #[test_case(1 as *mut gpiod_line_settings, true; "set direction")]
    #[test]
    fn test_gpio_set_direction(settings: *mut gpiod_line_settings, desired: bool) {
        GPIOD_SETTINGS_DIRECTION_SET.store(desired, Ordering::SeqCst);
        let result = Gpiod {}.settings_set_direction(settings, 1);
        assert_eq!(result.is_err(), !desired);
    }

    #[test_case(false; "fail to create config")]
    #[test_case(true; "create config")]
    #[test]
    fn test_gpio_create_config(desired: bool) {
        GPIOD_CONFIG_CREATED.store(desired, Ordering::SeqCst);
        let config = Gpiod {}.config();
        assert_eq!(config.is_err(), !desired);
    }

    #[test_case(ptr::null_mut(), ptr::null_mut(), false; "fail on null ptr input")]
    #[test_case(1 as *mut gpiod_line_config, 1 as *mut gpiod_line_settings, false; "fail to add settings")]
    #[test_case(1 as *mut gpiod_line_config, 1 as *mut gpiod_line_settings, true; "add settings")]
    #[test]
    fn test_gpio_add_settings(
        config: *mut gpiod_line_config,
        settings: *mut gpiod_line_settings,
        desired: bool,
    ) {
        GPIOD_CONFIG_ADD_SETTINGS_RESULT.store(desired, Ordering::SeqCst);
        let result = Gpiod {}.config_add_settings(config, settings);
        assert_eq!(result.is_err(), !desired);
    }

    #[test_case(ptr::null_mut(), ptr::null_mut(), false; "fail on null ptr input")]
    #[test_case(1 as *mut gpiod_chip, 1 as *mut gpiod_line_config, false; "fail to request lines")]
    #[test_case(1 as *mut gpiod_chip, 1 as *mut gpiod_line_config, true; "request lines")]
    #[test]
    fn test_gpio_chip_request_lines(
        chip: *mut gpiod_chip,
        line_cfg: *mut gpiod_line_config,
        desired: bool,
    ) {
        GPIOD_CHIP_REQUEST_LINES_RESULT.store(desired, Ordering::SeqCst);
        let result = Gpiod {}.chip_request_lines(chip, line_cfg);
        assert_eq!(result.is_err(), !desired);
    }

    #[test_case(ptr::null_mut(), 0, 1, false; "fail on null ptr input")]
    #[test_case(1 as *mut gpiod_line_request, 0, 1, false; "fail to set value")]
    #[test_case(1 as *mut gpiod_line_request, 0, 1, true; "set value")]
    #[test]
    fn test_gpio_line_request_set_value(
        request: *mut gpiod_line_request,
        offset: std::os::raw::c_uint,
        value: gpiod_line_value,
        desired: bool,
    ) {
        GPIOD_LINE_REQUEST_SET_VALUE_RESULT.store(desired, Ordering::SeqCst);
        let result = Gpiod {}.line_request_set_value(request, offset, value);
        assert_eq!(result.is_err(), !desired);
    }

    #[test_case(ptr::null_mut(), ptr::null_mut(), false; "fail on null ptr input")]
    #[test_case(1 as *mut gpiod_line_request, 1 as *mut gpiod_line_config, false; "fail to reconfigure lines")]
    #[test_case(1 as *mut gpiod_line_request, 1 as *mut gpiod_line_config, true; "reconfigure lines")]
    #[test]
    fn test_gpio_line_request_reconfigure_lines(
        request: *mut gpiod_line_request,
        config: *mut gpiod_line_config,
        desired: bool,
    ) {
        GPIOD_LINE_REQUEST_RECONFIGURE_LINES_RESULT.store(desired, Ordering::SeqCst);
        let result = Gpiod {}.line_request_reconfigure_lines(request, config);
        assert_eq!(result.is_err(), !desired);
    }

    #[test_case(ptr::null_mut(), 0, false; "fail on null ptr input")]
    #[test_case(1 as *mut gpiod_line_request, 0, false; "fail to get value")]
    #[test_case(1 as *mut gpiod_line_request, 0, true; "get value")]
    #[test]
    fn test_gpio_line_request_get_value(
        request: *mut gpiod_line_request,
        offset: std::os::raw::c_uint,
        desired: bool,
    ) {
        GPIOD_LINE_REQUEST_GET_VALUE_RESULT.store(desired, Ordering::SeqCst);
        let result = Gpiod {}.line_request_get_value(request, offset);
        assert_eq!(result.is_err(), !desired);
        if desired {
            assert_eq!(result.unwrap(), true); // hardcoded value from mock
        }
    }

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
