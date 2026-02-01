use super::Probe;
use crate::{
    ffi::{self, ffitraits::BoxedHealthProbe},
    hamserror::HamsError,
};
use log::info;
use std::sync::Arc;

#[derive(Debug)]
pub struct ProbeDnsInner {
    pub c: *mut ffi::DnsProbe,
}

unsafe impl Send for ProbeDnsInner {}
unsafe impl Sync for ProbeDnsInner {}

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

/// A probe that checks DNS resolution for a given hostname.
#[derive(Clone, Debug)]
pub struct ProbeDns {
    pub inner: Arc<ProbeDnsInner>,
}

impl ProbeDns {
    /// Construct a new DNS probe
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the probe
    /// * `host` - The hostname to resolve
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
