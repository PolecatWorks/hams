use log::info;
use std::marker::PhantomData;

use crate::ffi;

use super::{BoxedProbe, Probe};

pub struct ProbeManual {
    pub c: *mut ffi::ManualProbe,
}

impl Probe for ProbeManual {
    fn boxed(&self) -> BoxedProbe {
        let c = unsafe { ffi::probe_manual_boxed(self.c) };

        BoxedProbe { c }
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
        info!("New ManualHealthProbe: {}", &name);
        let c_name = std::ffi::CString::new(name.into())?;
        let c = unsafe { ffi::probe_manual_new(c_name.as_ptr(), valid) };
        if c.is_null() {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to create Probe object".to_string(),
            ));
        }
        Ok(ProbeManual { c })
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
}

impl<'a> Drop for ProbeManual {
    /// Releaes the HaMS ffi on drop
    fn drop(&mut self) {
        let retval = unsafe { ffi::probe_manual_free(self.c) };
        if retval == 0 {
            panic!("FAILED to free Probe");
        }

        info!("Probe freed")
    }
}

#[cfg(test)]
mod tests {
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
}
