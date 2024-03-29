use crate::error::HamsError;
use crate::health::probe::HealthProbe;
use std::time::{Duration, Instant};

use super::BoxedHealthProbe;

/// A liveness check that automatically fails when the timer has not been reset before
/// the duration. Equivalent of a dead mans handle.
#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Kick {
    name: String,
    latest: Instant,
    margin: Duration,
}

impl Kick {
    pub fn new<S: Into<String>>(name: S, margin: Duration) -> Self {
        Self {
            name: name.into(),
            latest: Instant::now(),
            margin,
        }
    }

    pub fn kick(&mut self) {
        self.latest = Instant::now();
    }

    pub fn boxed_probe(&self) -> BoxedHealthProbe<'static> {
        BoxedHealthProbe::new(self.clone())
    }
}

impl HealthProbe for Kick {
    fn name(&self) -> Result<String, HamsError> {
        Ok(self.name.clone())
    }

    fn check(&self, time: Instant) -> Result<bool, HamsError> {
        Ok(time < self.latest + self.margin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kick() {
        let mut probe = Kick::new("test", Duration::from_secs(1));
        assert_eq!(probe.check(Instant::now()).unwrap(), true);
        probe.kick();
        assert_eq!(probe.check(Instant::now()).unwrap(), true);
        //No need to sleep, we can just check the time
        assert_eq!(
            probe
                .check(Instant::now() + Duration::from_secs(2))
                .unwrap(),
            false
        );
    }
}
