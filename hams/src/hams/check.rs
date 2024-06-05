use std::{collections::HashSet, sync::Arc};

use futures::future::join_all;
use serde::Serialize;
use tokio::{sync::Mutex, time::Instant};

use crate::probe::HealthProbe;

use crate::probe::{AsyncHealthProbe, HealthProbeResult};

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

    /// Insert a probe into the HealthCheck using an async safe lock
    pub(crate) async fn insert_async(&self, probe: Box<dyn AsyncHealthProbe + 'static>) -> bool {
        self.probes.lock().await.insert(probe)
    }

    /// Remove a probe from the HealthCheck
    pub(crate) fn remove(&self, probe: &Box<dyn AsyncHealthProbe>) -> bool {
        self.probes.blocking_lock().remove(probe)
    }

    /// Remove a probe from the HealthCheck using an async safe lock
    pub(crate) async fn remove_async(&self, probe: &Box<dyn AsyncHealthProbe>) -> bool {
        self.probes.lock().await.remove(probe)
    }

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
    pub(super) fn len(&self) -> usize {
        self.probes.blocking_lock().len()
    }
    async fn len_async(&self) -> usize {
        self.probes.lock().await.len()
    }
}

// #[cfg(test)]
// pub(crate) async fn blocking_probe_remove<T: HealthProbe + Clone + 'static>(
//     check: &HealthCheck,
//     probe: &T,
// ) -> bool {
//     let check = check.clone();
//     let probe = (*probe).clone();

//     task::spawn_blocking(move || check.remove(&probe.into()))
//         .await
//         .expect("removed alive")
// }

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::probe::{kick::Kick, manual::Manual, FFIProbe};

    /// Test insert on a health check using manual
    #[test]
    fn test_health_check_insert() {
        let check = HealthCheck::new("test");
        let manual0 = Manual::new("test_probe0", true);
        let manual1 = Manual::new("test_probe1", true);

        assert_eq!(check.len(), 0);
        assert!(check.insert(FFIProbe::from(manual0.clone()).into()));
        assert_eq!(check.len(), 1);
        assert!(!check.insert(FFIProbe::from(manual0.clone()).into()));
        assert_eq!(check.len(), 1);

        assert!(check.insert(FFIProbe::from(manual1.clone()).into()));
        assert_eq!(check.len(), 2);
    }

    /// Test insert_async on a health check using manual
    #[tokio::test]
    async fn test_health_check_insert_async() {
        let check = HealthCheck::new("test");
        let manual0 = Manual::new("test_probe0", true);
        let manual1 = Manual::new("test_probe1", true);

        assert_eq!(check.len_async().await, 0);
        assert!(
            check
                .insert_async(FFIProbe::from(manual0.clone()).into())
                .await
        );
        assert_eq!(check.len_async().await, 1);
        assert!(
            !check
                .insert_async(FFIProbe::from(manual0.clone()).into())
                .await
        );
        assert_eq!(check.len_async().await, 1);

        assert!(
            check
                .insert_async(FFIProbe::from(manual1.clone()).into())
                .await
        );
        assert_eq!(check.len_async().await, 2);
    }

    /// Test we can insert different types of probes into the health check
    #[test]
    fn test_health_check_insert_different_types() {
        let check = HealthCheck::new("test");
        let manual0 = Manual::new("test_probe0", true);
        let kick0 = Kick::new("test_probe1", Duration::from_secs(1));

        assert_eq!(check.len(), 0);
        assert!(check.insert(FFIProbe::from(manual0.clone()).into()));
        assert_eq!(check.len(), 1);
        assert!(!check.insert(FFIProbe::from(manual0.clone()).into()));
        assert_eq!(check.len(), 1);

        assert!(check.insert(FFIProbe::from(kick0.clone()).into()));
        assert_eq!(check.len(), 2);
    }

    /// Test remove on a health check using manual
    #[test]
    fn test_health_check_remove() {
        let check = HealthCheck::new("test");
        let manual0 = Manual::new("test_probe0", true);
        let manual1 = Manual::new("test_probe1", true);

        assert_eq!(check.len(), 0);
        assert!(!check.remove(&(FFIProbe::from(manual0.clone()).into())));
        assert_eq!(check.len(), 0);

        check.insert(FFIProbe::from(manual0.clone()).into());
        check.insert(FFIProbe::from(manual1.clone()).into());
        assert_eq!(check.len(), 2);

        assert!(check.remove(&(FFIProbe::from(manual0.clone()).into())));
        assert_eq!(check.len(), 1);
        assert!(!check.remove(&(FFIProbe::from(manual0.clone()).into())));
        assert_eq!(check.len(), 1);

        assert!(check.remove(&(FFIProbe::from(manual1.clone()).into())));
        assert_eq!(check.len(), 0);
    }

    /// Test remove_async on a health check using manual
    #[tokio::test]
    async fn test_health_check_remove_async() {
        let check = HealthCheck::new("test");
        let manual0 = Manual::new("test_probe0", true);
        let manual1 = Manual::new("test_probe1", true);

        assert_eq!(check.len_async().await, 0);
        assert!(
            !check
                .remove_async(&(FFIProbe::from(manual0.clone()).into()))
                .await
        );
        assert_eq!(check.len_async().await, 0);

        check
            .insert_async(FFIProbe::from(manual0.clone()).into())
            .await;
        check
            .insert_async(FFIProbe::from(manual1.clone()).into())
            .await;
        assert_eq!(check.len_async().await, 2);

        assert!(
            check
                .remove_async(&(FFIProbe::from(manual0.clone()).into()))
                .await
        );
        assert_eq!(check.len_async().await, 1);
        assert!(
            !check
                .remove_async(&(FFIProbe::from(manual0.clone()).into()))
                .await
        );
        assert_eq!(check.len_async().await, 1);

        assert!(
            check
                .remove_async(&(FFIProbe::from(manual1.clone()).into()))
                .await
        );
        assert_eq!(check.len_async().await, 0);
    }

    /// Test check on a health check using manual
    #[tokio::test]
    async fn test_health_check_check() {
        let check = HealthCheck::new("test");
        let mut manual0 = Manual::new("test_probe0", true);
        let manual1 = Manual::new("test_probe1", true);

        let replies = check.check(Instant::now()).await;
        assert!(replies.valid);
        assert!(replies.details.is_none());

        let replies = check.check_verbose(Instant::now()).await;
        assert!(replies.valid);
        assert_eq!(replies.details.unwrap().len(), 0);

        assert!(
            check
                .insert_async(FFIProbe::from(manual0.clone()).into())
                .await
        );
        assert!(
            check
                .insert_async(FFIProbe::from(manual1.clone()).into())
                .await
        );

        let replies = check.check(Instant::now()).await;
        assert!(replies.valid);
        assert!(replies.details.is_none());

        let replies = check.check_verbose(Instant::now()).await;
        assert!(replies.valid);
        assert_eq!(replies.details.unwrap().len(), 2);

        manual0.disable();
        let replies = check.check(Instant::now()).await;
        assert!(!replies.valid);
        // assert_eq!(replies.details.unwrap().len(), 2);
    }

    /// Test check_verbose to confirm names match to probes
    #[tokio::test]
    async fn test_health_check_check_verbose() {
        let check = HealthCheck::new("test");
        let manual0 = Manual::new("test_probe0", true);
        let manual1 = Manual::new("test_probe1", true);

        assert!(
            check
                .insert_async(FFIProbe::from(manual0.clone()).into())
                .await
        );
        assert!(
            check
                .insert_async(FFIProbe::from(manual1.clone()).into())
                .await
        );

        let replies = check.check_verbose(Instant::now()).await;
        assert_eq!(replies.details.as_ref().unwrap().len(), 2);
        let names = replies
            .details
            .as_ref()
            .unwrap()
            .iter()
            .map(|probe| probe.name.clone())
            .collect::<Vec<String>>();
        assert!(names.contains(&"test_probe0".to_string()));
        assert!(names.contains(&"test_probe1".to_string()));
    }
}
