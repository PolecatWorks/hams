use super::Probe;
use crate::{
    ffi::{self, ffitraits::BoxedHealthProbe},
    hamserror::HamsError,
};
use log::info;
use std::sync::Arc;

#[derive(Debug)]
pub struct ProbeTcpInner {
    pub c: *mut ffi::TcpProbe,
}

unsafe impl Send for ProbeTcpInner {}
unsafe impl Sync for ProbeTcpInner {}

impl Drop for ProbeTcpInner {
    fn drop(&mut self) {
        unsafe { ffi::probe_tcp_free(self.c) };
        info!("Tcp Probe freed");
    }
}

impl ProbeTcpInner {
    pub fn new(name: &str, addr: &str, timeout_ms: u64) -> Result<Self, HamsError> {
        let c_name = std::ffi::CString::new(name)?;
        let c_addr = std::ffi::CString::new(addr)?;

        let c = unsafe { ffi::probe_tcp_new(c_name.as_ptr(), c_addr.as_ptr(), timeout_ms) };
        if c.is_null() {
            return Err(HamsError::Message("Failed to create Tcp Probe".to_string()));
        }
        Ok(Self { c })
    }

    fn boxed(&self) -> Result<BoxedHealthProbe<'static>, HamsError> {
        let c = unsafe { ffi::probe_tcp_boxed(self.c) };
        if c.is_null() {
            return Err(HamsError::Message("Failed to box Tcp Probe".to_string()));
        }
        unsafe { Ok(BoxedHealthProbe::from_raw(c as *mut ())) }
    }
}

/// A probe that attempts to connect to a TCP socket.
#[derive(Clone, Debug)]
pub struct ProbeTcp {
    pub inner: Arc<ProbeTcpInner>,
}

impl ProbeTcp {
    /// Construct a new TCP probe
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the probe
    /// * `addr` - The address to connect to (e.g., "127.0.0.1:8080" or "example.com:80")
    /// * `timeout_ms` - The connection timeout in milliseconds
    pub fn new(name: &str, addr: &str, timeout_ms: u64) -> Result<Self, HamsError> {
        Ok(Self {
            inner: Arc::new(ProbeTcpInner::new(name, addr, timeout_ms)?),
        })
    }
}

impl Probe for ProbeTcp {
    fn boxed(&self) -> Result<BoxedHealthProbe<'static>, HamsError> {
        self.inner.boxed()
    }
}
