use std::{collections::HashSet, sync::Arc};

use futures::future::join_all;
use serde::Serialize;
use tokio::{sync::Mutex, task, time::Instant};

use crate::health::probe::HealthProbe;

use super::probe::{AsyncHealthProbe, HealthProbeResult};

/// Reply structure to return from a health check
#[derive(Debug, Serialize)]
pub struct HealthCheckResult {
    pub(crate) name: String,
    pub(crate) valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) details: Option<Vec<HealthProbeResult>>,
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
    pub name: String,
    pub(crate) probes: Arc<Mutex<HashSet<Box<dyn AsyncHealthProbe>>>>,
}

// TODO: This does not look right to add Send to HealthCheck
unsafe impl Send for HealthCheck {}

impl HealthCheck {
    /// Create a new HealthCheck with a name
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            probes: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Insert a probe into the HealthCheck
    pub(crate) fn insert(&self, probe: Box<dyn AsyncHealthProbe + 'static>) -> bool {
        self.probes.blocking_lock().insert(probe)
    }

    /// Create an async version of the insert function to wrap around the blocking one and allow async insert
    /// This should only be used in test
    #[cfg(test)]
    pub(crate) async fn async_insert_dyn(
        &self,
        probe: Box<dyn AsyncHealthProbe + 'static>,
    ) -> bool {
        let check = self.clone();

        task::spawn_blocking(move || check.insert(probe))
            .await
            .expect("insert to probe")
    }

    #[cfg(test)]
    pub(crate) async fn async_insert<T: AsyncHealthProbe + 'static>(&self, probe: T) -> bool {
        let check = self.clone();

        task::spawn_blocking(move || check.insert(Box::new(probe)))
            .await
            .expect("insert to probe")
    }

    /// Remove a probe from the HealthCheck
    pub(crate) fn remove(&self, probe: &Box<dyn AsyncHealthProbe + 'static>) -> bool {
        self.probes.blocking_lock().remove(probe)
    }

    // #[cfg(test)]
    // pub(crate) async fn async_remove_dyn(&self, probe: &Box<dyn AsyncHealthProbe + 'static>) -> bool {
    //     let check = self.clone();

    //     task::spawn_blocking(move || {
    //         check.remove(probe)
    //     }).await.expect("remove a probe")
    // }

    /// Check the health of the HealthCheck
    pub async fn check(&self, time: Instant) -> HealthCheckResult {
        let checks = self.probes.lock().await;

        // TODO: The use of std Mutex (MutexGuard cannot be sent over an async bondary)
        // Can we code this so that the MutexGuard is not sent over the async boundary? OR do we need to use the tokio::Mutex
        // Downside of that is that we need to use mutex:: blocking_lock() where executing on the synchronous
        // NOTE: Did attempt to clone the AsyncHealthProbes to use outside the mutex BUT that does not work as the AsyncHealthProbe is dyn so cannot be Sized as it is erased.

        // TODOL Consider to buffer this so that we run multiple checks in parallel
        let valid = checks
            .iter()
            .map(|probe| async { probe.check(time).await.unwrap_or(false) });

        let valids = join_all(valid).await;
        let valid = valids.into_iter().all(|check| check);

        HealthCheckResult {
            name: self.name.clone(),
            valid,
            details: None,
        }
    }

    /// Check the health of the HealthCheck and return a vector of results of type [HealthProbeResult]
    pub async fn check_verbose(&self, time: Instant) -> HealthCheckResult {
        let my_probes = self.probes.lock().await;

        let checks: Vec<_> = my_probes
            .iter()
            .map(|probe| async {
                HealthProbeResult {
                    name: probe.name().unwrap_or("Unknown".to_string()),
                    valid: probe.check(time).await.unwrap_or(false),
                }
            })
            .collect();

        let checks = futures::future::join_all(checks).await;

        // let checks: Vec<HealthProbeResult> = vec!();

        HealthCheckResult {
            name: self.name.clone(),
            valid: checks.iter().all(|check| check.valid),
            details: Some(checks),
        }
    }
}

#[cfg(test)]
pub(crate) async fn blocking_probe_remove<T: HealthProbe + Clone + 'static>(
    check: &HealthCheck,
    probe: &T,
) -> bool {
    let check = check.clone();
    let probe = (*probe).clone();

    task::spawn_blocking(move || check.remove(&probe.into()))
        .await
        .expect("removed alive")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::{
        self,
        probe::{manual::Manual, BoxedHealthProbe, FFIProbe},
    };

    #[tokio::test]
    async fn test_health_check() {
        let health_check = HealthCheck::new("test");
        let probe = Manual::new("test_probe", true);
        assert!(
            health_check
                .async_insert(FFIProbe::from(probe.clone()))
                .await
        );
        // health_check.insert(probe.into());
        assert!(health_check.check(Instant::now()).await.valid);
    }

    #[tokio::test]
    async fn test_health_probe_remove() {
        let health_check = HealthCheck::new("test");

        let manual0 = Manual::new("test_probe0", true);
        assert!(
            health_check
                .async_insert(FFIProbe::from(manual0.clone()))
                .await
        );

        let manual1 = Manual::new("test_probe1", true);
        assert!(
            health_check
                .async_insert(FFIProbe::from(manual1.clone()))
                .await
        );

        let replies = health_check.check_verbose(Instant::now()).await;
        assert_eq!(replies.details.unwrap().len(), 2);

        // let probe = BoxedHealthProbe::new(Manual::new("test_probe", true));
        assert!(blocking_probe_remove(&health_check, &manual0).await);
        // health_check.remove(&BoxedHealthProbe::new(manual0.clone()).into());
        let replies = health_check.check_verbose(Instant::now()).await;
        let details = replies.details.unwrap();
        assert_eq!(details.len(), 1);

        assert!(!blocking_probe_remove(&health_check, &manual0).await);
        assert_eq!(details.len(), 1);

        assert!(blocking_probe_remove(&health_check, &manual1).await);
        let replies = health_check.check_verbose(Instant::now()).await;
        assert_eq!(replies.details.unwrap().len(), 0);
    }

    #[test]
    #[ignore]
    fn test_health_probe_by_ref() {
        // NOTE: This it is not a good idea to use an address as a unique ID.
        // reference: https://stackoverflow.com/questions/72148631/how-can-i-hash-by-a-raw-pointer
        let health_check = HealthCheck::new("test");

        let manual0 = Manual::new("test_probe0", true);
        assert!(health_check.insert(Box::new(FFIProbe::from(manual0))));

        let manual1 = Manual::new("test_probe0", true);
        assert!(health_check.insert(Box::new(FFIProbe::from(manual1))));
    }

    #[tokio::test]
    async fn test_health_check_reply() {
        let health_check = HealthCheck::new("test");
        let probe = Manual::new("test_probe", true);
        assert!(
            health_check
                .async_insert(FFIProbe::from(probe.clone()))
                .await
        );

        let replies = health_check.check_verbose(Instant::now()).await;
        let details = replies.details.unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].name, "test_probe");
        assert!(details[0].valid);
        let check = health_check.check(Instant::now());
        assert!(check.await.valid);
    }

    #[tokio::test]
    async fn test_health_check_reply_fail() {
        let health_check = HealthCheck::new("test");
        let probe = Manual::new("test_probe", false);
        assert!(
            health_check
                .async_insert(FFIProbe::from(probe.clone()))
                .await
        );
        let replies = health_check.check_verbose(Instant::now()).await;
        let details = replies.details.unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].name, "test_probe");
        assert!(!details[0].valid);
    }

    #[tokio::test]
    async fn test_health_check_reply_multiple() {
        let health_check = HealthCheck::new("test");
        let probe0 = Manual::new("test_probe", true);

        assert!(
            health_check
                .async_insert(FFIProbe::from(probe0.clone()))
                .await
        );

        let probe1 = Manual::new("test_probe2", true);
        assert!(
            health_check
                .async_insert(FFIProbe::from(probe1.clone()))
                .await
        );

        let replies = health_check.check_verbose(Instant::now()).await;
        assert_eq!(replies.details.unwrap().len(), 2);
        let check = health_check.check(Instant::now());
        assert!(check.await.valid);
    }

    #[tokio::test]
    async fn test_health_check_reply_failures() {
        let health_check = HealthCheck::new("test");
        let probe0 = Manual::new("test_probe", true);

        assert!(
            health_check
                .async_insert(FFIProbe::from(probe0.clone()))
                .await
        );

        let probe1 = Manual::new("test_probe2", false);
        assert!(
            health_check
                .async_insert(FFIProbe::from(probe1.clone()))
                .await
        );

        let replies = health_check.check_verbose(Instant::now()).await;
        assert_eq!(replies.details.unwrap().len(), 2);
        let check = health_check.check(Instant::now());
        assert!(!check.await.valid);
    }
}
