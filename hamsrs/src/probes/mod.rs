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
    /// Get the Boxed Probe for the probe. This is used to pass the probe to the C API
    /// This method provides a cloned BProbe that is owned by the caller
    fn boxed(&self) -> Result<ffi::BProbe, HamsError>;
}
