use libc::{c_char, c_int, time_t};
use std::ffi::{CStr, CString};
use std::net::ToSocketAddrs;

use super::{BoxedHealthProbe, HealthProbe};
use crate::error::HamsError;

#[derive(Debug, Clone)]
pub struct DnsProbe {
    name: String,
    c_name: CString,
    host: String,
}

impl DnsProbe {
    pub fn new<S: Into<String>>(name: S, host: &str) -> Result<Self, HamsError> {
        let name_str = name.into();
        let c_name = CString::new(name_str.clone()).map_err(|e| HamsError::NulError(e))?;

        Ok(Self {
            name: name_str,
            c_name,
            host: host.to_string(),
        })
    }
}

impl HealthProbe for DnsProbe {
    fn name(&self) -> *const c_char {
        self.c_name.as_ptr()
    }

    fn check(&self, _time: time_t) -> c_int {
        // Simple check: does it resolve?
        // We append port 80 because ToSocketAddrs expects host:port
        let addr_str = format!("{}:80", self.host);
        match addr_str.to_socket_addrs() {
            Ok(mut iter) => {
                if iter.next().is_some() {
                    1
                } else {
                    0
                }
            }
            Err(_) => 0,
        }
    }
}

// FFI Exports
use ffi_helpers::catch_panic;
use log::info;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_dns_new(name: *const c_char, host: *const c_char) -> *mut DnsProbe {
    ffi_helpers::null_pointer_check!(name);
    ffi_helpers::null_pointer_check!(host);

    catch_panic!(
        let name_str = unsafe { CStr::from_ptr(name).to_str().unwrap() };
        let host_str = unsafe { CStr::from_ptr(host).to_str().unwrap() };

        match DnsProbe::new(name_str, host_str) {
            Ok(probe) => Ok(Box::into_raw(Box::new(probe))),
            Err(e) => {
                log::error!("Failed to create DnsProbe: {}", e);
                Ok(std::ptr::null_mut())
            }
        }
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_dns_free(ptr: *mut DnsProbe) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);
    catch_panic!(
        let probe = unsafe { Box::from_raw(ptr) };
        info!("Releasing DnsProbe: {}", unsafe { CStr::from_ptr(probe.name()) }.to_string_lossy());
        drop(probe);
        Ok(1)
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_dns_boxed(ptr: *mut DnsProbe) -> *mut BoxedHealthProbe<'static> {
    ffi_helpers::null_pointer_check!(ptr);
    catch_panic!(
        let probe = unsafe { (*ptr).clone() };
        let boxed_probe = BoxedHealthProbe::new(probe);
        Ok(boxed_probe.into_raw() as *mut BoxedHealthProbe<'static>)
    )
}
