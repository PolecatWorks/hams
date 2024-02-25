/// A liveness check that is manuall controlled. Allowing the developer to manually
/// enable or disable it as appropriate.
#[derive(Debug, Hash, PartialEq)]
pub struct Manual {
    name: String,
    enabled: bool,
}

impl Manual {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            enabled: true,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl HealthProbe for Manual {
    fn name(&self) -> Result<String, HamsError> {
        Ok(self.name.clone())
    }

    fn check(&self, _time: Instant) -> Result<bool, HamsError> {
        Ok(self.enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_manual() {
        let mut probe = Manual::new("test");
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
}
