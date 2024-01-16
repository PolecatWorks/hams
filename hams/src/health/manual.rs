use std::time::{Duration, Instant};

use super::HealthProbeInner;

/// A liveness check that is manuall controlled. Allowing the developer to manually
/// enable or disable it as appropriate.
#[derive(Debug, Hash, PartialEq)]
pub struct Manual {
    name: String,
    enabled: bool,
}
impl Eq for Manual {}

impl Manual {
    /// Construct a new Manual health check providing name and initial state
    pub fn new<S: Into<String>>(name: S, enabled: bool) -> Manual {
        Self {
            name: name.into(),
            enabled,
        }
    }

    /// Toggle state of the liveness check
    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    /// Set the state of the liveness check to a specific state
    pub fn set(&mut self, value: bool) {
        self.enabled = value
    }

    /// Enable the liveness check
    pub fn enable(&mut self) {
        self.enabled = true
    }

    /// Disabel the liveness check
    pub fn disable(&mut self) {
        self.enabled = false
    }
}

impl HealthProbeInner for Manual {
    fn name(&self) -> &str {
        &self.name
    }

    fn check_reply(&self, time: Instant) -> super::HealthProbeResult {
        super::HealthProbeResult {
            name: &self.name,
            valid: self.check(time),
        }
    }

    fn check(&self, time: Instant) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::health::health_probe::{HealthProbe, HpW};

    use super::*;

    #[test]
    fn manual() {
        println!("Checking kick");

        let now_precreate = Instant::now();
        let mut manual = Manual::new("mykick_true", true);
        assert!(manual.check(now_precreate));

        let mut manual = Manual::new("mykick_false", false);
        assert!(!manual.check(now_precreate));

        manual.toggle();
        assert!(manual.check(now_precreate));

        manual.disable();
        assert!(!manual.check(now_precreate));

        manual.enable();
        assert!(manual.check(now_precreate));

        manual.set(false);
        assert!(!manual.check(now_precreate));

        manual.set(true);
        assert!(manual.check(now_precreate));

        let mpw = HpW::new(manual);

        assert!(mpw.check(now_precreate));

        mpw.inner_through_lock().toggle();
        assert!(!mpw.check(now_precreate));
    }
}
