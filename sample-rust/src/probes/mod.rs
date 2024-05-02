mod manual;

pub use manual::ProbeManual;

use crate::ffi;

pub struct BoxedProbe {
    // This pointer must never be allowed to leave the struct
    pub(crate) c: *mut ffi::Probe,
}

// impl BoxedProbe {
//     // access for c var
//     pub fn c(&self) -> *mut ffi::Probe {
//         println!("c is");
//         self.c
//     }
// }
