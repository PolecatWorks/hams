#![warn(missing_docs)]

//! Provide a FFI interface to health utility funcitons

pub mod error;

mod hams;
mod tokio_tools;
mod webservice;
// use libc::c_void;

// pub mod ffi;

/// Health checks
pub mod health;

#[cfg(all(feature = "axum", feature = "warp"))]
compile_error!("feature \"axum\" and feature \"warp\" cannot be enabled at the same time");

use crate::health::probe::HealthProbe;

use self::hams::Hams;
use ffi_helpers::catch_panic;
use ffi_log2::{logger_init, LogParam};
use health::probe::kick::Kick;
use health::probe::manual::Manual;
use health::probe::BoxedHealthProbe;
use libc::c_int;
use log::info;
use std::ffi::CStr;
use std::process;
use std::time::Instant;

/// Name of the Crate
const NAME: &str = env!("CARGO_PKG_NAME");
/// Version of the Crate
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Fill this out
#[no_mangle]
pub extern "C" fn hello_world() {
    println!("HOWDY World");
    println!("Hello I am {}:{}", NAME, VERSION);
}

/// Fill this out
#[no_mangle]
pub extern "C" fn hello_node() -> c_int {
    println!("HOWDY Node");
    println!("Hello I am {}:{}", NAME, VERSION);
    7
}

/// Fill this out
#[no_mangle]
pub extern "C" fn hello_callback(my_cb: extern "C" fn()) {
    println!("HOWDY callback");
    my_cb();
}

/// Return the version of the library
#[no_mangle]
pub extern "C" fn hams_version() -> *const libc::c_char {
    let version = format!("{}:{}", NAME, VERSION);
    let c_version = std::ffi::CString::new(version).unwrap();
    c_version.into_raw()
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
pub unsafe extern "C" fn hams_new<'a>(name: *const libc::c_char) -> *mut Hams {
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
        hams.stop().expect("Hams stopped here");
        info!("HaMS stopped");
        Ok(1)
    )
}

/// # Safety
/// Insert a health probe into the alive list of a HaMS object
#[no_mangle]
pub unsafe extern "C" fn hams_add_alive(ptr: *mut Hams, probe: *mut BoxedHealthProbe) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);
    ffi_helpers::null_pointer_check!(probe);

    catch_panic!(
        let hams = unsafe {&mut *ptr};
        let probe = unsafe {&mut *probe};
        // let probe = unsafe { Box::from_raw(probe) };

        info!("Adding alive probe: {}", probe.name().unwrap_or("unknown".to_owned()));
        todo!("Add alive probe");
        // hams.alive_insert(probe.clone());
        // hams.add_alive(probe);
        Ok(1)
    )
}

// #[no_mangle]
// pub unsafe extern "C" fn hams_add_alive(ptr: *mut Hams, probe: *mut BoxedHealthProbe) -> i32 {
//     ffi_helpers::null_pointer_check!(ptr);
//     ffi_helpers::null_pointer_check!(probe);

//     catch_panic!(
//         let hams = unsafe {&mut *ptr};
//         let probe = unsafe { Box::from_raw(probe) };

//         info!("Adding alive probe: {}", probe.name().unwrap_or("unknown".to_owned()));
//         hams.add_alive(probe);
//         Ok(1)
//     )
// }

/// Return a manual health probe
///
/// # Safety
/// Create a manual health probe
#[no_mangle]
pub unsafe extern "C" fn probe_manual_new(name: *const libc::c_char, check: bool) -> *mut Manual {
    // TODO: Not sure if we should be returning a BoxedHealthProbe as this will mean we cannot call the set functions.
    ffi_helpers::null_pointer_check!(name);

    catch_panic!(
        let name_str = unsafe {CStr::from_ptr(name) }.to_str().unwrap();
        info!("Creating ManualHealthProbe: {}", name_str);

        let probe = health::probe::manual::Manual::new(name_str, check);

        Ok(Box::into_raw(Box::new(probe)))
    )
}

/// Return a boxed health probe from the manual health probe
/// # Safety
/// Return a boxed health probe from the manual health probe
#[no_mangle]
pub unsafe extern "C" fn probe_manual_boxed(ptr: *mut Manual) -> *mut BoxedHealthProbe<'static> {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let probe = &mut *ptr;
        let boxed_probe = probe.boxed_probe();
        Ok(Box::into_raw(Box::new(boxed_probe)))
    )
}

/// Free Health Probe
/// # Safety
/// Free the Health Probe. The object must be created with HaMS library
#[no_mangle]
pub unsafe extern "C" fn probe_free(ptr: *mut BoxedHealthProbe) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let probe = Box::from_raw(ptr);

        let name = &probe.name().unwrap_or("unknown".to_owned());

        info!("Releasing probe: {}", name);
        drop(probe);
        Ok(1)
    )
}

/// Free Manual Health Probe
///
/// # Safety
/// Free the Manual Health Probe. The object must be created with HaMS library
#[no_mangle]
pub unsafe extern "C" fn probe_manual_free(ptr: *mut Manual) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let probe = Box::from_raw(ptr);

        let name = &probe.name().unwrap_or("unknown".to_owned());

        info!("Releasing manual probe: {}", name);
        drop(probe);
        Ok(1)
    )
}

/// Enable the Manual Health Probe
/// # Safety
/// Enable the Manual Health Probe
/// This will set the check to true
#[no_mangle]
pub unsafe extern "C" fn probe_manual_enable(ptr: *mut Manual) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let probe = &mut *ptr;
        probe.enable();
        Ok(1)
    )
}

/// Disable the Manual Health Probe
/// # Safety
/// Disable the Manual Health Probe
/// This will set the check to false
#[no_mangle]
pub unsafe extern "C" fn probe_manual_disable(ptr: *mut Manual) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let probe = &mut *ptr;
        probe.disable();
        Ok(1)
    )
}

/// Toggle the Manual Health Probe
/// # Safety
/// Toggle the Manual Health Probe
/// This will set the check to the opposite of the current value
#[no_mangle]
pub unsafe extern "C" fn probe_manual_toggle(ptr: *mut Manual) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let probe = &mut *ptr;
        probe.toggle();
        Ok(1)
    )
}

/// Check the Manual Health Probe
/// # Safety
/// Check the Manual Health Probe
/// This will return the current value of the check
#[no_mangle]
pub unsafe extern "C" fn probe_manual_check(ptr: *mut Manual) -> i32 {
    ffi_helpers::null_pointer_check!(ptr, -1);

    let now = Instant::now();
    catch_panic!(
        let probe = &mut *ptr;

        match probe.check(now) {
            Ok(x) => Ok(x as i32),
            Err(_) => Ok(-1_i32),
        }
    )
}

/// Return a kick health probe
///
/// # Safety
/// Create a kick health probe
pub unsafe extern "C" fn probe_kick_new(
    name: *const libc::c_char,
    margin_secs: c_int,
) -> *mut Kick {
    ffi_helpers::null_pointer_check!(name);

    catch_panic!(
        let name_str = unsafe {CStr::from_ptr(name) }.to_str().unwrap();
        let margin = std::time::Duration::from_secs(margin_secs as u64);
        info!("Creating KickHealthProbe: {}", name_str);

        let probe = health::probe::kick::Kick::new(name_str, margin);
        Ok(Box::into_raw(Box::new(probe)))
    )
}

/// Call kick method
///
/// # Safety
/// Call the kick method on the Kick object
pub unsafe extern "C" fn probe_kick_kick(ptr: *mut Kick) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let probe = &mut *ptr;
        probe.kick();
        Ok(1)
    )
}

/// Free Health Probe
///
/// # Safety
/// Free the Health Probe. The object must be created with HaMS library
#[no_mangle]
pub unsafe extern "C" fn probe_kick_free(ptr: *mut Kick) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let probe = Box::from_raw(ptr);

        let name = &probe.name().unwrap_or("unknown".to_owned());

        info!("Releasing kick probe: {}", name);
        drop(probe);
        Ok(1)
    )
}

/// Return a boxed health probe from the manual health probe
/// # Safety
/// Return a boxed health probe from the manual health probe
#[no_mangle]
pub unsafe extern "C" fn probe_kick_boxed(ptr: *mut Kick) -> *mut BoxedHealthProbe<'static> {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let probe = &mut *ptr;
        let boxed_probe = probe.boxed_probe();
        Ok(Box::into_raw(Box::new(boxed_probe)))
    )
}

// /// # Safety
// ///
// /// Create an alive kicked health check
// #[no_mangle]
// pub unsafe extern "C" fn kicked_create(
//     name: *const libc::c_char,
//     duration_millis: libc::c_ulong,
// ) -> *mut AliveCheckKicked {
//     ffi_helpers::null_pointer_check!(name);

//     catch_panic!(
//         let name_str = unsafe {CStr::from_ptr(name) }.to_str().unwrap();
//         info!("Creating AliveCheckKicked: {}", name_str);

//         Ok(Box::into_raw(Box::new(AliveCheckKicked::new(name_str, Duration::from_millis(duration_millis)))))
//     )
// }
// /// # Safety
// ///
// /// Free the AliveCheckKicked. The object must be created wtih the kicked_create function
// #[no_mangle]
// pub unsafe extern "C" fn kicked_free(ptr: *mut AliveCheckKicked) -> i32 {
//     ffi_helpers::null_pointer_check!(ptr);

//     catch_panic!(

//         let kicked = unsafe { Box::from_raw(ptr) };

//         let name = &kicked.as_ref().name;

//         info!("Releasing kicked: {}", name);
//         drop(kicked);
//         Ok(1)
//     )
// }

// /// # Safety
// ///
// /// kick the AliveCheckKicked
// #[no_mangle]
// pub unsafe extern "C" fn kicked_kick(ptr: *mut AliveCheckKicked) -> i32 {
//     ffi_helpers::null_pointer_check!(ptr);

//     catch_panic!(
//         let kicked = unsafe {&mut *ptr};

//         // info!("Kicking {}", kicked.name);
//         kicked.kick();

//         Ok(1)
//     )
// }

// /// # Safety
// ///
// /// Register a shutdown function to be called when the health system receives a trigger to shutdown
// /// This could be a kubernetes shutdown hook or a sig event
// #[no_mangle]
// pub unsafe extern "C" fn hams_register_shutdown(
//     hams_ptr: *mut Hams,
//     user_data: *mut c_void,
//     cb: unsafe extern "C" fn(*mut c_void),
// ) -> i32 {
//     ffi_helpers::null_pointer_check!(hams_ptr);
//     // ffi_helpers::null_pointer_check!(my_cb);

//     catch_panic!(
//         let hams = unsafe {&mut *hams_ptr};

//         hams.register_shutdown(user_data, cb);

//         Ok(1)
//     )
// }

// /// # Safety
// ///
// /// kick the AliveCheckKicked
// #[no_mangle]
// pub unsafe extern "C" fn hams_add_alive(
//     hams_ptr: *mut Hams,
//     alive_ptr: *mut AliveCheckKicked,
// ) -> i32 {
//     ffi_helpers::null_pointer_check!(hams_ptr);
//     ffi_helpers::null_pointer_check!(alive_ptr);

//     catch_panic!(
//         let hams = unsafe {&mut *hams_ptr};
//         let alive = unsafe {&mut *alive_ptr};

//         // info!("Kicking {}", kicked.name);
//         hams.add_alive(Box::new(alive.clone()));

//         Ok(1)
//     )
// }

// /// # Safety
// ///
// /// kick the AliveCheckKicked
// #[no_mangle]
// pub unsafe extern "C" fn hams_remove_alive(
//     hams_ptr: *mut Hams,
//     alive_ptr: *mut AliveCheckKicked,
// ) -> i32 {
//     ffi_helpers::null_pointer_check!(hams_ptr);
//     ffi_helpers::null_pointer_check!(alive_ptr);

//     catch_panic!(
//         let hams = unsafe {&mut *hams_ptr};
//         let alive = unsafe {&mut *alive_ptr};

//         // info!("Kicking {}", kicked.name);
//         hams.remove_alive(Box::new(alive.clone()));

//         Ok(1)
//     )
// }

/// Test the FFI interfaces
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
    fn hams_init_free() {
        let c_library_name = std::ffi::CString::new("name").unwrap();

        let my_hams = unsafe { hams_new(c_library_name.as_ptr()) };

        assert_ne!(my_hams, ptr::null_mut());

        println!("initialised HaMS");

        let retval = unsafe { hams_free(my_hams) };

        assert_eq!(retval, 1);
    }

    #[test]
    fn null_init() {
        // let c_library_name: libc::c_char = ptr::null();
        let my_hams = unsafe { hams_new(ptr::null()) };

        assert_eq!(my_hams, ptr::null_mut());

        assert!(ffi_error_to_result().is_err(), "Error should be returned");
    }

    #[test]
    fn null_free() {
        let retval = unsafe { hams_free(ptr::null_mut()) };

        assert_eq!(retval, 0);

        assert!(ffi_error_to_result().is_err(), "Error should be returned");
    }

    // create and free manual probe
    #[test]
    fn probe_manual_create_free() {
        let c_probe_name = std::ffi::CString::new("name").unwrap();

        let my_probe = unsafe { probe_manual_new(c_probe_name.as_ptr(), true) };

        assert_ne!(my_probe, ptr::null_mut());

        println!("initialised Manual Probe");

        let retval = unsafe { probe_manual_free(my_probe) };

        assert_eq!(retval, 1);
    }

    // Create and free kick probe
    #[test]
    fn probe_kick_create_free() {
        let c_probe_name = std::ffi::CString::new("name").unwrap();

        let my_probe = unsafe { probe_kick_new(c_probe_name.as_ptr(), 10) };

        assert_ne!(my_probe, ptr::null_mut());

        println!("initialised Kick Probe");

        let retval = unsafe { probe_kick_free(my_probe) };

        assert_eq!(retval, 1);
    }

    // Create Hams and insert + remove manual probe
    #[test]
    fn hams_start_stop() {
        let c_library_name = std::ffi::CString::new("name").unwrap();

        let my_hams = unsafe { hams_new(c_library_name.as_ptr()) };

        assert_ne!(my_hams, ptr::null_mut());

        println!("initialised HaMS");

        // let retval = unsafe { hams_start(my_hams) };

        // assert_eq!(retval, 1);

        // let retval = unsafe { hams_stop(my_hams) };

        // assert_eq!(retval, 1);

        let retval = unsafe { hams_free(my_hams) };

        assert_eq!(retval, 1);
    }
}
