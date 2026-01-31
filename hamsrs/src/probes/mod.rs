mod custom;
mod kick;
mod manual;

pub use kick::ProbeKick;
pub use manual::ProbeManual;

use crate::{ffi, hamserror::HamsError};

// #[derive(Clone)]
// pub struct BoxedProbe {
//     // This pointer must never be allowed to leave the struct
//     pub(crate) c: *mut ffi::BProbe,
// }

pub trait Probe {
    /// Get the Boxed Probe for the probe.
    ///
    /// This method is used to pass the probe to the C API.
    /// It provides a cloned `BProbe` that is owned by the caller.
    ///
    /// # Errors
    /// Returns a `HamsError` if the probe cannot be boxed.
    fn boxed(&self) -> Result<ffi::BProbe, HamsError>;
}
