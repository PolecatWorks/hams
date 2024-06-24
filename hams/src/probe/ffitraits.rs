use libc::time_t;
use std::ffi::{c_char, c_int};
use thin_trait_object::thin_trait_object;

/// A boxed HealthProbe for use over FFI
#[thin_trait_object]
/// Trait for health probes
pub trait HealthProbe: Sync + Send {
    /// Name of the probe. Created as a c_str and converted to a raw pointer
    /// to be used in FFI.
    /// Received owns the pointer and is responsible for freeing it.
    fn name(&self) -> *mut c_char;
    /// Check the health of the probe
    /// Returns 1 if the probe is healthy, 0 otherwise
    /// Returns -1 if an error occurred
    fn check(&self, time: time_t) -> c_int;
}
