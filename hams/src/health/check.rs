use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Instant,
};

use serde::Serialize;

use super::probe::{BoxedHealthProbe, HealthProbe};

/// Reply structure to return from a health check
#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct HealthCheckReply {
    pub(crate) name: String,
    pub(crate) valid: bool,
}

impl<'a> warp::Reply for HealthCheckReply {
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
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            probes: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn insert(&self, probe: BoxedHealthProbe<'static>) {
        self.probes.lock().unwrap().insert(probe);
    }

    pub fn remove(&self, probe: BoxedHealthProbe<'static>) {
        self.probes.lock().unwrap().remove(&probe);
    }

    pub fn check(&self, time: Instant) -> bool {
        self.probes
            .lock()
            .unwrap()
            .iter()
            .all(|probe| probe.check(time).unwrap_or_else(|_err| false))
    }

    pub fn check_reply(&self, time: Instant) -> Vec<HealthCheckReply> {
        self.probes
            .lock()
            .unwrap()
            .iter()
            .map(|probe| HealthCheckReply {
                name: probe.name().unwrap_or_else(|_err| "Unknown".to_string()),
                valid: probe.check(time).unwrap_or_else(|_err| false),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::probe::manual::ManualHealthProbe;
    use std::time::Duration;

    #[test]
    fn test_health_check() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(ManualHealthProbe::new("test_probe", true));
        health_check.insert(probe);
        assert!(health_check.check(Instant::now()));
    }

    #[test]
    fn test_health_probe_remove() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(ManualHealthProbe::new("test_probe", true));
        health_check.insert(probe);
        let probe = BoxedHealthProbe::new(ManualHealthProbe::new("test_probe2", true));
        health_check.insert(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 2);
        assert!(health_check.check(Instant::now()));
        let probe = BoxedHealthProbe::new(ManualHealthProbe::new("test_probe", true));
        health_check.remove(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 1);
        assert!(health_check.check(Instant::now()));
    }

    #[test]
    fn test_health_check_reply() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(ManualHealthProbe::new("test_probe", true));
        health_check.insert(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0].name, "test_probe");
        assert_eq!(replies[0].valid, true);
    }

    #[test]
    fn test_health_check_reply_fail() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(ManualHealthProbe::new("test_probe", false));
        health_check.insert(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0].name, "test_probe");
        assert_eq!(replies[0].valid, false);
    }

    #[test]
    fn test_health_check_reply_multiple() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(ManualHealthProbe::new("test_probe", true));
        health_check.insert(probe);
        let probe = BoxedHealthProbe::new(ManualHealthProbe::new("test_probe2", false));
        health_check.insert(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 2);
    }

    #[test]
    fn test_health_check_reply_failures() {
        let health_check = HealthCheck::new("test");
        let probe = BoxedHealthProbe::new(ManualHealthProbe::new("test_probe", true));
        health_check.insert(probe);
        let probe = BoxedHealthProbe::new(ManualHealthProbe::new("test_probe2", false));
        health_check.insert(probe);
        let replies = health_check.check_reply(Instant::now());
        assert_eq!(replies.len(), 2);
    }
}
