use crate::probe::HealthProbe;
use libc::time_t;
use std::ffi::{c_char, CString};
use std::time::{Duration, SystemTime};

use super::BoxedHealthProbe;

/// A liveness check that automatically fails when the timer has not been reset before
/// the duration. Equivalent of a dead mans handle.
#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Kick {
    name: String,
    /// The time of the last kick in seconds since UNIX_EPOCH
    latest: time_t,
    margin: Duration,
}

impl Kick {
    /// Create a new Kick probe with the given name and margin
    pub fn new<S: Into<String>>(name: S, margin: Duration) -> Self {
        Self {
            name: name.into(),
            latest: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .try_into()
                .unwrap(),
            margin,
        }
    }

    /// Reset the timer
    pub fn kick(&mut self) {
        self.latest = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap();
    }

    /// Return a BoxedHealthProbe for the probe
    pub fn boxed_probe(&self) -> BoxedHealthProbe<'static> {
        BoxedHealthProbe::new(self.clone())
    }
}

impl HealthProbe for Kick {
    #[doc = "Name of the probe"]
    fn name(&self) -> *mut c_char {
        CString::new(self.name.clone()).unwrap().into_raw()
    }

    fn check(&self, time: time_t) -> i32 {
        let duration_secs: i64 = self.margin.as_secs().try_into().unwrap();

        (time < self.latest + duration_secs) as i32
    }
}

#[cfg(test)]
mod tests {

    use std::time::UNIX_EPOCH;

    use super::*;

    #[test]
    fn test_kick() {
        let mut probe = Kick::new("test", Duration::from_secs(1));

        // let time_now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().try_into().unwrap();

        let time_now = probe.latest;

        assert!(probe.check(time_now) == 1);
        probe.kick();
        assert!(probe.check(time_now) == 1);
        //No need to sleep, we can just check the time
        assert!(probe.check(time_now + 2) == 0);
    }

    // Test the boxed_probe method
    #[test]
    fn test_boxed_probe() {
        let probe = Kick::new("test", Duration::from_secs(1));
        let boxed_probe = probe.boxed_probe();
        assert_eq!(
            unsafe { CString::from_raw(boxed_probe.name()) }
                .into_string()
                .expect("Converted CString"),
            "test"
        );

        let time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap();

        assert!(boxed_probe.check(time_now) == 1);
    }
}
