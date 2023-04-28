//! HealthChecks in Hams

use std::time::Instant;

use serde::Serialize;
use std::fmt::Debug;

use std::hash::{Hash, Hasher};

/// Result from whole HealthSystem
#[derive(Debug, Serialize)]
pub struct HealthSystemResult<'a> {
    pub(crate) name: &'a str,
    pub(crate) valid: bool,
    pub(crate) detail: Vec<HealthCheckResult<'a>>,
}

/// Detail structure for replies from ready and alive
#[derive(Serialize, Debug, PartialEq)]
pub struct HealthCheckResult<'a> {
    pub name: &'a str,
    pub valid: bool,
}

/// Trait to define the health check functionality
pub trait HealthCheck: Debug + Send {
    fn get_name(&self) -> &str;
    fn check(&self, time: Instant) -> HealthCheckResult;
}

#[derive(Debug)]
pub struct HealthCheckWrapper(pub Box<dyn HealthCheck>);

impl HealthCheckWrapper {
    pub fn get_name(&self) -> &str {
        self.0.get_name()
    }
    pub fn check(&self, time: Instant) -> HealthCheckResult {
        self.0.check(time)
    }
}
impl Eq for HealthCheckWrapper {}

impl PartialEq for HealthCheckWrapper {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0.as_ref(), other.0.as_ref())
    }
}

impl Hash for HealthCheckWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0.as_ref(), state);
    }
}

/// Comparison function for equality of two HealthChecks
/// We can only use the methods available in the HealthCheck for
/// implementation of equality so that limits us to name for comparison
impl PartialEq<dyn HealthCheck> for dyn HealthCheck {
    fn eq(&self, other: &dyn HealthCheck) -> bool {
        println!("IM IN PartialEq");
        println!("Comparing {} and {}", self.get_name(), other.get_name());
        self.get_name() == other.get_name()
    }
}

impl Hash for dyn HealthCheck {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_name().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{Arc, Mutex, RwLock},
        thread::{self, spawn},
        time::{Duration, Instant},
    };

    #[derive(Debug, Clone)]
    struct I {
        name: String,
        count: i64,
    }

    impl HealthCheck for I {
        fn get_name(&self) -> &str {
            println!("HealthCheck for I {}", self.name);
            &self.name
        }

        fn check(&self, time: std::time::Instant) -> HealthCheckResult {
            todo!()
        }
    }

    #[test]
    fn compare_hc_via_trait() {
        let hc0 = I {
            name: "Test0".to_owned(),
            count: 0,
        };
        let hc1 = I {
            name: "Test1".to_owned(),
            count: 0,
        };

        // assert_ne!(hc0, hc1);

        let hc2 = hc0.clone();
        // assert_eq!(hc0, hc2);

        assert_eq!(&hc0 as &dyn HealthCheck, &hc2 as &dyn HealthCheck);
        assert_ne!(&hc0 as &dyn HealthCheck, &hc1 as &dyn HealthCheck);

        let hclist: Vec<Box<dyn HealthCheck>> = vec![Box::new(hc0), Box::new(hc1), Box::new(hc2)];

        let ben = *hclist[0] == *hclist[1];

        assert_eq!(*hclist[0], *hclist[2]);
        assert_ne!(*hclist[0], *hclist[1]);

        let hc0_ref: *const dyn HealthCheck = hclist[0].as_ref();
        let hc1_ref: *const dyn HealthCheck = hclist[1].as_ref();
        let hc2_ref: *const dyn HealthCheck = hclist[2].as_ref();

        assert_ne!(hc0_ref, hc1_ref);
        assert_ne!(hc0_ref, hc2_ref);
    }

    // Create the alive check and in another thread create reference the alive and allow it to be used for generating an alive response
    // #[test]
    // fn threading_mutex() {
    //     let my_health_orig = Arc::new(Mutex::new(AliveCheckKicked::new(
    //         "apple".to_string(),
    //         Duration::from_secs(10),
    //     )));

    //     let my_health = my_health_orig.clone();
    //     my_health_orig.lock().unwrap().kick();
    //     my_health.lock().unwrap().kick();

    //     let jh = spawn(move || {
    //         println!("from spawned");
    //         my_health.lock().unwrap().kick();
    //     });

    //     my_health_orig.lock().unwrap().kick();

    //     println!("in main thread");

    //     jh.join().unwrap();

    //     println!("Complete test after join");
    // }

    // Create the alive check and in another thread create reference the alive and allow it to be used for generating an alive response
    // #[test]
    // fn threading_rwlock() {
    //     let my_health_orig = Arc::new(RwLock::new(AliveCheckKicked::new(
    //         "apple".to_string(),
    //         Duration::from_secs(10),
    //     )));

    //     let my_health = my_health_orig.clone();
    //     let my_check_reply = my_health.read().unwrap().check(Instant::now());

    //     my_health_orig.write().unwrap().kick();
    //     my_health.write().unwrap().kick();

    //     let jh = spawn(move || {
    //         println!("from spawned");
    //         my_health.write().unwrap().kick();
    //     });

    //     my_health_orig.write().unwrap().kick();

    //     println!("in main thread");

    //     jh.join().unwrap();

    //     println!("Complete test after join");
    // }
}
