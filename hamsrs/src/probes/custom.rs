use std::ffi::c_char;
use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use libc::time_t;
use log::info;

use crate::ffi;
use crate::hamserror::HamsError;

use crate::ffi::ffitraits::{BoxedHealthProbe, HealthProbe};

use super::Probe;

#[derive(Clone)]
pub struct ProbeCustom {
    name: String,
    valid: Arc<AtomicBool>,
}

impl HealthProbe for ProbeCustom {
    fn name(&self) -> *mut c_char {
        CString::new(self.name.clone()).unwrap().into_raw()
    }

    fn check(&self, _time: time_t) -> i32 {
        self.valid.load(Ordering::Relaxed) as i32
    }
}

impl Probe for ProbeCustom {
    fn boxed(&self) -> Result<ffi::BProbe, HamsError> {
        let probe = BoxedHealthProbe::new(self.clone());

        Ok(probe)
    }
}

impl ProbeCustom {
    /// Construct a new manual probe
    pub fn new<S: Into<String>>(
        name: S,
        valid: bool,
    ) -> Result<ProbeCustom, crate::hamserror::HamsError>
    where
        S: std::fmt::Display,
    {
        info!("New CustomHealthProbe: {}", &name);

        Ok(ProbeCustom {
            name: name.to_string(),
            valid: AtomicBool::new(valid).into(),
        })
    }

    /// Enable the probe
    pub fn enable(&mut self) -> Result<(), crate::hamserror::HamsError> {
        self.valid.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Disable the probe
    pub fn disable(&mut self) -> Result<(), crate::hamserror::HamsError> {
        self.valid.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Toggle the probe
    pub fn toggle(&mut self) -> Result<(), crate::hamserror::HamsError> {
        self.valid.fetch_xor(true, Ordering::Relaxed);

        Ok(())
    }

    // Check the probe
    pub fn check(&self) -> Result<bool, crate::hamserror::HamsError> {
        Ok(self.valid.load(Ordering::Relaxed))
    }
}

impl Drop for ProbeCustom {
    /// Releaes the HaMS ffi on drop
    fn drop(&mut self) {
        info!("Custom Probe freed")
    }
}

#[cfg(test)]
mod tests {
    use crate::ProbeManual;

    use super::*;

    #[test]
    fn test_custom_probe() {
        let mut probe = ProbeCustom::new("test", true).unwrap();
        assert_eq!(
            unsafe { CString::from_raw(probe.name()) }
                .into_string()
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
        let hams = crate::hams::Hams::new("my hams").unwrap();
        let probe_custom = ProbeCustom::new("test", true).unwrap();

        // hams.alive_insert_boxed( probe_custom.clone().boxed()).unwrap();
        hams.alive_insert(probe_custom.clone()).unwrap();
        hams.alive_remove(&probe_custom).unwrap();
    }
}
