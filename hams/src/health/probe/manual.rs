use tokio::time::Instant;

use super::HealthProbe;
use crate::error::HamsError;

use std::sync::{Arc, Mutex};

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
    fn name(&self) -> Result<String, HamsError> {
        Ok(self.name.clone())
    }

    fn check(&self, _time: Instant) -> Result<bool, HamsError> {
        Ok(self.enabled.lock().unwrap().valid)
    }
}

// impl Into<FFIProbe> for Manual {
//     fn into(self) -> FFIProbe {
//         FFIProbe::from(BoxedHealthProbe::new(self))
//     }
// }

#[cfg(test)]
mod tests {
    use crate::health::check::blocking_probe_remove;
    use crate::health::probe::{AsyncHealthProbe, BoxedHealthProbe, FFIProbe};

    use super::*;

    #[test]
    fn test_manual() {
        let mut probe = Manual::new("test", true);
        assert!(probe.check(Instant::now()).unwrap());
        probe.disable();
        assert!(!probe.check(Instant::now()).unwrap());
        probe.enable();
        assert!(probe.check(Instant::now()).unwrap());
        probe.toggle();
        assert!(!probe.check(Instant::now()).unwrap());
        probe.toggle();
        assert!(probe.check(Instant::now()).unwrap());
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
    #[tokio::test]
    async fn test_insert() {
        let mut manual = Manual::new("test", true);

        // let probe = BoxedHealthProbe::new(manual.clone());
        let probe = BoxedHealthProbe::new(manual.clone());

        assert!(probe.check(Instant::now()).unwrap());

        manual.toggle();

        assert!(!probe.check(Instant::now()).unwrap());

        let check = crate::health::check::HealthCheck::new("test");

        check.async_insert(FFIProbe::from(manual.clone())).await;

        assert_eq!(check.probes.lock().await.len(), 1);

        let new_probe = BoxedHealthProbe::new(manual.clone());

        let x = Box::new(FFIProbe::from(new_probe)) as Box<dyn AsyncHealthProbe>;

        blocking_probe_remove(&check, &manual).await;
        // check.remove(&x);

        assert_eq!(check.probes.lock().await.len(), 0);
    }
}
