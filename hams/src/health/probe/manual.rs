use super::{BoxedHealthProbe, HealthProbe};
use crate::error::HamsError;

use std::{
    sync::{Arc, Mutex},
    time::Instant,
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

    pub fn boxed_probe(&self) -> BoxedHealthProbe<'static> {
        BoxedHealthProbe::new(self.clone())
    }
}

impl HealthProbe for Manual {
    fn name(&self) -> Result<String, HamsError> {
        Ok(self.name.clone())
    }

    fn check(&self, _time: Instant) -> Result<bool, HamsError> {
        Ok(self.enabled.lock().unwrap().valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual() {
        let mut probe = Manual::new("test", true);
        assert_eq!(probe.check(Instant::now()).unwrap(), true);
        probe.disable();
        assert_eq!(probe.check(Instant::now()).unwrap(), false);
        probe.enable();
        assert_eq!(probe.check(Instant::now()).unwrap(), true);
        probe.toggle();
        assert_eq!(probe.check(Instant::now()).unwrap(), false);
        probe.toggle();
        assert_eq!(probe.check(Instant::now()).unwrap(), true);
    }

    // Test that clone of Manual refers to same inner
    #[test]
    fn test_manual_clone() {
        let mut probe = Manual::new("test", true);
        let probe2 = probe.clone();
        assert!(probe.check(Instant::now()).unwrap());
        assert!(probe2.check(Instant::now()).unwrap());
        probe.disable();
        assert!(!probe.check(Instant::now()).unwrap());
        assert!(!probe2.check(Instant::now()).unwrap());
    }

    // Test that the probe can be inserted into a HealthCheck
    #[test]
    fn test_insert() {
        let mut manual = Manual::new("test", true);

        // let probe = BoxedHealthProbe::new(manual.clone());
        let probe = manual.boxed_probe();

        assert!(probe.check(Instant::now()).unwrap());

        manual.toggle();

        assert!(!probe.check(Instant::now()).unwrap());

        let check = crate::health::check::HealthCheck::new("test");

        check.insert(probe);

        assert_eq!(check.probes.lock().unwrap().len(), 1);

        check.remove(&manual.boxed_probe());

        assert_eq!(check.probes.lock().unwrap().len(), 0);
    }
}
