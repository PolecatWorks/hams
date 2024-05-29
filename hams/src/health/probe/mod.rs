use std::fmt::Display;

use serde::Serialize;
use tokio::time::Instant;

use std::hash::{Hash, Hasher};

use thin_trait_object::thin_trait_object;

use crate::error::HamsError;

use async_trait::async_trait;
use std::fmt;
use std::fmt::Debug;

pub mod kick;
pub mod manual;

/// Detail structure for replies from ready and alive for a single probe
#[derive(Serialize, PartialEq, Clone)]
pub struct HealthProbeResult {
    /// Name of health Reply
    pub name: String,
    /// Return value of health Reply
    pub valid: bool,
}

impl fmt::Debug for HealthProbeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.valid)
    }
}

impl Display for HealthProbeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.valid)
    }
}

#[async_trait]
pub(crate) trait AsyncHealthProbe: Debug + Sync + Send {
    // pub(crate) trait AsyncHealthProbe: Debug + Sync + Send + Eq + Hash {
    fn name(&self) -> Result<String, HamsError>;
    async fn check(&self, time: Instant) -> Result<bool, HamsError>;
}

impl Hash for dyn AsyncHealthProbe {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().unwrap().hash(state);
    }
}

impl PartialEq for dyn AsyncHealthProbe {
    fn eq(&self, other: &Self) -> bool {
        self.name().unwrap() == other.name().unwrap()
    }
}
impl Eq for dyn AsyncHealthProbe {}

// #[thin_trait_object(
//     vtable(
//         // name of the vtable struct
//         #[repr(C)]
//         #[derive(PartialEq)]
//         pub HealthProbeVTable
//     )
// )]

// A boxed HealthProbe for use over FFI
#[thin_trait_object]
pub trait HealthProbe: Sync + Send {
    /// Name of the probe
    fn name(&self) -> Result<String, HamsError>;
    /// Check the health of the probe
    fn check(&self, time: Instant) -> Result<bool, HamsError>;

    /// Return a boxed version of the probe that is FFI safe
    fn ffi_boxed(&self) -> BoxedHealthProbe<'static>;
}

// impl BoxedHealthProbe {
//     /// Create a new BoxedHealthProbe from a HealthProbe
//     pub fn boxme(probe: &impl HealthProbe + 'static) -> Self {
//         BoxedHealthProbe::new(probe.clone())
//     }
// }

impl<'a> Hash for BoxedHealthProbe<'a> {
    // NOTE: Use a unique identifier to distinguish probes. NOT the probe address.
    // Reference here: https://stackoverflow.com/questions/72148631/how-can-i-hash-by-a-raw-pointer
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

impl fmt::Debug for BoxedHealthProbe<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BoxedHealthProbe")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Hash)]
    struct AsyncProbe0 {
        name: String,
        check: bool,
    }

    #[async_trait]
    impl AsyncHealthProbe for AsyncProbe0 {
        fn name(&self) -> Result<String, HamsError> {
            Ok(self.name.clone())
        }

        async fn check(&self, _time: Instant) -> Result<bool, HamsError> {
            Ok(self.check)
        }
    }

    #[derive(Debug, Hash, PartialEq, Eq)]
    struct AsyncProbe1 {
        name: String,
        check: bool,
    }

    #[async_trait]
    impl AsyncHealthProbe for AsyncProbe1 {
        fn name(&self) -> Result<String, HamsError> {
            Ok(self.name.clone())
        }

        async fn check(&self, _time: Instant) -> Result<bool, HamsError> {
            Ok(self.check)
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash)]
    struct FFIProbe {
        probe: BoxedHealthProbe<'static>,
    }

    #[async_trait]
    impl AsyncHealthProbe for FFIProbe {
        fn name(&self) -> Result<String, HamsError> {
            self.probe.name()
        }

        async fn check(&self, time: Instant) -> Result<bool, HamsError> {
            self.probe.check(time)
        }
    }

    /// Confirm check and name work for AsyncHealthProbe
    #[tokio::test]
    async fn test_async_health_probe() {
        let probe = AsyncProbe0 {
            name: "test".to_string(),
            check: true,
        };
        assert_eq!(probe.name().unwrap(), "test");
        assert!(probe.check(Instant::now()).await.unwrap());
    }

    /// Create a vec of AsyncHealthProbe and run check on each
    #[tokio::test]
    async fn test_async_health_probe_vec() {
        let probe0 = AsyncProbe0 {
            name: "test0".to_string(),
            check: true,
        };
        let probe1 = AsyncProbe1 {
            name: "test1".to_string(),
            check: false,
        };
        let probe2 = FFIProbe {
            probe: BoxedHealthProbe::new(Probe0 {
                name: "test2".to_string(),
                check: true,
            }),
        };

        let probes: Vec<Box<dyn AsyncHealthProbe>> =
            vec![Box::new(probe0), Box::new(probe1), Box::new(probe2)];
        let mut results = Vec::new();
        for probe in probes {
            results.push(probe.check(Instant::now()).await.unwrap());
        }
        assert_eq!(results, vec![true, false, true]);
    }

    #[test]
    fn test_health_probe_result() {
        let hpr = HealthProbeResult {
            name: "test".to_owned(),
            valid: true,
        };
        assert_eq!(hpr.name, "test");
        assert!(hpr.valid);
    }

    #[derive(Clone)]
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
        fn ffi_boxed(&self) -> BoxedHealthProbe<'static> {
            BoxedHealthProbe::new(self.clone())
        }
    }

    #[test]
    fn health_probe_to_from_boxed() {
        let probe = Probe0 {
            name: "test".to_string(),
            check: true,
        };

        let boxed = BoxedHealthProbe::new(probe.clone());

        assert_eq!(boxed.name().unwrap(), "test");
    }

    #[test]
    fn test_health_probe() {
        let probe = Probe0 {
            name: "test".to_string(),
            check: true,
        };
        assert_eq!(probe.name().unwrap(), "test");
        assert!(probe.check(Instant::now()).unwrap());
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
        assert!(set.contains(&probe1));

        set.insert(probe1);

        assert_eq!(set.len(), 1);
    }
}
