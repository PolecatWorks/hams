use self::hams::Hams;
use ffi_log2::{logger_init, LogParam};
use log::info;
use std::{process, ptr};

mod hams;
// pub mod ffi;
pub mod error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialise the FFI based logging for this crate
#[no_mangle]
pub extern "C" fn hams_logger_init(param: LogParam) -> i32 {
    logger_init(param);
    info!(
        "Logging registered for {}:{} (PID: {})",
        NAME,
        VERSION,
        process::id()
    );
    0
}

/** Initialise the HaMS
 *
 */
#[no_mangle]
pub extern "C" fn hams_init<'a>(name: *const libc::c_char) -> *mut Hams {
    ffi_helpers::null_pointer_check!(name);

    // TODO: Correct this to follow the FFI safe error respose
    let name_str: &str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            ffi_helpers::update_last_error(e);
            return ptr::null_mut();
        }
    };
    info!("Registering HaMS: {}", name_str);

    Box::into_raw(Box::new(Hams::new(name_str)))
}

/** Free the HaMS
 *
 */
#[no_mangle]
pub extern "C" fn hams_free(ptr: *mut Hams) -> i32 {
    ffi_helpers::null_pointer_check!(ptr, -1);

    let name = &unsafe { &(*ptr) }.name;

    info!("Releasing hams: {}", name);

    drop(unsafe { Box::from_raw(ptr) });

    0
}

#[cfg(test)]
mod tests {

    use ffi_log2::log_param;

    use super::*;

    #[test]
    fn logger_init() {
        hams_logger_init(log_param());
    }

    #[test]
    fn init_free() {
        let c_library_name = std::ffi::CString::new("name").unwrap();

        let my_hams = hams_init(c_library_name.as_ptr());

        assert_ne!(my_hams, ptr::null_mut());

        println!("initialised HaMS");

        let retval = hams_free(my_hams);

        assert_eq!(retval, 0);
    }
}
