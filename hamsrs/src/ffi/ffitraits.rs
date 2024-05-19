use crate::hamserror::HamsError;
use std::time::Instant;
use thin_trait_object::thin_trait_object;

/// A boxed HealthProbe for use over FFI
#[thin_trait_object]
/// Trait for health probes
pub trait HealthProbe {
    /// Name of the probe
    fn name(&self) -> Result<String, HamsError>;
    /// Check the health of the probe
    fn check(&self, time: Instant) -> Result<bool, HamsError>;

    /// Return a boxed version of the probe that is FFI safe
    fn ffi_boxed(&self) -> BoxedHealthProbe<'static>;
}
