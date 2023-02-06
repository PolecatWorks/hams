use std::time::{Duration, Instant};

use serde::Serialize;

/// Detail structure for replies from ready and alive
#[derive(Serialize, Debug, PartialEq)]
pub struct HealthCheckResult {
    name: String,
    valid: bool,
}

/// Trait to define the health check functionality
pub trait HealthCheck {
    fn get_name(&self) -> &str;
    fn check(&self, time: Instant) -> HealthCheckResult;
}

/// Implement the alive check which will fail if the service has not been triggered within the margin
#[derive(Debug)]
pub struct AliveCheck {
    name: String,
    latest: Instant,
    margin: Duration,
}

/// Create an alive check that takes a margin and fails when the time has not been kept up to date within the margin
impl AliveCheck {
    pub fn new(name: String, margin: Duration) -> Self {
        Self {
            name,
            latest: Instant::now(),
            margin,
        }
    }

    /// Update the latest time record
    pub fn kick(&mut self) {
        self.latest = Instant::now();
    }
}

impl HealthCheck for AliveCheck {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn check(&self, time: Instant) -> HealthCheckResult {
        HealthCheckResult {
            name: self.name.clone(),
            valid: self.latest + self.margin >= time,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex, RwLock},
        thread::spawn,
        time::{Duration, Instant},
    };

    use crate::healthcheck::{AliveCheck, HealthCheck, HealthCheckResult};

    /// Create the alive check and in another thread create reference the alive and allow it to be used for generating an alive response
    #[test]
    fn threading_mutex() {
        let my_health_orig = Arc::new(Mutex::new(AliveCheck::new(
            "apple".to_string(),
            Duration::from_secs(10),
        )));

        let my_health = my_health_orig.clone();
        my_health_orig.lock().unwrap().kick();
        my_health.lock().unwrap().kick();

        let jh = spawn(move || {
            println!("from spawned");
            my_health.lock().unwrap().kick();
        });

        my_health_orig.lock().unwrap().kick();

        println!("in main thread");

        jh.join().unwrap();

        println!("Complete test after join");
    }

    /// Create the alive check and in another thread create reference the alive and allow it to be used for generating an alive response
    #[test]
    fn threading_rwlock() {
        let my_health_orig = Arc::new(RwLock::new(AliveCheck::new(
            "apple".to_string(),
            Duration::from_secs(10),
        )));

        let my_health = my_health_orig.clone();
        let my_check_reply = my_health.read().unwrap().check(Instant::now());

        my_health_orig.write().unwrap().kick();
        my_health.write().unwrap().kick();

        let jh = spawn(move || {
            println!("from spawned");
            my_health.write().unwrap().kick();
        });

        my_health_orig.write().unwrap().kick();

        println!("in main thread");

        jh.join().unwrap();

        println!("Complete test after join");
    }

    /// Test the API of alive to confirm check and kick
    #[test]
    fn alive() {
        println!("OK");

        let mut alive = AliveCheck::new("apple".to_string(), Duration::from_secs(10));

        let alive_ok = alive.check(alive.latest + Duration::from_secs(1));
        assert_eq!(
            HealthCheckResult {
                name: "apple".to_string(),
                valid: true
            },
            alive_ok
        );

        let alive_margin = alive.check(alive.latest + Duration::from_secs(10));
        assert_eq!(
            HealthCheckResult {
                name: "apple".to_string(),
                valid: true
            },
            alive_margin
        );

        let alive_fail = alive.check(alive.latest + Duration::from_secs(11));
        assert_eq!(
            HealthCheckResult {
                name: "apple".to_string(),
                valid: false
            },
            alive_fail
        );

        let create_time = alive.latest;

        alive.kick();

        assert!(alive.latest > create_time);
    }
}
