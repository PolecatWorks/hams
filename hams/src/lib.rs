use self::hams::Hams;
use ffi_helpers::catch_panic;
use ffi_log2::{logger_init, LogParam};
use log::info;
use std::ffi::CStr;
use std::process;
mod hams;
// pub mod ffi;
pub mod error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialise the FFI based logging for this crate
#[no_mangle]
pub extern "C" fn hams_logger_init(param: LogParam) -> i32 {
    // ffi_helpers::null_pointer_check!(param);

    catch_panic!(
        logger_init(param);
        info!(
            "Logging registered for {}:{} (PID: {})",
            NAME,
            VERSION,
            process::id()
        );
        Ok(1)
    )
}

/** Initialise the HaMS
 *
 */
#[no_mangle]
pub extern "C" fn hams_init<'a>(name: *const libc::c_char) -> *mut Hams {
    ffi_helpers::null_pointer_check!(name);

    catch_panic!(
        let name_str = unsafe {CStr::from_ptr(name) }.to_str().unwrap();
        info!("Registering HaMS: {}", name_str);

        Ok(Box::into_raw(Box::new(Hams::new(name_str))))
    )
}

/// Free the HaMS
#[no_mangle]
pub extern "C" fn hams_free(ptr: *mut Hams) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let hams = unsafe { Box::from_raw(ptr) };

        let name = &hams.as_ref().name;

        info!("Releasing hams: {}", name);
        drop(hams);
        Ok(1)
    )
}

#[no_mangle]
pub extern "C" fn hams_start(ptr: *mut Hams) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let hams = unsafe {&mut *ptr};
        info!("start my ham {}", hams.name);
        Ok(1)
    )
}

#[cfg(test)]
mod tests {

    use std::ptr;

    use ffi_log2::log_param;

    use crate::error::ffi_error_to_result;

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

        assert_eq!(retval, 1);
    }

    #[test]
    fn null_init() {
        // let c_library_name: libc::c_char = ptr::null();
        let my_hams = hams_init(ptr::null());

        assert_eq!(my_hams, ptr::null_mut());

        assert!(ffi_error_to_result().is_err(), "Error should be returned");
    }

    #[test]
    fn null_free() {
        let retval = hams_free(ptr::null_mut());

        assert_eq!(retval, 0);

        assert!(ffi_error_to_result().is_err(), "Error should be returned");
    }
}
