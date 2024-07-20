/// Manual probe provides a liveness probe that is explicitly enabled and disabled.
use super::HealthProbe;
use libc::time_t;
use std::{
    ffi::{c_char, CString},
    sync::{Arc, Mutex},
};

#[derive(Debug, Hash, PartialEq)]
struct Inner {
    valid: bool,
}

/// A liveness check that is manually controlled. Allowing the developer to
/// enable,  disable or toggle it as appropriate.
#[derive(Debug, Clone)]
pub struct Manual {
    name: String,
    enabled: Arc<Mutex<Inner>>,
}

impl Manual {
    /// Create a new Manual probe with the given name and enabled state
    pub fn new<S: Into<String>>(name: S, enabled: bool) -> Self {
        Self {
            name: name.into(),
            enabled: Arc::new(Mutex::new(Inner { valid: enabled })),
        }
    }

    /// Enable the probe
    pub fn enable(&mut self) {
        self.enabled.lock().unwrap().valid = true;
    }

    /// Disable the probe
    pub fn disable(&mut self) {
        self.enabled.lock().unwrap().valid = false;
    }

    /// Toggle the probe
    pub fn toggle(&mut self) {
        let mut inner = self.enabled.lock().unwrap();
        inner.valid = !inner.valid;
    }

    // pub fn boxed_probe(&self) -> BoxedHealthProbe<'static> {
    //     BoxedHealthProbe::new(self.clone())
    // }
}

impl HealthProbe for Manual {
    #[doc = "Name of the probe"]
    fn name(&self) -> *mut c_char {
        CString::new(self.name.clone()).unwrap().into_raw()
    }

    fn check(&self, _time: time_t) -> i32 {
        self.enabled.lock().unwrap().valid as i32
    }
}

// impl Into<FFIProbe> for Manual {
//     fn into(self) -> FFIProbe {
//         FFIProbe::from(BoxedHealthProbe::new(self))
//     }
// }

#[cfg(test)]
mod tests {

    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn test_manual() {
        let mut probe = Manual::new("test", true);

        let time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap();

        assert!(probe.check(time_now) == 1);
        probe.disable();
        assert!(probe.check(time_now) == 0);
        probe.enable();
        assert!(probe.check(time_now) == 1);
        probe.toggle();
        assert!(probe.check(time_now) == 0);
        probe.toggle();
        assert!(probe.check(time_now) == 1);

        drop(probe);
    }

    // Test that clone of Manual refers to same inner
    #[test]
    fn test_manual_clone() {
        let mut probe = Manual::new("test", true);
        let probe2 = probe.clone();

        let time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap();

        assert!(probe.check(time_now) == 1);
        assert!(probe2.check(time_now) == 1);
        probe.disable();

        assert!(probe.check(time_now) == 0);
        assert!(probe2.check(time_now) == 0);

        drop(probe);
        drop(probe2);
    }

    // Test that the probe can be inserted into a HealthCheck
    // #[tokio::test]
    // async fn test_insert() {
    //     let mut manual = Manual::new("test", true);

    //     // let probe = BoxedHealthProbe::new(manual.clone());
    //     let probe = BoxedHealthProbe::new(manual.clone());

    //     assert!(probe.check(Instant::now()).unwrap());

    //     manual.toggle();

    //     assert!(!probe.check(Instant::now()).unwrap());

    //     let check = crate::health::check::HealthCheck::new("test");

    //     check.async_insert(FFIProbe::from(manual.clone())).await;

    //     assert_eq!(check.probes.lock().await.len(), 1);

    //     let new_probe = BoxedHealthProbe::new(manual.clone());

    //     let x = Box::new(FFIProbe::from(new_probe)) as Box<dyn AsyncHealthProbe>;

    //     blocking_probe_remove(&check, &manual).await;
    //     // check.remove(&x);

    //     assert_eq!(check.probes.lock().await.len(), 0);
    // }
}
