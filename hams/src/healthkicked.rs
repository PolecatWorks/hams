//! Specific Kick style for health Checks

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::healthcheck::{HealthCheck, HealthCheckResult};

#[derive(Debug)]
struct AliveCheckKickedInner {
    latest: Instant,
    margin: Duration,
}

/// Implement the alive check which will fail if the service has not been triggered within the margin
#[derive(Debug, Clone)]
pub struct AliveCheckKicked {
    /// Name of the alive check for human reading
    pub name: String,
    /// An object representing the actual content behind an Arc<Mutex>>
    inner: Arc<Mutex<AliveCheckKickedInner>>,
}

/// Create an alive check that takes a margin and fails when the time has not been kept up to date within the margin
impl AliveCheckKicked {
    /// Create an alive kicked object providing name and duration of time before triggering failure
    pub fn new<S: Into<String>>(name: S, margin: Duration) -> Self {
        Self {
            name: name.into(),
            inner: Arc::new(Mutex::new(AliveCheckKickedInner {
                latest: Instant::now(),
                margin,
            })),
        }
    }
    fn get_inner(&self) -> std::sync::MutexGuard<AliveCheckKickedInner> {
        self.inner.lock().unwrap()
    }
    /// Update the latest time record
    pub fn kick(&self) {
        // info!("kickinHg {}", self.name);
        self.get_inner().latest = Instant::now();
        // info!("did kick on {}", self.name);
    }
}

impl HealthCheck for AliveCheckKicked {
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
    use std::thread;

    use super::*;

    // /// Test the API of alive to confirm check and kick
    // #[test]
    // fn alive() {
    //     println!("OK");

    //     let mut alive = I {
    //         name: "apple".to_owned(),
    //         count: 0,
    //     };

    //     let alive_ok = alive.check(alive.get_inner().latest + Duration::from_secs(1));
    //     assert_eq!(
    //         HealthCheckResult {
    //             name: "apple",
    //             valid: true
    //         },
    //         alive_ok
    //     );

    //     let alive_margin = alive.check(alive.get_inner().latest + Duration::from_secs(10));
    //     assert_eq!(
    //         HealthCheckResult {
    //             name: "apple",
    //             valid: true
    //         },
    //         alive_margin
    //     );

    //     let alive_fail = alive.check(alive.get_inner().latest + Duration::from_secs(11));
    //     assert_eq!(
    //         HealthCheckResult {
    //             name: "apple",
    //             valid: false
    //         },
    //         alive_fail
    //     );

    //     let create_time = alive.get_inner().latest;

    //     alive.kick();

    //     assert!(alive.get_inner().latest > create_time);
    // }

    #[test]
    fn alive_practical_use() {
        let health = AliveCheckKicked::new("hello", Duration::from_secs(10));

        let orig = health.get_inner().latest;

        health.kick();

        assert!(health.get_inner().latest > orig);

        println!("all done with {}", health.get_name());
    }

    #[test]
    fn test_clone() {
        let health = AliveCheckKicked::new("hello", Duration::from_secs(10));

        let health2 = health.clone();

        println!("{:?} => {:?}", health, health2);
        drop(health);
        println!("=> {:?}", health2);
    }

    #[test]
    fn test_check() {
        let health = AliveCheckKicked::new("hello", Duration::from_secs(10));

        let reply = health.check(Instant::now());

        println!("reply = {:?}", reply);
    }

    #[test]
    fn test_data_manager() {
        let health = AliveCheckKicked::new("hello", Duration::from_secs(10));

        let now = health.get_inner().latest;

        // spawn two threads that increment a field in the InnerData struct
        let health_1 = health.clone();

        let handle_1 = thread::spawn(move || {
            health_1.get_inner().latest += Duration::from_secs(1);
            // let mut data = health_1.get_instant();
            // data.latest += Duration::from_secs(1);
        });

        let health_2 = health.clone();
        let handle_2 = thread::spawn(move || {
            health_2.get_inner().latest += Duration::from_secs(2);
            // let mut data = health_2.get_data();
            // data.latest += Duration::from_secs(2);
        });

        handle_1.join().unwrap();
        handle_2.join().unwrap();

        // check that the field has been incremented by two
        let data = health.get_inner().latest;
        assert_eq!(data, now + Duration::from_secs(3));
    }
}
