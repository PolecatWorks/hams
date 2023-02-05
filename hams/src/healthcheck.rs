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

impl AliveCheck {
    pub fn new(name: String, margin: Duration) -> Self {
        Self {
            name,
            latest: Instant::now(),
            margin,
        }
    }

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
    use std::time::Duration;

    use crate::healthcheck::{AliveCheck, HealthCheck, HealthCheckResult};

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
