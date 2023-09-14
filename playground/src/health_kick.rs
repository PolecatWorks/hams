use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use log::info;

use crate::{
    error::HamsError,
    health::{Health, HealthCheckResult},
};

// What is the ideal interface for the HealthCheck.
// Create the healthCheck object. Then use that to add/remove to probes.

#[derive(Debug)]
struct KickInner {
    latest: Instant,
    margin: Duration,
    previous: bool,
    /// Name of the alive check for human reading
    pub name: String,
}

impl Drop for KickInner {
    fn drop(&mut self) {
        println!("Dropping Inner kick {}", self.name);
    }
}

#[derive(Debug, Clone)]
pub struct HealthKick {
    /// An object representing the actual content behind an Arc<Mutex>>
    inner: Arc<Mutex<KickInner>>,
}

impl HealthKick {
    /// Create an alive kicked object providing name and duration of time before triggering failure
    pub fn new<S: Into<String>>(name: S, margin: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(KickInner {
                name: name.into(),
                latest: Instant::now() - margin, // Set earlier than margin so check will eval to false
                margin,
                previous: false,
            })),
        }
    }
    fn get_inner(&self) -> std::sync::MutexGuard<KickInner> {
        self.inner.lock().unwrap()
    }

    // Update the Health Check timestamp to indicate it is still alive
    pub fn kick(&self) {
        self.get_inner().latest = Instant::now();
    }
}

// impl Eq for HealthCheck {}

// impl PartialEq<dyn HealthCheck> for dyn HealthCheck {
//     fn eq(&self, other: &dyn HealthCheck) -> bool {
//         println!("IM IN PartialEq");
//         println!("Comparing {} and {}", self.get_name(), other.get_name());
//         self.get_name() == other.get_name()
//     }
// }

impl Health for HealthKick {
    fn check(&self, time: Instant) -> Result<HealthCheckResult, HamsError> {
        let mut me = self.get_inner();
        let previous = me.previous;
        let valid = me.latest + me.margin >= time;

        if previous != valid {
            info!("Health: {} changed to {}", me.name, valid);
        }
        me.previous = valid;
        Ok(HealthCheckResult {
            name: me.name.clone(),
            valid,
        })
    }

    // fn name(&self) -> Result<String, crate::error::HamsError> {
    //     let me = self.get_inner();
    //     Ok(me.name.clone())
    // }

    // fn previous(&self) -> Result<bool, crate::error::HamsError> {
    //     let me = self.get_inner();
    //     Ok(me.previous)
    // }
}

impl Drop for HealthKick {
    fn drop(&mut self) {
        let me = self.get_inner();
        println!("Dropping my kick {}", me.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let check = hc0.check(Instant::now()).unwrap();

        println!("check = {}", check);
    }

    #[test]
    fn kick_eq() {
        let hc0 = HealthKick::new("apple", Duration::from_secs(10));

        let _hc0_clone = hc0;

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
