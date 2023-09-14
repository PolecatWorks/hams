use std::time::Instant;

use crate::{
    error::HamsError,
    health::{Health, HealthCheckResult},
};

/** Manual Health check that uses enable/disable to set liveness */
#[derive(Debug)]
pub struct HealthManual {
    name: String,
    pub state: bool,
}

impl Health for HealthManual {
    fn check(&self, _time: Instant) -> Result<HealthCheckResult, HamsError> {
        Ok(HealthCheckResult {
            name: self.name.clone(),
            valid: self.state,
        })
    }
}

impl HealthManual {
    pub fn new<S: Into<String>>(name: S, state: bool) -> Self {
        Self {
            name: name.into(),
            state,
        }
    }

    pub fn enable(&mut self) {
        self.set(true)
    }
    pub fn disable(&mut self) {
        self.set(false)
    }
    pub fn set(&mut self, state: bool) {
        self.state = state
    }
}
