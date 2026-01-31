use libc::{c_char, c_int, time_t};
use std::ffi::{CStr, CString};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::error::HamsError;

use super::{BoxedHealthProbe, HealthProbe};

#[derive(Debug, Clone)]
pub struct TcpProbe {
    name: String,
    c_name: CString,
    addr: String,
    timeout: Duration,
}

impl TcpProbe {
    pub fn new<S: Into<String>>(name: S, addr: &str, timeout_ms: u64) -> Result<Self, HamsError> {
        let name_str = name.into();
        let c_name = CString::new(name_str.clone()).map_err(|e| HamsError::NulError(e))?;

        // Validate address format
        if addr.to_socket_addrs().is_err() {
            return Err(HamsError::Message(format!("Invalid address: {}", addr)));
        }

        Ok(Self {
            name: name_str,
            c_name,
            addr: addr.to_string(),
            timeout: Duration::from_millis(timeout_ms),
        })
    }
}

impl HealthProbe for TcpProbe {
    fn name(&self) -> *const c_char {
        self.c_name.as_ptr()
    }

    fn check(&self, _time: time_t) -> c_int {
        match self.addr.to_socket_addrs() {
            Ok(mut addrs) => {
                // Try to connect to any of the resolved addresses
                // We need to resolve again inside check because IPs might change
                for addr in addrs {
                    if TcpStream::connect_timeout(&addr, self.timeout).is_ok() {
                        return 1;
                    }
                }
                0
            }
            Err(_) => 0,
        }
    }
}

// FFI Exports
use ffi_helpers::catch_panic;
use log::info;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_tcp_new(
    name: *const c_char,
    addr: *const c_char,
    timeout_ms: u64,
) -> *mut TcpProbe {
    ffi_helpers::null_pointer_check!(name);
    ffi_helpers::null_pointer_check!(addr);

    catch_panic!(
        let name_str = unsafe { CStr::from_ptr(name).to_str().unwrap() };
        let addr_str = unsafe { CStr::from_ptr(addr).to_str().unwrap() };

        match TcpProbe::new(name_str, addr_str, timeout_ms) {
            Ok(probe) => Ok(Box::into_raw(Box::new(probe))),
            Err(e) => {
                log::error!("Failed to create TcpProbe: {}", e);
                Ok(std::ptr::null_mut())
            }
        }
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_tcp_free(ptr: *mut TcpProbe) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);
    catch_panic!(
        let probe = unsafe { Box::from_raw(ptr) };
        info!("Releasing TcpProbe: {}", unsafe { CStr::from_ptr(probe.name()) }.to_string_lossy());
        drop(probe);
        Ok(1)
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_tcp_boxed(ptr: *mut TcpProbe) -> *mut BoxedHealthProbe<'static> {
    ffi_helpers::null_pointer_check!(ptr);
    catch_panic!(
        let probe = unsafe { (*ptr).clone() };
        let boxed_probe = BoxedHealthProbe::new(probe);
        Ok(boxed_probe.into_raw() as *mut BoxedHealthProbe<'static>)
    )
}
