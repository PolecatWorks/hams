use std::time::Duration;

use log::{error, info};

use super::{BoxedProbe, Probe};

use crate::ffi;

pub struct ProbeKick {
    c: *mut ffi::KickProbe,
}

impl Probe for ProbeKick {
    fn boxed(&self) -> BoxedProbe {
        let c = unsafe { ffi::probe_kick_boxed(self.c) };

        BoxedProbe { c }
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
        info!("New KickHealthProbe: {}", &name);
        let c_name = std::ffi::CString::new(name.into())?;
        let c = unsafe { ffi::probe_kick_new(c_name.as_ptr(), margin.as_secs()) };
        if c.is_null() {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to create Probe object".to_string(),
            ));
        }
        Ok(ProbeKick { c })
    }

    /// Kick the probe
    pub fn kick(&self) -> Result<(), crate::hamserror::HamsError> {
        let retval = unsafe { ffi::probe_kick_kick(self.c) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to kick Probe".to_string(),
            ));
        }
        Ok(())
    }
}

impl Drop for ProbeKick {
    fn drop(&mut self) {
        let retval = unsafe { ffi::probe_kick_free(self.c) };

        if retval == 0 {
            error!("Failed to free Probe object");
        }

        info!("Kick Probe freed")
    }
}

#[cfg(test)]
mod tests {
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
}
