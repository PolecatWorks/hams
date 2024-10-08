#![warn(missing_docs)]

//! Provide a FFI interface to health utility funcitons

pub mod error;
mod hams;
mod preflight;
/// This module provides the health probes
pub mod probe;
mod tokio_tools;

use crate::probe::ffitraits::HealthProbe;

/// Health checks
use crate::probe::{AsyncHealthProbe, FFIProbe};

use self::hams::Hams;
use error::{FFIEnum, HamsError};
use ffi_helpers::catch_panic;
use ffi_log2::{logger_init, LogParam};
use hams::config::HamsConfig;
use libc::{c_int, c_void};
use log::{error, info};
use probe::ffitraits::BoxedHealthProbe;
use probe::kick::Kick;
use probe::manual::Manual;

use std::ffi::{CStr, CString};
use std::panic::AssertUnwindSafe;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

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
    my_cb();
    my_cb();
    my_cb();
}

/// C function to take two functions as callbacks.
/// The first function returns a c string the second frees the c string
#[no_mangle]
pub extern "C" fn hello_callback2(
    my_cb: extern "C" fn() -> *const libc::c_char,
    my_cb_free: extern "C" fn(*const libc::c_char),
) {
    println!("HOWDY callback2");
    let c_string = my_cb();
    let c_string = unsafe { CStr::from_ptr(c_string) };
    println!("C string: {:?}", c_string);
    my_cb_free(c_string.as_ptr());
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
pub unsafe extern "C" fn hams_new(
    name: *const libc::c_char,
    address: *const libc::c_char,
) -> *mut Hams {
    ffi_helpers::null_pointer_check!(name);
    ffi_helpers::null_pointer_check!(address);

    catch_panic!(
        let name_str =  unsafe { CStr::from_ptr(name) }
            .to_str()
            .map_err(HamsError::from)?;

        let address_str =  unsafe { CStr::from_ptr(address) }
            .to_str()
            .map_err(HamsError::from)?;

        let config = HamsConfig{
            address: address_str.parse()?,
            name: name_str.to_string()
         };

        info!("Registering HaMS: {}", name_str);
        Ok(Box::into_raw(Box::new(Hams::new(config))))
    )
}

/// # Safety
///
/// Free the HaMS. The object must be created wtih the hams_init function
#[no_mangle]
pub unsafe extern "C" fn hams_free(ptr: *mut Hams) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    let hams = AssertUnwindSafe(unsafe { Box::from_raw(ptr) });

    catch_panic!(
        let name = &hams.as_ref().name;

        info!("Releasing hams: {}", name);
        drop(hams);
        Ok(1)
    )
}

/// # Safety
/// Register the prometheus callback
/// This will register the prometheus callback with the HaMS object
/// ```rust
/// let x = 3;
/// assert_eq!(x, 3);
///
/// ```
/// # Safety
///
/// Register the prometheus callback
/// This will register the prometheus callback with the HaMS object
/// ```rust
/// use libc;
/// use self::{hams_new,hams_register_prometheus};
///
/// // Define the callback function
/// extern "C" fn prometheus_callback(state: *const libc::c_void) -> *mut libc::c_char {
///       let prometheus = String::from("test");
///       let c_str_prometheus = std::ffi::CString::new(prometheus).unwrap();
///       c_str_prometheus.into_raw()
/// }
///
/// // Define the callback function to free the allocated memory
/// extern "C" fn prometheus_callback_free(ptr: *mut libc::c_char) {
///     unsafe {
///         if !ptr.is_null() {
///             std::ffi::CString::from_raw(ptr);
///         }
///     }
/// }
///
/// // Create a HaMS object
/// let name = std::ffi::CString::new("MyHaMS").unwrap();
/// let hams = unsafe { hams_new(name.as_ptr()) };
///
/// // Register the prometheus callback
/// let result = unsafe {
///     hams_register_prometheus(
///         hams,
///         prometheus_callback,
///         prometheus_callback_free,
///         std::ptr::null()
///     )
/// };
///
/// assert_eq!(result, 1);
/// ```
#[no_mangle]
pub unsafe extern "C" fn hams_register_prometheus(
    ptr: *mut Hams,
    my_cb: extern "C" fn(ptr: *const c_void) -> *mut libc::c_char,
    my_cb_free: extern "C" fn(*mut libc::c_char),
    state: *const c_void,
) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);
    let hams = AssertUnwindSafe(unsafe { &mut *ptr });
    info!("Registering Prometheus callback for {}", hams.name);

    catch_panic!(
        AssertUnwindSafe(hams).register_prometheus(my_cb, my_cb_free, state).expect("Register prometheus callbacks");

        Ok(1)
    )

    // )
}

/// Degregister promethues from HaMS
///
/// https://stackoverflow.com/questions/65762689/how-can-assertunwindsafe-be-used-with-the-catchunwind-future suggests we need to use AssertUnwindSafe to allow the use of async inside the catch_panic
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn hams_deregister_prometheus(ptr: *mut Hams) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    let hams = AssertUnwindSafe(unsafe { &mut *ptr });
    info!("Deregistering Prometheus callback for {}", hams.name);

    catch_panic!(
        AssertUnwindSafe(hams).deregister_prometheus().expect("Deregister prometheus");
        Ok(1)
    )
}

/// Register a shutdown callback
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn hams_register_shutdown(
    ptr: *mut Hams,
    my_cb: extern "C" fn(ptr: *mut c_void),
    state: *mut c_void,
) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    let hams = AssertUnwindSafe(unsafe { &mut *ptr });
    info!("Registering Shutdown callback for {}", hams.name);

    catch_panic!(
        AssertUnwindSafe(hams).register_shutdown(my_cb, state).expect("Register shutdown callbacks");
        Ok(FFIEnum::Success as i32)
    )
}

/// DeRegister a shutdown callback
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn hams_deregister_shutdown(ptr: *mut Hams) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    let hams = AssertUnwindSafe(unsafe { &mut *ptr });
    info!("Deregistering Shutdown callback for {}", hams.name);

    catch_panic!(
        AssertUnwindSafe(hams).deregister_shutdown().expect("Deregister shutdown");
        Ok(FFIEnum::Success as i32)
    )
}

/// # Safety
///
/// Start the HaMS service. This requires a valid hams object constructed from hams_init
#[no_mangle]
pub unsafe extern "C" fn hams_start(ptr: *mut Hams) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    let hams = AssertUnwindSafe(unsafe { &mut *ptr });
    info!("start my ham {}", hams.name);
    catch_panic!(
        AssertUnwindSafe(hams).start().expect("Start HaMS");
        Ok(1)
    )
}

/// # Safety
///
/// Stop the HaMS service. This requires a valid hams object constructed from hams_init
#[no_mangle]
pub unsafe extern "C" fn hams_stop(ptr: *mut Hams) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    let hams = AssertUnwindSafe(unsafe { &mut *ptr });
    info!("Stop HaMS {}", hams.name);
    catch_panic!(match AssertUnwindSafe(hams).stop() {
        Ok(_) => {
            info!("HaMS stopped");
            Ok(FFIEnum::Success as i32)
        }
        Err(e) => {
            match e {
                HamsError::NotRunning => {
                    info!("HaMS already stopped");
                    ffi_helpers::update_last_error(e);
                    Ok(FFIEnum::NotRunning as i32)
                }
                _ => {
                    println!("Failed to stop HaMS: {}", e);
                    Err(e.into())
                }
            }
        }
    })
}

/// # Safety
/// Insert a health probe into the alive list of a HaMS object
/// This will take ownership of the probe and store it
#[no_mangle]
pub unsafe extern "C" fn hams_alive_insert(
    ptr: *mut Hams,
    probe: *mut BoxedHealthProbe<'static>,
) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);
    ffi_helpers::null_pointer_check!(probe);

    let hams = AssertUnwindSafe(unsafe { &mut *ptr });
    catch_panic!(

        // Take ownership of the probe
        let probe = unsafe { BoxedHealthProbe::from_raw(probe as *mut () ) };

        info!("Adding alive probe: {}", CString::from_raw(probe.name()).into_string().unwrap());

        // Convert a BoxedHealthProbe to a FFIProbe (which is a Box<dyn AsyncHealthProbe>) so we can store it
        let ffi_probe = Box::new(FFIProbe::from(probe)) as Box<dyn AsyncHealthProbe>;

        let retval_bool = if AssertUnwindSafe(hams).alive_insert(ffi_probe) {
            1
        } else {
            0
        };
        Ok(retval_bool)
    )
}

/// # Safety
/// Remove a health probe from the alive list of a HaMS object
#[no_mangle]
pub unsafe extern "C" fn hams_alive_remove(
    ptr: *mut Hams,
    probe: *mut BoxedHealthProbe<'static>,
) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);
    ffi_helpers::null_pointer_check!(probe);

    let hams = AssertUnwindSafe(unsafe { &mut *ptr });

    catch_panic!(
        // Take ownership of the probe
        let probe = unsafe { BoxedHealthProbe::from_raw(probe as *mut () ) };

        info!("Removing alive probe: {}", CString::from_raw(probe.name()).into_string().unwrap());

        let ffi_probe = Box::new(FFIProbe::from(probe)) as Box<dyn AsyncHealthProbe>;
        match AssertUnwindSafe(hams).alive_remove(&ffi_probe) {
            true => Ok(1),
            false => Ok(0),
        }
        // drop ffi_probe will delete the probe from the heap
        // AND the alive_remove should have dropped the one from the list
    )
}

/// # Safety
/// Insert a health probe into the ready list of a HaMS object
/// This will NOT take ownership of the probe but will store a copy of it
#[no_mangle]
pub unsafe extern "C" fn hams_ready_insert(
    ptr: *mut Hams,
    probe: *mut BoxedHealthProbe<'static>,
) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);
    ffi_helpers::null_pointer_check!(probe);

    let hams = AssertUnwindSafe(unsafe { &mut *ptr });
    catch_panic!(

          // Take ownership of the probe
          let probe = unsafe { BoxedHealthProbe::from_raw(probe as *mut () ) };

          info!("Adding alive probe: {}", CString::from_raw(probe.name()).into_string().unwrap());
          println!("Adding alive probe: {:?}", CString::from_raw(probe.name()).into_string().unwrap());

          // Convert a BoxedHealthProbe to a FFIProbe (which is a Box<dyn AsyncHealthProbe>) so we can store it
          let ffi_probe = Box::new(FFIProbe::from(probe)) as Box<dyn AsyncHealthProbe>;
          println!("using FFIProbe {:?}", ffi_probe.name());

        if AssertUnwindSafe(hams).ready_insert(ffi_probe) {
            Ok(1)
        } else {
            Ok(0)
        }
    )
}

/// # Safety
/// Remove a health probe from the ready list of a HaMS object
#[no_mangle]
pub unsafe extern "C" fn hams_ready_remove(
    ptr: *mut Hams,
    probe: *mut BoxedHealthProbe<'static>,
) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);
    ffi_helpers::null_pointer_check!(probe);

    let hams = AssertUnwindSafe(unsafe { &mut *ptr });

    catch_panic!(
        // Take ownership of the probe
        let probe = unsafe { BoxedHealthProbe::from_raw(probe as *mut () ) };

        info!("Removing alive probe: {}", CString::from_raw(probe.name()).into_string().unwrap());

        let ffi_probe = Box::new(FFIProbe::from(probe)) as Box<dyn AsyncHealthProbe>;
        match AssertUnwindSafe(hams).ready_remove(&ffi_probe) {
            true => Ok(1),
            false => Ok(0),
        }
        // drop ffi_probe will delete the probe from the heap
        // AND the alive_remove should have dropped the one from the list
    )
}

/// # Safety
/// Check the alive probe to see if it is still alive
/// TODO: This will require to store the runtime and block on teh thred while we execute on the async runtime
// #[no_mangle]
// pub unsafe extern "C" fn hams_alive_check(ptr: *mut Hams) -> i32 {
//     ffi_helpers::null_pointer_check!(ptr, -1);

//     let now = Instant::now();
//     catch_panic!(
//         let hams = unsafe {&mut *ptr};

//         if hams.alive.check(now).await.valid {
//             Ok(1)
//         } else {
//             Ok(0)
//         }
//     )
// }

/// Return a manual health probe
///   We must return a ManualHealthProbe so that we can call set/enable etc. Later we box it for poly use
/// # Safety
/// Create a manual health probe
#[no_mangle]
pub unsafe extern "C" fn probe_manual_new(name: *const libc::c_char, check: bool) -> *mut Manual {
    ffi_helpers::null_pointer_check!(name);

    let name_str = unsafe { CStr::from_ptr(name).to_str().map_err(HamsError::from) };

    match name_str {
        Ok(name_str) => {
            info!("Creating ManualHealthProbe: {}", name_str);
            Box::into_raw(Box::new(Manual::new(name_str, check)))
        }
        Err(e) => {
            error!("Failed to create ManualHealthProbe");
            (e.into_ffi_enum_with_update() as u32) as *mut Manual
        }
    }
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

        info!("Releasing manual probe: {}", CString::from_raw(probe.name()).into_string().unwrap());
        drop(probe);
        Ok(1)
    )
}

/// Return a boxed health probe from the manual health probe
/// # Safety
/// Return a boxed health probe from the manual health probe
#[no_mangle]
// pub unsafe extern "C" fn probe_manual_boxed(ptr: *mut Manual) -> *mut () {
pub unsafe extern "C" fn probe_manual_boxed(ptr: *mut Manual) -> *mut BoxedHealthProbe<'static> {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let x = {
            let probe =  (*ptr).clone();
            let boxed_probe = BoxedHealthProbe::new(probe);


            // Use into_raw to pass ownership to the caller
            boxed_probe.into_raw() as *mut BoxedHealthProbe<'static>
        };

        Ok(x)
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

        info!("Releasing probe: {}", CString::from_raw(probe.name()).into_string().unwrap());
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

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    catch_panic!(
        let probe = &mut *ptr;

        Ok(probe.check(now.try_into()?) as i32)
    )
}

/// Return a kick health probe
///
/// # Safety
/// Create a kick health probe
#[no_mangle]
pub unsafe extern "C" fn probe_kick_new(
    name: *const libc::c_char,
    margin_secs: c_int,
) -> *mut Kick {
    ffi_helpers::null_pointer_check!(name);

    catch_panic!(
        let name_str = unsafe {CStr::from_ptr(name) }.to_str().unwrap();
        let margin = std::time::Duration::from_secs(margin_secs as u64);
        info!("Creating KickHealthProbe: {}", name_str);

        let probe = probe::kick::Kick::new(name_str, margin);
        Ok(Box::into_raw(Box::new(probe)))
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

        // let name = &probe.name();

        // let name = CString::from_raw(probe.name());
        info!("Releasing kick probe: {}", CString::from_raw(probe.name()).into_string().unwrap());
        drop(probe);
        Ok(1)
    )
}

/// Call kick method
///
/// # Safety
/// Call the kick method on the Kick object
#[no_mangle]
pub unsafe extern "C" fn probe_kick_kick(ptr: *mut Kick) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);

    catch_panic!(
        let probe = &mut *ptr;
        probe.kick();
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
        let probe = (*ptr).clone();
        let boxed_probe = BoxedHealthProbe::new(probe);

        Ok(boxed_probe.into_raw() as *mut BoxedHealthProbe<'static>)
    )
}

/// Test the FFI interfaces
#[cfg(test)]
mod tests {

    use std::{ptr, thread, time::Duration};

    use ffi_log2::log_param;

    use crate::error::ffi_error_to_result;

    use super::*;

    #[ignore]
    #[test]
    fn logger_init() {
        let retval = hams_logger_init(log_param());

        assert_ne!(retval, 0);
    }

    /// Test the register_prometheus function
    /// This will register the prometheus callback with the HaMS object
    #[test]
    fn register_prometheus() {
        let c_library_name = std::ffi::CString::new("name").unwrap();
        let c_address = std::ffi::CString::new("0.0.0.0:8079").unwrap();

        let my_hams = unsafe { hams_new(c_library_name.as_ptr(), c_address.as_ptr()) };
        assert_ne!(my_hams, ptr::null_mut());

        println!("initialised HaMS");

        // Define the callback function
        extern "C" fn prometheus_callback(ptr: *const c_void) -> *mut libc::c_char {
            let state = unsafe { &*(ptr as *const String) };

            let prometheus = format!("test {state}");
            let c_str_prometheus = std::ffi::CString::new(prometheus).unwrap();
            c_str_prometheus.into_raw()
        }

        // Define the callback function to free the allocated memory
        extern "C" fn prometheus_callback_free(ptr: *mut libc::c_char) {
            unsafe {
                if !ptr.is_null() {
                    drop(std::ffi::CString::from_raw(ptr));
                }
            }
        }

        let result = unsafe {
            hams_register_prometheus(
                my_hams,
                prometheus_callback,
                prometheus_callback_free,
                ptr::null(),
            )
        };
        assert_eq!(result, 1);

        let result = unsafe { hams_deregister_prometheus(my_hams) };
        assert_eq!(result, 1);

        let retval = unsafe { hams_free(my_hams) };

        assert_eq!(retval, 1);
    }

    #[test]
    fn hams_init_free() {
        let c_library_name = std::ffi::CString::new("name").unwrap();
        let c_address = std::ffi::CString::new("0.0.0.0:8079").unwrap();

        let my_hams = unsafe { hams_new(c_library_name.as_ptr(), c_address.as_ptr()) };

        assert_ne!(my_hams, ptr::null_mut());

        println!("initialised HaMS");

        let retval = unsafe { hams_free(my_hams) };

        assert_eq!(retval, 1);
    }

    #[test]
    fn null_init_name() {
        let c_address = std::ffi::CString::new("0.0.0.0:8079").unwrap();

        let my_hams = unsafe { hams_new(ptr::null(), c_address.as_ptr()) };

        assert_eq!(my_hams, ptr::null_mut());

        assert!(ffi_error_to_result().is_err(), "Error should be returned");

        assert_eq!(
            ffi_error_to_result().err().unwrap().to_string(),
            "FFI Error: A null pointer was passed in where it wasn't expected"
        );
    }

    #[test]
    fn null_init_address() {
        let c_name = std::ffi::CString::new("name").unwrap();

        let my_hams = unsafe { hams_new(c_name.as_ptr(), ptr::null()) };

        assert_eq!(my_hams, ptr::null_mut());

        assert!(ffi_error_to_result().is_err(), "Error should be returned");

        assert_eq!(
            ffi_error_to_result().err().unwrap().to_string(),
            "FFI Error: A null pointer was passed in where it wasn't expected"
        );
    }

    #[test]
    fn invalid_init_address() {
        let c_name = std::ffi::CString::new("name").unwrap();
        let c_address = std::ffi::CString::new("noaddress:noport").unwrap();

        let my_hams = unsafe { hams_new(c_name.as_ptr(), c_address.as_ptr()) };

        assert_eq!(my_hams, ptr::null_mut());

        assert!(ffi_error_to_result().is_err(), "Error should be returned");

        assert_eq!(
            ffi_error_to_result().err().unwrap().to_string(),
            "FFI Error: invalid socket address syntax"
        );
    }

    #[test]
    fn null_free() {
        let retval = unsafe { hams_free(ptr::null_mut()) };

        assert_eq!(retval, 0);

        assert!(ffi_error_to_result().is_err(), "Error should be returned");
        assert_eq!(
            ffi_error_to_result().err().unwrap().to_string(),
            "FFI Error: A null pointer was passed in where it wasn't expected"
        );
    }

    // create and free manual probe
    #[test]
    fn probe_manual_create_free() {
        let my_probe = unsafe { probe_manual_new(ptr::null(), true) };
        assert_eq!(my_probe, ptr::null_mut());
        assert!(ffi_error_to_result().is_err(), "Error should be returned");
        assert_eq!(
            ffi_error_to_result().err().unwrap().to_string(),
            "FFI Error: A null pointer was passed in where it wasn't expected"
        );

        let c_probe_name = std::ffi::CString::new("name").unwrap();

        let my_probe = unsafe { probe_manual_new(c_probe_name.as_ptr(), true) };

        assert_ne!(my_probe, ptr::null_mut());

        println!("initialised Manual Probe");

        let retval = unsafe { probe_manual_free(my_probe) };

        assert_eq!(retval, 1);

        let retval = unsafe { probe_free(ptr::null_mut()) };
        assert_eq!(retval, 0);
        assert_eq!(
            ffi_error_to_result().err().unwrap().to_string(),
            "FFI Error: A null pointer was passed in where it wasn't expected"
        );
    }

    // Create and free kick probe
    #[test]
    fn probe_kick_create_free() {
        let my_probe = unsafe { probe_kick_new(ptr::null(), 10) };
        assert_eq!(my_probe, ptr::null_mut());
        assert!(ffi_error_to_result().is_err(), "Error should be returned");
        assert_eq!(
            ffi_error_to_result().err().unwrap().to_string(),
            "FFI Error: A null pointer was passed in where it wasn't expected"
        );

        let c_probe_name = std::ffi::CString::new("name").unwrap();

        let my_probe = unsafe { probe_kick_new(c_probe_name.as_ptr(), 10) };

        assert_ne!(my_probe, ptr::null_mut());

        println!("initialised Kick Probe");

        let retval = unsafe { probe_kick_free(my_probe) };

        assert_eq!(retval, 1);

        let retval = unsafe { probe_kick_free(ptr::null_mut()) };
        assert_eq!(retval, 0);
        assert_eq!(
            ffi_error_to_result().err().unwrap().to_string(),
            "FFI Error: A null pointer was passed in where it wasn't expected"
        );
    }

    // Create Hams and insert + remove manual probe
    #[test]
    fn ffi_hams_start_stop() {
        let c_library_name = std::ffi::CString::new("name").unwrap();
        let c_address = std::ffi::CString::new("0.0.0.0:8077").unwrap();

        let my_hams = unsafe { hams_new(c_library_name.as_ptr(), c_address.as_ptr()) };

        assert_ne!(my_hams, ptr::null_mut());

        println!("initialised HaMS");

        let retval = unsafe { hams_start(my_hams) };
        thread::sleep(Duration::from_secs(1));

        assert_eq!(retval, 1);

        let retval = unsafe { hams_stop(my_hams) };

        assert_eq!(retval, FFIEnum::Success as i32);

        let retval = unsafe { hams_stop(my_hams) };
        assert_eq!(retval, FFIEnum::NotRunning as i32);
        assert_eq!(
            ffi_error_to_result().err().unwrap().to_string(),
            "FFI Error: Service is not running"
        );

        let retval = unsafe { hams_free(my_hams) };

        assert_eq!(retval, 1);
    }

    // Test when we start HaMS and the port is already in use
    #[test]
    fn ffi_hams_start_port_in_use() {
        let c_library_name = std::ffi::CString::new("name").unwrap();
        let c_address = std::ffi::CString::new("0.0.0.0:8078").unwrap();

        let my_hams = unsafe { hams_new(c_library_name.as_ptr(), c_address.as_ptr()) };

        assert_ne!(my_hams, ptr::null_mut());

        println!("initialised HaMS");

        let retval = unsafe { hams_start(my_hams) };
        thread::sleep(Duration::from_secs(1));

        assert_eq!(retval, 1);

        let my_hams2 = unsafe { hams_new(c_library_name.as_ptr(), c_address.as_ptr()) };

        assert_ne!(my_hams2, ptr::null_mut());

        let retval = unsafe { hams_start(my_hams) };
        assert_ne!(retval, 1);

        assert!(ffi_error_to_result().is_err(), "Error should be returned");
        assert_eq!(
            ffi_error_to_result().err().unwrap().to_string(),
            "FFI Error: Panic: Start HaMS: AlreadyRunning"
        );
        let retval = unsafe { hams_free(my_hams2) };
        assert_eq!(retval, 1);

        let retval = unsafe { hams_stop(my_hams) };
        assert_eq!(retval, 1);

        let retval = unsafe { hams_free(my_hams) };

        assert_eq!(retval, 1);
    }

    // Test insert remove of manual probe into hams
    #[test]
    fn hams_insert_remove_manual() {
        let c_library_name = std::ffi::CString::new("name").unwrap();
        let c_address = std::ffi::CString::new("0.0.0.0:8079").unwrap();

        let my_hams = unsafe { hams_new(c_library_name.as_ptr(), c_address.as_ptr()) };
        assert_ne!(my_hams, ptr::null_mut());

        println!("initialised HaMS");

        let c_probe_name = std::ffi::CString::new("name").unwrap();

        let my_probe = unsafe { probe_manual_new(c_probe_name.as_ptr(), true) };
        assert_ne!(my_probe, ptr::null_mut());

        // let check_response = unsafe { hams_alive_check(my_hams) };
        // assert_eq!(check_response, 1);

        let probe_boxed = unsafe { probe_manual_boxed(my_probe) };
        assert_ne!(probe_boxed, ptr::null_mut());

        let retval = unsafe { hams_alive_insert(my_hams, probe_boxed) };
        assert_eq!(retval, 1);

        // let check_response = unsafe { hams_alive_check(my_hams) };
        // assert_eq!(check_response, 1);

        let probe_boxed = unsafe { probe_manual_boxed(my_probe) };
        assert_ne!(probe_boxed, ptr::null_mut());

        let retval = unsafe { hams_alive_remove(my_hams, probe_boxed) };
        assert_eq!(retval, 1);

        // let check_response = unsafe { hams_alive_check(my_hams) };
        // assert_eq!(check_response, 1);

        let retval = unsafe { probe_manual_free(my_probe) };
        assert_eq!(retval, 1);

        let retval = unsafe { hams_free(my_hams) };
        assert_eq!(retval, 1);
    }

    // Test of register deregister shutdown with state
    #[test]
    fn hams_register_deregister_shutdown() {
        let c_library_name = std::ffi::CString::new("name").unwrap();
        let c_address = std::ffi::CString::new("0.0.0.0:8076").unwrap();

        let state = 0;
        extern "C" fn shutdown_callback(state: *mut c_void) {
            let state = unsafe { &mut *(state as *mut i32) };
            *state += 1;
        }

        let my_hams = unsafe { hams_new(c_library_name.as_ptr(), c_address.as_ptr()) };

        let retval = unsafe {
            hams_register_shutdown(
                my_hams,
                shutdown_callback,
                &state as *const i32 as *mut c_void,
            )
        };
        assert_eq!(retval, FFIEnum::Success as i32);

        unsafe { hams_start(my_hams) };

        assert!(state == 0);

        unsafe { hams_stop(my_hams) };
        assert!(state == 1);

        let retval = unsafe { hams_deregister_shutdown(my_hams) };
        assert_eq!(retval, FFIEnum::Success as i32);
    }
}
