use std::{sync::Arc, time::Duration};

use log::{error, info};

use super::Probe;

use crate::ffi::ffitraits::BoxedHealthProbe;
use crate::{ffi, hamserror::HamsError};

#[derive(Debug)]
pub struct ProbeKickInner {
    pub c: *mut ffi::KickProbe,
}

impl Drop for ProbeKickInner {
    fn drop(&mut self) {
        let retval = unsafe { ffi::probe_kick_free(self.c) };

        if retval == 0 {
            error!("Failed to free Probe object");
        }

        info!("Kick Probe freed")
    }
}

impl ProbeKickInner {
    pub fn new<S: Into<String>>(name: S, margin: Duration) -> Result<ProbeKickInner, HamsError>
    where
        S: std::fmt::Display,
    {
        info!("New KickHealthProbe: {}", &name);
        let c_name = std::ffi::CString::new(name.into())?;
        let c = unsafe { ffi::probe_kick_new(c_name.as_ptr(), margin.as_millis().try_into()?) };

        if c.is_null() {
            return Err(HamsError::Message(
                "Failed to create Probe object".to_string(),
            ));
        }
        Ok(ProbeKickInner { c: c.into() })
    }

    pub fn kick(&self) -> Result<(), HamsError> {
        let retval = unsafe { ffi::probe_kick_kick(self.c) };

        if retval == 0 {
            return Err(HamsError::Message("Failed to kick Probe".to_string()));
        }
        Ok(())
    }

    fn boxed(&self) -> Result<ffi::BProbe, HamsError> {
        let c = unsafe { ffi::probe_kick_boxed(self.c) };

        if c.is_null() {
            // panic!("PUT GOOD ERROR HERE");
            return Err(HamsError::Message("Could not box probe".to_string()));
        }
        let probe = unsafe { BoxedHealthProbe::from_raw(c as *mut ()) };

        Ok(probe)
    }
}

#[derive(Clone, Debug)]
pub struct ProbeKick {
    inner: Arc<ProbeKickInner>,
}

impl Probe for ProbeKick {
    fn boxed(&self) -> Result<ffi::BProbe, HamsError> {
        self.inner.boxed()
    }
}

impl ProbeKick {
    /// Construct a new kick probe
    pub fn new<S: Into<String>>(
        name: S,
        margin: Duration,
    ) -> Result<ProbeKick, crate::hamserror::HamsError>
    where
        S: std::fmt::Display,
    {
        Ok(ProbeKick {
            inner: Arc::new(ProbeKickInner::new(name, margin)?),
        })
    }

    /// Kick the probe
    pub fn kick(&self) -> Result<(), crate::hamserror::HamsError> {
        self.inner.kick()
    }
}

#[cfg(test)]
mod tests {

    use crate::hams::config::HamsConfig;

    use super::*;

    #[test]
    fn test_probe_kick() {
        let probe = ProbeKick::new("test_probe_kick", Duration::from_secs(1)).unwrap();
        probe.kick().unwrap();

        drop(probe);
    }

    #[test]
    fn test_probe_kick_fail() {
        let probe = ProbeKick::new("test_probe_kick_fail", Duration::from_secs(1)).unwrap();
        probe.kick().unwrap();

        drop(probe);
    }

    /// Add Kick Probe to Hams
    #[test]
    fn add_kick_probe_to_hams() {
        let hams = crate::hams::Hams::new(HamsConfig::default()).unwrap();
        let probe_kick = ProbeKick::new("test", Duration::from_secs(1)).unwrap();

        println!("Probe: {:?}", probe_kick);
        let _p2 = probe_kick.clone();

        println!("Probe: {:?}", probe_kick);

        hams.alive_insert(probe_kick.clone()).unwrap();
        println!("Probe added to hams: {:?}", probe_kick);
        hams.alive_remove(&probe_kick).unwrap();
    }
}
