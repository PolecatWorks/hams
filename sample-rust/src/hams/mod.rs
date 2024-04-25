use log::info;

use crate::{ffi, probes::BoxedProbe};

/// Hams is an FFI struct to opaquely handle the object that was created by the Hams API.
///
/// This allows the developer to use the Hams C API using safe rust.
/// Following this example of wrapping the pointer: https://medium.com/dwelo-r-d/wrapping-unsafe-c-libraries-in-rust-d75aeb283c65
pub struct Hams {
    // This pointer must never be allowed to leave the struct
    c: *mut ffi::Hams,
}

impl Hams {
    /// Construct the new Hams.
    /// The return of thi call will have created an object via FFI to handle and manage
    /// your alive and readyness checks.
    /// It also manages your monitoring via prometheus exports
    pub fn new<S: Into<String>>(name: S) -> Result<Hams, crate::hamserror::HamsError>
    where
        S: std::fmt::Display,
    {
        info!("Registering HaMS: {}", &name);
        let c_name = std::ffi::CString::new(name.into())?;
        let c = unsafe { ffi::hams_new(c_name.as_ptr()) };
        if c.is_null() {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to create Hams object".to_string(),
            ));
        }
        Ok(Hams { c })
    }

    /// Start the HaMS
    ///
    /// This will start the HaMS and begin serving the readyness and liveness checks
    /// as well as the prometheus metrics
    ///
    pub fn start(&self) -> Result<(), crate::hamserror::HamsError> {
        let retval = unsafe { ffi::hams_start(self.c) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to start HaMS".to_string(),
            ));
        }
        Ok(())
    }

    /// Stop the HaMS
    ///
    /// This will stop the HaMS and stop serving the readyness and liveness checks
    /// as well as the prometheus metrics
    pub fn stop(&self) -> Result<(), crate::hamserror::HamsError> {
        let retval = unsafe { ffi::hams_stop(self.c) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to stop HaMS".to_string(),
            ));
        }
        Ok(())
    }

    /// Insert a probe into the alive checks
    ///
    /// This will insert a probe into the alive checks
    pub fn alive_insert(&self, probe: BoxedProbe) -> Result<(), crate::hamserror::HamsError> {
        // let probe_c = probe.c;
        let retval = unsafe { ffi::hams_alive_insert(self.c, probe.c()) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to insert probe into alive checks".to_string(),
            ));
        }
        Ok(())
    }
}

/// This trait automatically handles the deallocation of the hams api when the Hams object
/// goes out of scope
impl Drop for Hams {
    /// Releaes the HaMS ffi on drop
    fn drop(&mut self) {
        let retval = unsafe { ffi::hams_free(self.c) };
        if retval == 0 {
            panic!("FAILED to free HaMS");
        }

        info!("HaMS freed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hams_startstop() {
        let hams = Hams::new("test").unwrap();
        hams.start().unwrap();
        hams.stop().unwrap();
    }

    #[test]
    fn add_probes_to_hams() {
        let hams = Hams::new("test").unwrap();
        let probe_manual = crate::probes::ProbeManual::new("test_probe", true).unwrap();

        // todo!("Add the probe to the hams");
        hams.alive_insert(probe_manual.boxed()).unwrap();

        // hams.start().unwrap();
        // hams.stop().unwrap();}
    }
}
