use std::sync::Arc;

use crate::{ffi::ffitraits::BoxedHealthProbe, hamserror::HamsError};
use log::info;

use crate::ffi;

use super::Probe;

/// Inner struct for the Manual Probe to manage lifecycle against the FFI

#[derive(Debug)]
pub struct ProbeManualInner {
    pub c: *mut ffi::ManualProbe,
}

impl Drop for ProbeManualInner {
    /// Releaes the HaMS ffi on drop
    fn drop(&mut self) {
        let retval = unsafe { ffi::probe_manual_free(self.c) };
        if retval == 0 {
            panic!("FAILED to free Probe");
        }

        info!("Manual Probe freed")
    }
}

impl ProbeManualInner {
    pub fn new<S>(name: S, valid: bool) -> Result<ProbeManualInner, crate::hamserror::HamsError>
    where
        S: std::fmt::Display + Into<String>,
    {
        info!("New ManualHealthProbe: {}", &name);

        let c_name = std::ffi::CString::new(name.into())?;
        let c = unsafe { ffi::probe_manual_new(c_name.as_ptr(), valid) };

        if c.is_null() {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to create Probe object".to_string(),
            ));
        }
        Ok(ProbeManualInner { c: c.into() })
    }

    /// Enable the probe
    pub fn enable(&self) -> Result<(), crate::hamserror::HamsError> {
        let retval = unsafe { ffi::probe_manual_enable(self.c, true) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to enable Probe".to_string(),
            ));
        }
        Ok(())
    }
    /// Disable the probe
    pub fn disable(&self) -> Result<(), crate::hamserror::HamsError> {
        let retval = unsafe { ffi::probe_manual_disable(self.c) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to disable Probe".to_string(),
            ));
        }
        Ok(())
    }

    /// Toggle the probe
    pub fn toggle(&self) -> Result<(), crate::hamserror::HamsError> {
        let retval = unsafe { ffi::probe_manual_toggle(self.c) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to toggle Probe".to_string(),
            ));
        }
        Ok(())
    }
    // Check the probe
    pub fn check(&self) -> Result<bool, crate::hamserror::HamsError> {
        let retval = unsafe { ffi::probe_manual_check(self.c) };
        if retval == -1 {
            // TODO: Retrieve the actual error from FFI and return it using: https://docs.rs/ffi_helpers/0.3.0/ffi_helpers/error_handling/index.html
            return Err(crate::hamserror::HamsError::Message(
                "Failed to check Probe".to_string(),
            ));
        }
        Ok(retval == 1)
    }

    fn boxed(&self) -> Result<ffi::BProbe, HamsError> {
        let c = unsafe { ffi::probe_manual_boxed(self.c) };

        if c.is_null() {
            // panic!("PUT GOOD ERROR HERE");
            return Err(HamsError::Message("Could not box probe".to_string()));
        }

        let probe = unsafe { BoxedHealthProbe::from_raw(c as *mut ()) };

        Ok(probe)
    }
}

#[derive(Clone, Debug)]
pub struct ProbeManual {
    pub inner: Arc<ProbeManualInner>,
}

impl Probe for ProbeManual {
    fn boxed(&self) -> Result<ffi::BProbe, HamsError> {
        self.inner.boxed()
    }
}

impl ProbeManual {
    /// Construct a new manual probe
    pub fn new<S: Into<String>>(
        name: S,
        valid: bool,
    ) -> Result<ProbeManual, crate::hamserror::HamsError>
    where
        S: std::fmt::Display,
    {
        Ok(ProbeManual {
            inner: Arc::new(ProbeManualInner::new(name, valid)?),
        })
    }

    /// Enable the probe
    pub fn enable(&self) -> Result<(), crate::hamserror::HamsError> {
        self.inner.enable()
    }

    /// Disable the probe
    pub fn disable(&self) -> Result<(), crate::hamserror::HamsError> {
        self.inner.disable()
    }

    /// Toggle the probe
    pub fn toggle(&self) -> Result<(), crate::hamserror::HamsError> {
        self.inner.toggle()
    }

    // Check the probe
    pub fn check(&self) -> Result<bool, crate::hamserror::HamsError> {
        self.inner.check()
    }
}

#[cfg(test)]
mod tests {

    use tokio_util::sync::CancellationToken;

    use crate::hams::config::HamsConfig;

    use super::*;

    #[test]
    fn test_manual_probe() {
        let probe_manual = ProbeManual::new("test_probe", true).unwrap();

        assert!(probe_manual.check().unwrap());
        probe_manual.disable().unwrap();
        assert!(!probe_manual.check().unwrap());

        probe_manual.toggle().unwrap();
        assert!(probe_manual.check().unwrap());
        probe_manual.toggle().unwrap();
        assert!(!probe_manual.check().unwrap());

        drop(probe_manual);
    }

    /// Add Manual Probe to Hams
    #[test]
    fn add_manual_probe_to_hams() {
        let hams = crate::hams::Hams::new(CancellationToken::new(), HamsConfig::default()).unwrap();
        let probe_manual = ProbeManual::new("test_probe", true).unwrap();

        println!("Probe: {:?}", probe_manual);
        let _p2 = probe_manual.clone();

        println!("Probe: {:?}", probe_manual);

        hams.alive_insert(probe_manual.clone()).unwrap();
        println!("Probe added to hams: {:?}", probe_manual);
        hams.alive_remove(&probe_manual).unwrap();
    }
}
