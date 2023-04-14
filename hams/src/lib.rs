// mod hams;

mod hams2;
mod wuggle;

// pub mod ffi;
pub mod error;
pub mod healthcheck;
pub mod healthkicked;

#[cfg(all(feature = "axum", feature = "warp"))]
compile_error!("feature \"axum\" and feature \"warp\" cannot be enabled at the same time");

use self::hams2::Hams;
use ffi_helpers::catch_panic;
use ffi_log2::{logger_init, LogParam};
use libc::c_int;
use log::info;
use std::ffi::CStr;
use std::process;

/// Name of the Crate
const NAME: &str = env!("CARGO_PKG_NAME");
/// Version of the Crate
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[no_mangle]
pub extern "C" fn hello_world() {
    println!("HOWDY World");
    println!("Hello I am {}:{}", NAME, VERSION);
}

#[no_mangle]
pub extern "C" fn hello_node() -> c_int {
    println!("HOWDY Node");
    println!("Hello I am {}:{}", NAME, VERSION);
    7
}

#[no_mangle]
pub extern "C" fn hello_callback(my_cb: extern "C" fn()) {
    println!("HOWDY callback");
    my_cb();
}

#[cfg_attr(doc, aquamarine::aquamarine)]
///
/// Register logging for uservice
/// ```mermaid
/// sequenceDiagram
///     participant Main
///     participant UService
///     participant Sample01
///
///     rect rgba(50,50,255,0.1)
///     note right of Main: Main register library and SoService
///
///     Main->>+UService: so_library_register
///     UService->>-Main: (SoLibrary)
///
///     Main->>+UService: so_service_register_ffi(SoLibrary)
///     UService->>-Main: (SoService)
///     end
/// ```
///
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

/// # Safety
///
/// Initialise the hams object giving it a name on construction
#[no_mangle]
pub unsafe extern "C" fn hams_init<'a>(name: *const libc::c_char) -> *mut Hams {
    ffi_helpers::null_pointer_check!(name);

    catch_panic!(
        let name_str = unsafe {CStr::from_ptr(name) }.to_str().unwrap();
        info!("Registering HaMS: {}", name_str);

        Ok(Box::into_raw(Box::new(Hams::new(name_str))))
    )
}

/// # Safety
///
/// Free the HaMS. The object must be created wtih the hams_init function
#[no_mangle]
pub unsafe extern "C" fn hams_free(ptr: *mut Hams) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let hams = unsafe { Box::from_raw(ptr) };

        let name = &hams.as_ref().name;

        info!("Releasing hams: {}", name);
        drop(hams);
        Ok(1)
    )
}

/// # Safety
///
/// Start the HaMS service. This requires a valid hams object constructed from hams_init
#[no_mangle]
pub unsafe extern "C" fn hams_start(ptr: *mut Hams) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let hams = unsafe {&mut *ptr};
        info!("start my ham {}", hams.name);
        hams.start().expect("Hams started");
        Ok(1)
    )
}

/// # Safety
///
/// Stop the HaMS service. This requires a valid hams object constructed from hams_init
#[no_mangle]
pub unsafe extern "C" fn hams_stop(ptr: *mut Hams) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let hams = unsafe {&mut *ptr};
        info!("stop my ham {}", hams.name);
        hams.stop().expect("Hams stopped");
        Ok(1)
    )
}

#[cfg(test)]
mod tests {

    use std::ptr;

    use ffi_log2::log_param;

    use crate::error::ffi_error_to_result;

    use super::*;

    #[ignore]
    #[test]
    fn logger_init() {
        let retval = hams_logger_init(log_param());

        assert_ne!(retval, 0);
    }

    #[test]
    fn init_free() {
        let c_library_name = std::ffi::CString::new("name").unwrap();

        let my_hams = unsafe { hams_init(c_library_name.as_ptr()) };

        assert_ne!(my_hams, ptr::null_mut());

        println!("initialised HaMS");

        let retval = unsafe { hams_free(my_hams) };

        assert_eq!(retval, 1);
    }

    #[test]
    fn null_init() {
        // let c_library_name: libc::c_char = ptr::null();
        let my_hams = unsafe { hams_init(ptr::null()) };

        assert_eq!(my_hams, ptr::null_mut());

        assert!(ffi_error_to_result().is_err(), "Error should be returned");
    }

    #[test]
    fn null_free() {
        let retval = unsafe { hams_free(ptr::null_mut()) };

        assert_eq!(retval, 0);

        assert!(ffi_error_to_result().is_err(), "Error should be returned");
    }
}
