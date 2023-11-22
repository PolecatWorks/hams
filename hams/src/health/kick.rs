use std::time::{Duration, Instant};

use super::HealthProbeInner;

#[derive(Debug, Hash, PartialEq)]
pub struct Kick {
    name: String,
    latest: Instant,
    margin: Duration,
}
impl Eq for Kick {}

impl Kick {
    pub fn new<S: Into<String>>(name: S, margin: Duration) -> Kick {
        Self {
            name: name.into(),
            latest: Instant::now(),
            margin,
        }
    }
    pub fn kick(&mut self) {
        self.latest = Instant::now();
    }
}

impl HealthProbeInner for Kick {
    fn name(&self) -> &str {
        &self.name
    }

    fn name_owned(&self) -> String {
        self.name.clone()
    }

    fn check_reply(&self, time: Instant) -> super::HealthProbeResult {
        super::HealthProbeResult {
            name: &self.name,
            valid: self.check(time),
        }
    }

    fn check(&self, time: Instant) -> bool {
        self.latest + self.margin >= time
    }
}

#[cfg(test)]
mod tests {
    use crate::health::health_probe::{HealthProbe, HpW};

    use super::*;

    #[test]
    fn kick() {
        println!("Checking kick");

        let now_precreate = Instant::now();
        let mut kick = Kick::new("mykick", Duration::from_secs(30));
        let now_postcreate = Instant::now();

        assert!(kick.check(now_precreate + Duration::from_secs(30)));
        assert!(!kick.check(now_postcreate + Duration::from_secs(30)));

        kick.kick();
        let now_postkick = Instant::now();
        assert!(kick.check(now_postcreate + Duration::from_secs(30)));

        let kpw = HpW::new(kick);

        assert!(!kpw.check(now_postkick + Duration::from_secs(30)));
        kpw.inner_through_lock().kick();
        assert!(kpw.check(now_postkick + Duration::from_secs(30)));
    }
}
