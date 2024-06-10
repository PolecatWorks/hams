use libc::c_void;
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

    pub fn register_prometheus(
        &self,
        my_cb: extern "C" fn(state: *const c_void) -> *const libc::c_char,
        my_cb_free: extern "C" fn(*mut libc::c_char),
        state: *const c_void,
    ) -> Result<(), crate::hamserror::HamsError> {
        let retval = unsafe { ffi::hams_register_prometheus(self.c, my_cb, my_cb_free, state) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to register prometheus".to_string(),
            ));
        }
        Ok(())
    }

    /// De-register the prometheus
    /// This will stop the prometheus metrics from being served
    pub fn deregister_prometheus(&self) -> Result<(), crate::hamserror::HamsError> {
        let retval = unsafe { ffi::hams_deregister_prometheus(self.c) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to deregister prometheus".to_string(),
            ));
        }
        Ok(())
    }

    /// Insert a probe into the alive checks
    ///
    /// This will insert a probe into the alive checks
    pub fn alive_insert_boxed(&self, probe: BoxedProbe) -> Result<(), crate::hamserror::HamsError> {
        // let probe_c = probe.c;
        println!("ABOUT to insert probe");
        let retval = unsafe { ffi::hams_alive_insert(self.c, probe.c) };
        println!("Part way through ");
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to insert probe into alive checks".to_string(),
            ));
        }
        println!("did insert");
        Ok(())
    }

    pub fn alive_insert(
        &self,
        probe: &dyn crate::probes::Probe,
    ) -> Result<(), crate::hamserror::HamsError> {
        let probe_c = probe.boxed().c;
        let retval = unsafe { ffi::hams_alive_insert(self.c, probe_c) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to insert probe into alive checks".to_string(),
            ));
        }
        Ok(())
    }

    pub fn alive_remove(
        &self,
        probe: &dyn crate::probes::Probe,
    ) -> Result<(), crate::hamserror::HamsError> {
        let probe_c = probe.boxed().c;
        let retval = unsafe { ffi::hams_alive_remove(self.c, probe_c) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to remove probe from alive checks".to_string(),
            ));
        }
        Ok(())
    }

    /// Insert a probe into the alive checks
    ///
    /// This will insert a probe into the alive checks
    pub fn ready_insert_boxed(&self, probe: BoxedProbe) -> Result<(), crate::hamserror::HamsError> {
        // let probe_c = probe.c;
        let retval = unsafe { ffi::hams_ready_insert(self.c, probe.c) };

        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to insert probe into ready checks".to_string(),
            ));
        }

        Ok(())
    }

    pub fn ready_remove(
        &self,
        probe: &dyn crate::probes::Probe,
    ) -> Result<(), crate::hamserror::HamsError> {
        let probe_c = probe.boxed().c;
        let retval = unsafe { ffi::hams_ready_remove(self.c, probe_c) };
        if retval == 0 {
            return Err(crate::hamserror::HamsError::Message(
                "Failed to remove probe from ready checks".to_string(),
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
    use crate::probes::Probe;

    use super::*;

    /// Start and stop HaMS
    #[test]
    fn test_hams_startstop() {
        let hams = Hams::new("test").unwrap();
        hams.start().unwrap();
        hams.stop().unwrap();
    }

    /// Add and remove probes from HaMS
    #[test]
    fn add_probes_to_hams_alive() {
        let hams = Hams::new("test").unwrap();
        let probe0 = crate::probes::ProbeManual::new("probe0", true).unwrap();
        let probe1 = crate::probes::ProbeManual::new("probe1", true).unwrap();

        // todo!("Add the probe to the hams");
        hams.alive_insert_boxed(probe0.boxed())
            .expect("Should be able to add the probe");
        hams.alive_insert_boxed(probe0.boxed())
            .expect_err("Should not be able to add the same probe twice");

        hams.alive_insert_boxed(probe1.boxed())
            .expect("Should be able to add the probe");

        hams.alive_remove(&probe0.boxed())
            .expect("Should be able to remove the probe");
        hams.alive_remove(&probe0.boxed())
            .expect_err("Should not be able to remove the same probe twice");

        hams.alive_remove(&probe1.boxed())
            .expect("Should be able to remove the probe");
    }

    /// Add and remove probes from HaMS ready and alive
    #[test]
    fn add_probes_to_hams_ready() {
        let hams = Hams::new("test").unwrap();
        let probe0 = crate::probes::ProbeManual::new("probe0", true).unwrap();
        let probe1 = crate::probes::ProbeManual::new("probe1", true).unwrap();

        hams.alive_insert_boxed(probe0.boxed())
            .expect("Should be able to add the probe");
        hams.alive_insert_boxed(probe1.boxed())
            .expect("Should be able to add the probe");

        hams.ready_insert_boxed(probe0.boxed())
            .expect("Should be able to add the probe");

        hams.alive_remove(&probe0)
            .expect("Should be able to remove the probe");
        hams.alive_remove(&probe1)
            .expect("Should be able to remove the probe");

        hams.ready_remove(&probe0)
            .expect("Should be able to remove the probe");
    }
}
