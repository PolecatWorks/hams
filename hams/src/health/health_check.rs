use std::{
    collections::HashSet,
    fmt::Display,
    sync::{Arc, Mutex},
    time::Instant,
};

use log::info;
use serde::Serialize;

use super::health_probe::HealthProbe;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct HealthCheckReply {
    pub(crate) name: String,
    pub(crate) valid: bool,
}

impl<'a> warp::Reply for HealthCheckReply {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::with_status(
            warp::reply::json(&self),
            if self.valid {
                warp::http::StatusCode::OK
            } else {
                warp::http::StatusCode::NOT_ACCEPTABLE
            },
        )
        .into_response()
    }
}

/// Represent the [HealthCheck] which collects [HealthProbe]s and replies to a check with a struct that can use returned as
/// a kubernetes readyness or liveness probe
///
/// Problems when creating this as the 'static lifetime required for dyn causes issues making this struct Send safe
/// This seems to capture the issue: https://users.rust-lang.org/t/why-this-impl-type-lifetime-may-not-live-long-enough/67855
#[derive(Debug, Clone)]
pub struct HealthCheck {
    name: String,
    probes: Arc<Mutex<HashSet<Box<dyn HealthProbe>>>>,
}

impl<'a> HealthCheck {
    pub fn new<S: Into<String>>(name: S) -> HealthCheck {
        HealthCheck {
            name: name.into(),
            probes: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Check the status of all probes and return state of check
    ///
    /// State is true if all are true else false
    pub fn check(&self, now: Instant) -> bool {
        self.probes
            .lock()
            .unwrap()
            .iter()
            .all(|probe| probe.check(now))
    }

    pub fn check_reply(&self, now: Instant) -> HealthCheckReply {
        let valid = self.check(now);
        if !valid {
            info!("Invalid check: {}", self);
        }
        HealthCheckReply {
            name: self.name.clone(),
            valid,
        }
    }

    pub fn insert_boxed(&self, newval: Box<dyn HealthProbe>) -> bool {
        self.probes.lock().unwrap().insert(newval)
    }

    pub fn remove_boxed(&self, value: Box<dyn HealthProbe>) -> bool {
        self.probes.lock().unwrap().remove(&value)
    }
}

impl<'a> Display for HealthCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.name)?;
        // TODO: Rewrite this to have time provided outside the function and pass down to probes

        let mut first = true;

        self.probes
            .lock()
            .unwrap()
            .iter()
            .try_fold((), |result, check| {
                if first {
                    first = false;
                    write!(f, "{:?}", check)
                } else {
                    write!(f, ",{:?}", check)
                }
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::health::{
        health_probe::{HealthProbeInner, HpW},
        HealthProbeResult,
    };

    use super::*;

    #[test]
    fn check_check() {
        #[derive(Debug, Hash, PartialEq)]
        struct TestInner {
            name: String,
            value: bool,
        }
        impl Eq for TestInner {}

        impl TestInner {
            pub fn set(&mut self) {
                self.value = true
            }
            pub fn clear(&mut self) {
                self.value = false
            }
            pub fn toggle(&mut self) {
                self.value = !self.value
            }
        }
        impl HealthProbeInner for TestInner {
            fn name(&self) -> &str {
                &self.name
            }

            fn check(&self, time: Instant) -> bool {
                self.value
            }

            fn check_reply(&self, time: Instant) -> HealthProbeResult {
                todo!()
            }
        }
        let mut hp_true = TestInner {
            name: "hp_true".to_owned(),
            value: true,
        };
        let mut hp_changing = TestInner {
            name: "hp_changing".to_owned(),
            value: true,
        };

        let hw_true = HpW::new(hp_true);
        let hw_changing = HpW::new(hp_changing);

        let check = HealthCheck::new("ready");

        let now = Instant::now();
        assert!(check.check(now));
        assert_eq!(
            check.check_reply(now),
            HealthCheckReply {
                name: "ready".to_string(),
                valid: true
            }
        );

        check.insert_boxed(Box::new(hw_true));

        let now = Instant::now();
        assert!(check.check(now));
        assert_eq!(
            check.check_reply(now),
            HealthCheckReply {
                name: "ready".to_string(),
                valid: true
            }
        );

        check.insert_boxed(Box::new(hw_changing.clone()));

        let now = Instant::now();
        assert!(check.check(now));
        assert_eq!(
            check.check_reply(now),
            HealthCheckReply {
                name: "ready".to_string(),
                valid: true
            }
        );

        hw_changing.inner_through_lock().toggle();

        let now = Instant::now();
        assert!(!check.check(now));
        assert_eq!(
            check.check_reply(now),
            HealthCheckReply {
                name: "ready".to_string(),
                valid: false
            }
        );

        hw_changing.inner_through_lock().toggle();

        let now = Instant::now();
        assert!(check.check(now));
        assert_eq!(
            check.check_reply(now),
            HealthCheckReply {
                name: "ready".to_string(),
                valid: true
            }
        );
    }

    // #[ignore]
    #[test]
    fn try_out_health_check() {
        #[derive(Debug, Hash, PartialEq)]
        struct InnerI {
            name: String,
            count: u32,
        }
        impl Eq for InnerI {}

        impl InnerI {
            pub fn kick(&mut self) {
                self.count += 1
            }
        }

        impl HealthProbeInner for InnerI {
            fn name(&self) -> &str {
                &self.name
            }

            fn check(&self, time: Instant) -> bool {
                true
            }

            fn check_reply(&self, time: Instant) -> HealthProbeResult {
                todo!()
            }
        }

        let mut hpi0 = InnerI {
            name: "Hpi0".to_owned(),
            count: 3,
        };
        let hw0 = HpW::new(hpi0);

        let check = HealthCheck::new("ready");

        println!("check = {:?}", check);

        assert!(check.insert_boxed(Box::new(hw0.clone())));
        assert!(!check.insert_boxed(Box::new(hw0.clone())));
        println!("check = {:?}", check);
        println!("Check display = {}", check);
        let now = Instant::now();
        assert!(check.check(now));

        assert!(check.remove_boxed(Box::new(hw0.clone())));
        assert!(!check.remove_boxed(Box::new(hw0.clone())));
        println!("check = {:?}", check);
    }
}
