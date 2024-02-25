/// A liveness check that automatically fails when the timer has not been reset before
/// the duration. Equivalent of a dead mans handle.
#[derive(Debug, Hash, PartialEq)]
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
    use std::time::Duration;

    #[test]
    fn test_kick() {
        let mut probe = Kick::new("test", Duration::from_secs(1));
        assert_eq!(probe.check(Instant::now()).unwrap(), true);
        probe.kick();
        assert_eq!(probe.check(Instant::now()).unwrap(), true);
        std::thread::sleep(Duration::from_secs(2));
        assert_eq!(probe.check(Instant::now()).unwrap(), false);
    }
}
