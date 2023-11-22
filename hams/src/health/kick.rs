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
    use std::thread;

    use libc::sleep;

    use crate::health::health_probe::{HealthProbe, HpW};

    use super::*;

    #[test]
    fn kick() {
        println!("Checking kick");

        let now_precreate = Instant::now();
        // Need to introduce the forced sleep to Guarantee this test to work. OSX Arm is NOT TIER 1. Therefore Instant::now() is NOT GUARANTED to be monatonic incrasing.
        // In testing is it observed that we get identical values sometimes.
        thread::sleep(Duration::from_millis(10));
        let mut kick = Kick::new("mykick", Duration::from_secs(30));
        thread::sleep(Duration::from_millis(10));
        let now_postcreate = Instant::now();
        assert!(kick.latest > now_precreate);
        println!(
            "precreate = {:?} kick = {:?}, postcreate = {:?}",
            now_precreate, kick.latest, now_postcreate
        );
        assert!(
            now_postcreate > kick.latest,
            "post_create was not greater than latest"
        );

        assert!(kick.check(now_precreate + Duration::from_secs(30)));
        assert!(!kick.check(now_postcreate + Duration::from_secs(30)));

        kick.kick();
        assert!(kick.check(now_postcreate + Duration::from_secs(30)));

        let now_postkick = Instant::now();

        assert!(!kick.check(now_postkick + Duration::from_secs(30)));

        let kpw = HpW::new(kick);

        assert!(!kpw.check(now_postkick + Duration::from_secs(30)));
        kpw.inner_through_lock().kick();
        assert!(kpw.check(now_postkick + Duration::from_secs(30)));
    }
}
