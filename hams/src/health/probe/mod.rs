use std::fmt::Display;

use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use thin_trait_object::thin_trait_object;

use crate::error::HamsError;

use std::fmt;

/// Detail structure for replies from ready and alive for a single probe
#[derive(Serialize, PartialEq, Clone)]
pub struct HealthProbeResult<'a> {
    /// Name of health Reply
    pub name: &'a str,
    /// Return value of health Reply
    pub valid: bool,
}
unsafe impl Send for BoxedHealthProbe<'_> {}

impl<'a> fmt::Debug for HealthProbeResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.valid)
    }
}

impl<'a> Hash for BoxedHealthProbe<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().unwrap().hash(state);
    }
}

impl<'a> PartialEq for BoxedHealthProbe<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name().unwrap() == other.name().unwrap()
    }
}

impl<'a> Eq for BoxedHealthProbe<'a> {}

impl<'a> Display for HealthProbeResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.valid)
    }
}

#[thin_trait_object]
pub trait HealthProbe {
    fn name(&self) -> Result<String, HamsError>;
    fn check(&self, time: Instant) -> Result<bool, HamsError>;
}

impl fmt::Debug for BoxedHealthProbe<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BoxedHealthProbe")
    }
}
