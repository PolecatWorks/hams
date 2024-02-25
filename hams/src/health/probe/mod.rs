use std::fmt::Display;

use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use thin_trait_object::thin_trait_object;

use crate::error::HamsError;

use std::fmt;

pub mod kick;
pub mod manual;

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

/// A boxed HealthProbe for use over FFI
#[thin_trait_object]
/// Trait for health probes
pub trait HealthProbe {
    /// Name of the probe
    fn name(&self) -> Result<String, HamsError>;
    /// Check the health of the probe
    fn check(&self, time: Instant) -> Result<bool, HamsError>;
}

impl fmt::Debug for BoxedHealthProbe<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BoxedHealthProbe")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_probe_result() {
        let hpr = HealthProbeResult {
            name: "test",
            valid: true,
        };
        assert_eq!(hpr.name, "test");
        assert_eq!(hpr.valid, true);
    }

    struct Probe0 {
        name: String,
        check: bool,
    }

    impl HealthProbe for Probe0 {
        fn name(&self) -> Result<String, HamsError> {
            Ok(self.name.clone())
        }

        fn check(&self, _time: Instant) -> Result<bool, HamsError> {
            Ok(self.check)
        }
    }

    #[test]
    fn test_health_probe() {
        let probe = Probe0 {
            name: "test".to_string(),
            check: true,
        };
        assert_eq!(probe.name().unwrap(), "test");
        assert_eq!(probe.check(Instant::now()).unwrap(), true);
    }

    #[test]
    fn test_health_probe_hashset() {
        let probe0 = BoxedHealthProbe::new(Probe0 {
            name: "test".to_string(),
            check: true,
        });
        let probe1 = BoxedHealthProbe::new(Probe0 {
            name: "test".to_string(),
            check: true,
        });
        let mut set = std::collections::HashSet::new();
        set.insert(probe0);
        assert_eq!(set.contains(&probe1), true);

        set.insert(probe1);

        assert_eq!(set.len(), 1);
    }
}
