use std::sync::Arc;
use crate::{ffi::{self, ffitraits::BoxedHealthProbe}, hamserror::HamsError};
use log::info;
use super::Probe;

#[derive(Debug)]
pub struct ProbeHttpInner {
    pub c: *mut ffi::HttpProbe,
}

impl Drop for ProbeHttpInner {
    fn drop(&mut self) {
        unsafe { ffi::probe_http_free(self.c) };
        info!("Http Probe freed");
    }
}

impl ProbeHttpInner {
    pub fn new(name: &str, url: &str, codes: Option<&[u16]>) -> Result<Self, HamsError> {
        let c_name = std::ffi::CString::new(name)?;
        let c_url = std::ffi::CString::new(url)?;

        let (codes_ptr, codes_len) = match codes {
            Some(c) => (c.as_ptr(), c.len()),
            None => (std::ptr::null(), 0),
        };

        let c = unsafe {
            ffi::probe_http_new(c_name.as_ptr(), c_url.as_ptr(), codes_ptr, codes_len)
        };
        if c.is_null() {
            return Err(HamsError::Message("Failed to create Http Probe".to_string()));
        }
        Ok(Self { c })
    }

    fn boxed(&self) -> Result<BoxedHealthProbe<'static>, HamsError> {
        let c = unsafe { ffi::probe_http_boxed(self.c) };
        if c.is_null() {
            return Err(HamsError::Message("Failed to box Http Probe".to_string()));
        }
        unsafe { Ok(BoxedHealthProbe::from_raw(c as *mut ())) }
    }
}

#[derive(Clone, Debug)]
pub struct ProbeHttp {
    pub inner: Arc<ProbeHttpInner>,
}

impl ProbeHttp {
    pub fn new(name: &str, url: &str, codes: Option<&[u16]>) -> Result<Self, HamsError> {
        Ok(Self {
            inner: Arc::new(ProbeHttpInner::new(name, url, codes)?),
        })
    }
}

impl Probe for ProbeHttp {
    fn boxed(&self) -> Result<BoxedHealthProbe<'static>, HamsError> {
        self.inner.boxed()
    }
}
