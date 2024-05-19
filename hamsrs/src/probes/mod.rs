mod custom;
mod kick;
mod manual;

pub use kick::ProbeKick;
pub use manual::ProbeManual;

use crate::ffi;

#[derive(Clone)]
pub struct BoxedProbe {
    // This pointer must never be allowed to leave the struct
    pub(crate) c: *mut ffi::Probe,
}

pub trait Probe {
    fn boxed(&self) -> BoxedProbe;
}

impl Probe for BoxedProbe {
    fn boxed(&self) -> BoxedProbe {
        self.clone()
    }
}
