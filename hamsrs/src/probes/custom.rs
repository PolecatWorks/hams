use std::ffi::CString;
use std::ffi::c_char;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use libc::time_t;
use log::info;

use crate::ffi;
use crate::hamserror::HamsError;

use crate::ffi::ffitraits::{BoxedHealthProbe, HealthProbe};

use super::Probe;

/// A custom probe that allows implementing health check logic via internal state.
///
/// This probe maintains a boolean state (`valid`) which determines its health.
/// It can be used for custom health checks where the logic allows toggling the state
/// from within the Rust application.
#[derive(Clone)]
pub struct ProbeCustom {
    name: CString,
    valid: Arc<AtomicBool>,
}

impl HealthProbe for ProbeCustom {
    fn name(&self) -> *const c_char {
        self.name.as_ptr()
    }

    fn check(&self, _time: time_t) -> i32 {
        self.valid.load(Ordering::Relaxed) as i32
    }
}

impl Probe for ProbeCustom {
    fn boxed(&self) -> Result<ffi::BProbe, HamsError> {
        let probe = BoxedHealthProbe::new(self.clone());

        println!("ProbeCustom::boxed: {:?}", probe.name());
        let pname = unsafe { std::ffi::CStr::from_ptr(probe.name()) }.to_string_lossy();
        println!("THIS IS THE BOXED PROBE: {:?}", pname);

        Ok(probe)
    }
}

impl ProbeCustom {
    /// Construct a new custom probe
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the probe
    /// * `valid` - The initial state of the probe (true for healthy, false for unhealthy)
    pub fn new<S: Into<String>>(
        name: S,
        valid: bool,
    ) -> Result<ProbeCustom, crate::hamserror::HamsError>
    where
        S: std::fmt::Display,
    {
        info!("New CustomHealthProbe: {}", &name);
        let c_name = CString::new(name.into()).unwrap();
        Ok(Self {
            name: c_name,
            valid: Arc::new(AtomicBool::new(valid)),
            // inner: Arc::new(ProbeCustomInner::new(name, valid)?),
        })
    }

    /// Enable the probe (set status to healthy)
    pub fn enable(&mut self) -> Result<(), crate::hamserror::HamsError> {
        self.valid.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Disable the probe (set status to unhealthy)
    pub fn disable(&mut self) -> Result<(), crate::hamserror::HamsError> {
        self.valid.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Toggle the probe's status
    pub fn toggle(&mut self) -> Result<(), crate::hamserror::HamsError> {
        self.valid.fetch_xor(true, Ordering::Relaxed);
        Ok(())
    }

    /// Check the current status of the probe
    pub fn check(&self) -> Result<bool, crate::hamserror::HamsError> {
        Ok(self.valid.load(Ordering::Relaxed))
    }
}

#[cfg(test)]
mod tests {

    use tokio_util::sync::CancellationToken;

    use crate::{Hams, hams::config::HamsConfig};

    use super::*;

    #[test]
    fn test_custom_probe() {
        let mut probe = ProbeCustom::new("test", true).unwrap();
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(probe.name()) }
                .to_str()
                .unwrap(),
            "test"
        );
        assert!(probe.check().unwrap());
        probe.disable().unwrap();
        assert!(!probe.check().unwrap());
        probe.enable().unwrap();
        assert!(probe.check().unwrap());
        probe.toggle().unwrap();
        assert!(!probe.check().unwrap());
    }

    /// Insert custom probe into hams
    #[test]
    fn add_custom_probe_to_hams() {
        // TODO: This test is not working and leads memory

        println!("THIS IS THE TEST");
        let ct = CancellationToken::new();
        let hc = HamsConfig::default();
        let hams = Hams::new(ct.clone(), &hc).unwrap();
        let probe_custom = ProbeCustom::new("test", true).unwrap();

        // hams.alive_insert_boxed( probe_custom.clone().boxed()).unwrap();
        hams.alive_insert(probe_custom.clone()).unwrap();
        hams.alive_remove(&probe_custom).unwrap();
    }
}
