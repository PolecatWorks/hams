use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Instant,
};

use serde::Serialize;

use super::probe::{BoxedHealthProbe, HealthProbe, HealthProbeResult};

/// Reply structure to return from a health check
#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct HealthCheckResult {
    pub(crate) name: String,
    pub(crate) valid: bool,
}

impl warp::Reply for HealthCheckResult {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::with_status(
            warp::reply::json(&self),
            if self.valid {
                warp::http::StatusCode::OK
            } else {
                warp::http::StatusCode::NOT_ACCEPTABLE
            },
        )
        .into_response()
    }
}

/// Represent the [HealthCheck] which collects [HealthProbe]s and replies to a check with a struct that can use returned as
/// a kubernetes readyness or liveness probe
///
/// Problems when creating this as the 'static lifetime required for dyn causes issues making this struct Send safe
/// This seems to capture the issue: https://users.rust-lang.org/t/why-this-impl-type-lifetime-may-not-live-long-enough/67855
#[derive(Debug, Clone)]
pub struct HealthCheck {
    name: String,
    probes: Arc<Mutex<HashSet<BoxedHealthProbe<'static>>>>,
}

impl HealthCheck {
    /// Create a new HealthCheck with a name
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            probes: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Insert a probe into the HealthCheck
    pub fn insert(&self, probe: BoxedHealthProbe<'static>) {
        self.probes.lock().unwrap().insert(probe);
    }

    /// Remove a probe from the HealthCheck
    pub fn remove(&self, probe: BoxedHealthProbe<'static>) {
        self.probes.lock().unwrap().remove(&probe);
    }

    /// Check the health of the HealthCheck
    pub fn check(&self, time: Instant) -> HealthCheckResult {
        let valid = self
            .probes
            .lock()
            .unwrap()
            .iter()
            .all(|probe| probe.check(time).unwrap_or_else(|_err| false));
        HealthCheckResult {
            name: self.name.clone(),
            valid,
        }
    }

    /// Check the health of the HealthCheck and return a vector of results of type [HealthProbeResult]
    pub fn check_reply(&self, time: Instant) -> Vec<HealthProbeResult> {
        self.probes
            .lock()
            .unwrap()
            .iter()
            .map(|probe| HealthProbeResult {
                name: probe.name().unwrap_or("Unknown".to_string()),
                valid: probe.check(time).unwrap_or(false),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::{check, probe::manual::Manual};
    use std::time::Duration;

    #[test]
    fn test_health_check() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(Manual::new("test_probe", true));
        health_check.insert(probe);
        assert!(health_check.check(Instant::now()).valid);
    }

    #[test]
    fn test_health_probe_remove() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(Manual::new("test_probe", true));
        health_check.insert(probe);
        let probe = BoxedHealthProbe::new(Manual::new("test_probe2", true));
        health_check.insert(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 2);
        let probe = BoxedHealthProbe::new(Manual::new("test_probe", true));
        health_check.remove(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 1);
    }

    #[test]
    fn test_health_check_reply() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(Manual::new("test_probe", true));
        health_check.insert(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0].name, "test_probe");
        assert_eq!(replies[0].valid, true);
        let check = health_check.check(Instant::now());
        assert!(check.valid);
    }

    #[test]
    fn test_health_check_reply_fail() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(Manual::new("test_probe", false));
        health_check.insert(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0].name, "test_probe");
        assert_eq!(replies[0].valid, false);
    }

    #[test]
    fn test_health_check_reply_multiple() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(Manual::new("test_probe", true));
        health_check.insert(probe);
        let probe = BoxedHealthProbe::new(Manual::new("test_probe2", true));
        health_check.insert(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 2);
        let check = health_check.check(Instant::now());
        assert!(check.valid);
    }

    #[test]
    fn test_health_check_reply_failures() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(Manual::new("test_probe", true));
        health_check.insert(probe);
        let probe = BoxedHealthProbe::new(Manual::new("test_probe2", false));
        health_check.insert(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 2);
        let check = health_check.check(Instant::now());
        assert!(!check.valid);
    }
}
