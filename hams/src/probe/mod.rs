use crate::error::HamsError;
use async_trait::async_trait;
use ffitraits::{BoxedHealthProbe, HealthProbe};
use serde::Serialize;
use std::ffi::CString;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::time::SystemTime;

pub(crate) mod ffitraits;

/// This module contains the kick probe
pub mod kick;
/// This module contains the manual probe
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

    async fn check(&self, time: SystemTime) -> Result<bool, HamsError>;
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

/// HealthProbe is a an AsyncHealthProbe that can be converted to a Box<dyn AsyncHealthProbe> so that it
/// is compatible with the async health that is required for some HealthChecks (network based)
/// This stuct includes a BoxedHealthProbe for the FFI probe
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FFIProbe {
    probe: BoxedHealthProbe<'static>,
}

impl<T> From<T> for FFIProbe
where
    T: HealthProbe + 'static,
{
    fn from(probe: T) -> Self {
        FFIProbe {
            probe: BoxedHealthProbe::new(probe),
        }
    }
}

#[async_trait]
impl AsyncHealthProbe for FFIProbe {
    fn name(&self) -> Result<String, HamsError> {
        Ok(unsafe { CString::from_raw(self.probe.name()) }.into_string()?)
    }

    async fn check(&self, time: SystemTime) -> Result<bool, HamsError> {
        let epoch_secs = time
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            .try_into()?;

        let check_reply = self.probe.check(epoch_secs);
        match check_reply {
            1 => Ok(true),
            0 => Ok(false),
            error_value => Err(HamsError::Message(
                "Error in check probe got value: ".to_string() + &error_value.to_string(),
            )),
        }
    }
}

// impl<T> From<T> for Box<dyn AsyncHealthProbe>
// where
//     T: HealthProbe + 'static,
// {
//     fn from(value: T) -> Self {
//         Box::new(FFIProbe::from(value))
//     }
// }

impl<T> From<T> for Box<dyn AsyncHealthProbe>
where
    T: AsyncHealthProbe + 'static,
{
    fn from(value: T) -> Self {
        Box::new(value) as Box<dyn AsyncHealthProbe>
    }
}

// impl From<BoxedHealthProbe<'static>> for FFIProbe {
//     fn from(probe: BoxedHealthProbe<'static>) -> Self {
//         FFIProbe { probe }
//     }
// }

// impl From<Box<dyn HealthProbe>> for Box<dyn AsyncHealthProbe> {
//     fn from(probe: Box< dyn HealthProbe>) -> Self {
//         let bx = BoxedHealthProbe::new(*probe);
//         Box::new(FFIProbe::from(probe))
//     }
// }

// #[thin_trait_object(
//     vtable(
//         // name of the vtable struct
//         #[repr(C)]
//         #[derive(PartialEq)]
//         pub HealthProbeVTable
//     )
// )]

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
        unsafe { CString::from_raw(self.name()) }.hash(state);
        // self.name().unwrap().hash(state);
    }
}

impl<'a> PartialEq for BoxedHealthProbe<'a> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { CString::from_raw(self.name()) }.into_string()
            == unsafe { CString::from_raw(other.name()) }.into_string()
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
    use libc::{c_int, time_t};
    use std::ffi::c_char;

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

        async fn check(&self, _time: SystemTime) -> Result<bool, HamsError> {
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

        async fn check(&self, _time: SystemTime) -> Result<bool, HamsError> {
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
            Ok(unsafe { CString::from_raw(self.probe.name()) }.into_string()?)
        }

        async fn check(&self, time: SystemTime) -> Result<bool, HamsError> {
            let epoch_secs = time
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs()
                .try_into()?;
            let reply = self.probe.check(epoch_secs);

            if reply == 1 {
                Ok(true)
            } else if reply == 0 {
                Ok(false)
            } else {
                Err(HamsError::Message("Error in check probe".to_string()))
            }
        }
    }

    /// Confirm check and name work for AsyncHealthProbe
    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_async_health_probe() {
        let probe = AsyncProbe0 {
            name: "test".to_string(),
            check: true,
        };
        assert_eq!(probe.name().unwrap(), "test");
        assert!(probe.check(SystemTime::now()).await.unwrap());
    }

    /// Create a vec of AsyncHealthProbe and run check on each
    #[tokio::test]
    #[cfg_attr(miri, ignore)]
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
            results.push(probe.check(SystemTime::now()).await.unwrap());
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
        #[doc = " Name of the probe"]
        fn name(&self) -> *mut c_char {
            CString::new(self.name.clone()).unwrap().into_raw()
        }

        fn check(&self, _time: time_t) -> c_int {
            self.check as c_int
        }
    }

    #[test]
    fn health_probe_to_from_boxed() {
        let probe = Probe0 {
            name: "test".to_string(),
            check: true,
        };

        let boxed = BoxedHealthProbe::new(probe.clone());

        assert_eq!(
            unsafe { CString::from_raw(boxed.name()) }
                .into_string()
                .unwrap(),
            "test"
        );
    }

    #[test]
    fn test_health_probe() {
        let probe = Probe0 {
            name: "test".to_string(),
            check: true,
        };
        // info!("Releasing kick probe: {}", CString::from_raw(probe.name()).into_string().unwrap());
        assert_eq!(
            unsafe { CString::from_raw(probe.name()) }
                .into_string()
                .unwrap(),
            "test"
        );
        let time_now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap();
        assert!(probe.check(time_now) == 1);
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
