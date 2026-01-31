use std::sync::Arc;
use crate::{ffi::{self, ffitraits::BoxedHealthProbe}, hamserror::HamsError};
use log::info;
use super::Probe;

#[derive(Debug)]
pub struct ProbeDnsInner {
    pub c: *mut ffi::DnsProbe,
}

impl Drop for ProbeDnsInner {
    fn drop(&mut self) {
        unsafe { ffi::probe_dns_free(self.c) };
        info!("Dns Probe freed");
    }
}

impl ProbeDnsInner {
    pub fn new(name: &str, host: &str) -> Result<Self, HamsError> {
        let c_name = std::ffi::CString::new(name)?;
        let c_host = std::ffi::CString::new(host)?;

        let c = unsafe { ffi::probe_dns_new(c_name.as_ptr(), c_host.as_ptr()) };
        if c.is_null() {
            return Err(HamsError::Message("Failed to create Dns Probe".to_string()));
        }
        Ok(Self { c })
    }

    fn boxed(&self) -> Result<BoxedHealthProbe<'static>, HamsError> {
        let c = unsafe { ffi::probe_dns_boxed(self.c) };
        if c.is_null() {
            return Err(HamsError::Message("Failed to box Dns Probe".to_string()));
        }
        unsafe { Ok(BoxedHealthProbe::from_raw(c as *mut ())) }
    }
}

#[derive(Clone, Debug)]
pub struct ProbeDns {
    pub inner: Arc<ProbeDnsInner>,
}

impl ProbeDns {
    pub fn new(name: &str, host: &str) -> Result<Self, HamsError> {
        Ok(Self {
            inner: Arc::new(ProbeDnsInner::new(name, host)?),
        })
    }
}

impl Probe for ProbeDns {
    fn boxed(&self) -> Result<BoxedHealthProbe<'static>, HamsError> {
        self.inner.boxed()
    }
}
