use libc::{c_char, c_int, time_t};
use reqwest::StatusCode;
use reqwest::blocking::Client;
use std::ffi::{CStr, CString};
use std::time::Duration;
use url::Url;

use crate::error::HamsError;

use super::{BoxedHealthProbe, HealthProbe};

#[derive(Debug, Clone)]
pub struct HttpProbe {
    name: String,
    c_name: CString,
    url: Url,
    expected_codes: Vec<StatusCode>,
}

impl HttpProbe {
    pub fn new<S: Into<String>>(
        name: S,
        url: &str,
        expected_codes: Option<Vec<u16>>,
    ) -> Result<Self, HamsError> {
        let name_str = name.into();
        let c_name = CString::new(name_str.clone()).map_err(|e| HamsError::NulError(e))?;
        let url = Url::parse(url).map_err(|e| HamsError::Message(format!("Invalid URL: {}", e)))?;

        let codes = match expected_codes {
            Some(codes) => codes
                .iter()
                .filter_map(|&c| StatusCode::from_u16(c).ok())
                .collect(),
            None => vec![StatusCode::OK],
        };

        Ok(Self {
            name: name_str,
            c_name,
            url,
            expected_codes: codes,
        })
    }
}

impl HealthProbe for HttpProbe {
    fn name(&self) -> *const c_char {
        self.c_name.as_ptr()
    }

    fn check(&self, _time: time_t) -> c_int {
        let client = match Client::builder().timeout(Duration::from_secs(5)).build() {
            Ok(c) => c,
            Err(_) => return -1,
        };

        match client.get(self.url.clone()).send() {
            Ok(response) => {
                if self.expected_codes.contains(&response.status()) {
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
pub unsafe extern "C" fn probe_http_new(
    name: *const c_char,
    url: *const c_char,
    codes: *const u16,
    codes_len: usize,
) -> *mut HttpProbe {
    ffi_helpers::null_pointer_check!(name);
    ffi_helpers::null_pointer_check!(url);

    catch_panic!(
        let name_str = unsafe { CStr::from_ptr(name).to_str().unwrap() };
        let url_str = unsafe { CStr::from_ptr(url).to_str().unwrap() };

        let expected_codes = if !codes.is_null() && codes_len > 0 {
            Some(unsafe { std::slice::from_raw_parts(codes, codes_len).to_vec() })
        } else {
            None
        };

        match HttpProbe::new(name_str, url_str, expected_codes) {
            Ok(probe) => Ok(Box::into_raw(Box::new(probe))),
            Err(e) => {
                log::error!("Failed to create HttpProbe: {}", e);
                Ok(std::ptr::null_mut())
            }
        }
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_http_free(ptr: *mut HttpProbe) -> i32 {
    ffi_helpers::null_pointer_check!(ptr);
    catch_panic!(
        let probe = unsafe { Box::from_raw(ptr) };
        info!("Releasing HttpProbe: {}", unsafe { CStr::from_ptr(probe.name()) }.to_string_lossy());
        drop(probe);
        Ok(1)
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_http_boxed(ptr: *mut HttpProbe) -> *mut BoxedHealthProbe<'static> {
    ffi_helpers::null_pointer_check!(ptr);
    catch_panic!(
        let probe = unsafe { (*ptr).clone() };
        let boxed_probe = BoxedHealthProbe::new(probe);
        Ok(boxed_probe.into_raw() as *mut BoxedHealthProbe<'static>)
    )
}
