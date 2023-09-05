use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use crate::health_check::{HealthCheck, HealthCheckResult};

// What is the ideal interface for the HealthCheck.
// Create the healthCheck object. Then use that to add/remove to probes.

#[derive(Debug)]
struct KickInner {
    latest: Instant,
    margin: Duration,
}

#[derive(Debug, Clone)]
pub struct HealthKick {
    /// Name of the alive check for human reading
    pub name: String,
    /// An object representing the actual content behind an Arc<Mutex>>
    inner: Arc<Mutex<KickInner>>,
}

impl HealthKick {
    /// Create an alive kicked object providing name and duration of time before triggering failure
    pub fn new<S: Into<String>>(name: S, margin: Duration) -> Self {
        Self {
            name: name.into(),
            inner: Arc::new(Mutex::new(KickInner {
                latest: Instant::now() - margin, // Set earlier than margin so check will eval to false
                margin,
            })),
        }
    }
    fn get_inner(&self) -> std::sync::MutexGuard<KickInner> {
        self.inner.lock().unwrap()
    }
    pub fn kick(&self) {
        self.get_inner().latest = Instant::now();
    }
}

// impl Eq for HealthCheck {}

impl PartialEq<dyn HealthCheck> for dyn HealthCheck {
    fn eq(&self, other: &dyn HealthCheck) -> bool {
        println!("IM IN PartialEq");
        println!("Comparing {} and {}", self.get_name(), other.get_name());
        self.get_name() == other.get_name()
    }
}

impl HealthCheck for HealthKick {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn check(&self, time: Instant) -> HealthCheckResult {
        let me = self.get_inner();
        HealthCheckResult {
            name: &self.name,
            valid: me.latest + me.margin >= time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn kick_create_and_destroy() {
        let hc0 = HealthKick::new("apple", Duration::from_secs(10));

        let orig = hc0.get_inner().latest;

        hc0.kick();

        assert!(hc0.get_inner().latest > orig);
    }

    #[test]
    fn kick_clone() {
        let hc0 = HealthKick::new("apple", Duration::from_secs(10));

        let hc1 = hc0.clone();

        hc0.kick();
        let orig = hc1.get_inner().latest;
        assert!(orig == hc0.get_inner().latest);

        drop(hc0);
    }

    #[test]
    fn kick_check() {
        let hc0 = HealthKick::new("apple", Duration::from_secs(10));

        let check = hc0.check(Instant::now());

        println!("check = {}", check);
    }

    #[test]
    fn kick_eq() {
        let hc0 = HealthKick::new("apple", Duration::from_secs(10));

        let hc0_clone = hc0.clone();

        // let hc0_boxed = HealthCheckWrapper(Box::new(hc0.clone()));
        // let hc0_clone_boxed: HealthCheckWrapper = HealthCheckWrapper(Box::new(hc0_clone.clone()));

        // assert!(hc0_boxed == hc0_clone_boxed);
    }

    // #[test]
    // fn kick_list_of_health() {
    //     let mut myList: Vec<Box<dyn HealthCheck>> = Vec::new();

    //     let mut myHash: HashSet<Box<dyn HealthCheck>> = HashSet::new();

    //     let hc0 = HealthKick::new("apple", Duration::from_secs(10));

    //     myList.push(Box::new(hc0.clone()));

    //     myHash.insert(Box::new(hc0.clone());

    // )
}
