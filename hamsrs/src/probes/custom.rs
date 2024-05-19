use std::time::Instant;

use log::info;

use crate::hamserror::HamsError;

use crate::ffi::ffitraits::{BoxedHealthProbe, HealthProbe};

use super::{BoxedProbe, Probe};

#[derive(Clone)]
pub struct ProbeCustom {
    name: String,
    valid: bool,
}

impl HealthProbe for ProbeCustom {
    fn name(&self) -> Result<String, HamsError> {
        Ok(self.name.clone())
    }

    fn check(&self, _time: Instant) -> Result<bool, HamsError> {
        Ok(self.valid)
    }
    fn ffi_boxed(&self) -> BoxedHealthProbe<'static> {
        BoxedHealthProbe::new(self.clone())
    }
}

impl Probe for ProbeCustom {
    fn boxed(&self) -> BoxedProbe {
        let boxed_probe = BoxedHealthProbe::new(self.clone());
        // From hams library this is the creation of the abc pointer
        let c = Box::into_raw(Box::new(boxed_probe));

        // Return a pointer to the probe inside the BoxedProbe struct
        // let c: *mut ffi::Probe = abc;
        BoxedProbe { c }
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
            valid,
        })
    }

    /// Enable the probe
    pub fn enable(&mut self) -> Result<(), crate::hamserror::HamsError> {
        self.valid = true;
        Ok(())
    }

    /// Disable the probe
    pub fn disable(&mut self) -> Result<(), crate::hamserror::HamsError> {
        self.valid = false;
        Ok(())
    }

    /// Toggle the probe
    pub fn toggle(&mut self) -> Result<(), crate::hamserror::HamsError> {
        self.valid = !self.valid;
        Ok(())
    }

    // Check the probe
    pub fn check(&self) -> Result<bool, crate::hamserror::HamsError> {
        Ok(self.valid)
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
    use super::*;

    #[test]
    fn test_custom_probe() {
        let mut probe = ProbeCustom::new("test", true).unwrap();
        assert_eq!(probe.name().unwrap(), "test");
        assert!(probe.check().unwrap());
        probe.disable().unwrap();
        assert!(!probe.check().unwrap());
        probe.enable().unwrap();
        assert!(probe.check().unwrap());
        probe.toggle().unwrap();
        assert!(!probe.check().unwrap());
    }
}
