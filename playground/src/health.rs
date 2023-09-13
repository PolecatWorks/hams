use std::fmt::Display;

use std::{
    hash::Hasher,
    sync::{Arc, Mutex, MutexGuard},
    time::Instant,
};

use serde::Serialize;

/// Detail structure for replies from ready and alive
#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct HealthCheckResult {
    /// Name of health Reply
    pub name: String,
    /// Return value of health Reply
    pub valid: bool,
}

impl<'a> Display for HealthCheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.valid)
    }
}

/** Health trait requires that the object implements the check function that returns a HealthCheckResult
 ** suitable for inclusion in a k8s health probe (eg ready or alive)
 */
pub trait Health {
    fn check(&self, time: Instant) -> HealthCheckResult;
}
