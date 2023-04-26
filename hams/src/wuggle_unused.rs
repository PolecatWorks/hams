use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// creating a appliation facing interface for building liveness checks
///
/// What is a good API for this?
/// 1. Easy create the check and assign it to the liveness check
/// 2. abilithy to deregister a check from the liveness check
/// 3. simplify the usage pattern hiding teh Arc/Mutex
/// 4. Can we use Atomics to simplify the Mutex parts
use serde::Serialize;
use std::fmt::Debug;

/// Detail structure for replies from ready and alive
#[derive(Serialize, Debug)]
pub struct HealthCheckResult<'a> {
    pub name: &'a str,
    pub valid: bool,
}

pub trait HealthCheck {
    /// Return name
    ///
    /// TODO: can this return a &str so we do not trigger a clone of the value.
    /// However it is behind a MutexGuard so we can only get a reference to the value
    /// mutexc guard exists so cant see easy way to solve
    fn get_name(&self) -> &str;
    fn check(&self, time: Instant) -> HealthCheckResult;
}

/// Implement the alive check which will fail if the service has not been triggered within the margin
#[derive(Debug)]
pub struct AliveCheckInner {
    latest: Instant,
}

#[derive(Debug, Clone)]
pub struct AliveCheckKicked {
    name: String,
    margin: Duration,

    inner: Arc<Mutex<AliveCheckInner>>,
}

impl AliveCheckKicked {
    pub fn new<S: Into<String>>(name: S, margin: Duration) -> Self {
        Self {
            name: name.into(),
            inner: Arc::new(Mutex::new(AliveCheckInner {
                latest: Instant::now(),
            })),
            margin,
        }
    }
    fn get_data(&self) -> std::sync::MutexGuard<AliveCheckInner> {
        self.inner.lock().unwrap()
    }

    pub fn kick(&self) {
        self.get_data().latest = Instant::now();
    }
}

impl HealthCheck for AliveCheckKicked {
    fn check(&self, time: Instant) -> HealthCheckResult {
        let my_data = self.get_data();
        HealthCheckResult {
            name: &self.name,
            valid: my_data.latest + self.margin >= time,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    // use crate::wuggle::{DataManager, InnerData};
    use super::*;

    #[test]
    fn test_real_alive() {
        let health = AliveCheckKicked::new(String::from("hello"), Duration::from_secs(10));

        let orig = health.get_data().latest;

        health.kick();

        assert!(health.get_data().latest > orig);

        println!("all done with {}", health.get_name());
    }

    #[test]
    fn test_check() {
        let health = AliveCheckKicked::new(String::from("hello"), Duration::from_secs(10));

        let reply = health.check(Instant::now());

        println!("reply = {:?}", reply);
    }

    #[test]
    fn test_data_manager() {
        let health = AliveCheckKicked::new(String::from("hello"), Duration::from_secs(10));

        let now = health.get_data().latest;

        // spawn two threads that increment a field in the InnerData struct
        let health_1 = health.clone();

        let handle_1 = thread::spawn(move || {
            let mut data = health_1.get_data();
            data.latest += Duration::from_secs(1);
        });

        let health_2 = health.clone();
        let handle_2 = thread::spawn(move || {
            let mut data = health_2.get_data();
            data.latest += Duration::from_secs(2);
        });

        handle_1.join().unwrap();
        handle_2.join().unwrap();

        // check that the field has been incremented by two
        let data = health.get_data();
        assert_eq!(data.latest, now + Duration::from_secs(3));
    }
}
