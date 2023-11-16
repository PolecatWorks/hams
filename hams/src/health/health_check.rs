use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Instant,
};

use log::info;
use serde::Serialize;

use super::health_probe::HealthProbe;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct HealthCheckReply<'a> {
    pub(crate) name: &'a str,
    pub(crate) valid: bool,
}

#[derive(Debug)]
struct HealthCheck {
    name: String,
    probes: Arc<Mutex<HashSet<Box<dyn HealthProbe>>>>,
}

impl HealthCheck {
    pub fn new<S: Into<String>>(name: S) -> HealthCheck {
        HealthCheck {
            name: name.into(),
            probes: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Check the status of all probes and return state of check
    ///
    /// State is true if all are true else false
    pub fn check(&self) -> bool {
        let now = Instant::now();
        self.probes
            .lock()
            .unwrap()
            .iter()
            .all(|probe| probe.check(now))
    }

    pub fn check_reply(&self) -> HealthCheckReply {
        let valid = self.check();
        if !valid {
            // TODO: Build a better view of state of failed check
            info!("Invalid: {} = {}", self.name, false)
        }
        HealthCheckReply {
            name: &self.name,
            valid: self.check(),
        }
    }

    pub fn insert_boxed(&self, newval: Box<dyn HealthProbe>) -> bool {
        self.probes.lock().unwrap().insert(newval)
    }

    pub fn remove_boxed(&self, value: Box<dyn HealthProbe>) -> bool {
        self.probes.lock().unwrap().remove(&value)
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
            fn name_owned(&self) -> String {
                self.name.clone()
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

        assert!(check.check());
        assert_eq!(
            check.check_reply(),
            HealthCheckReply {
                name: "ready",
                valid: true
            }
        );

        check.insert_boxed(Box::new(hw_true));
        assert!(check.check());
        assert_eq!(
            check.check_reply(),
            HealthCheckReply {
                name: "ready",
                valid: true
            }
        );

        check.insert_boxed(Box::new(hw_changing.clone()));
        assert!(check.check());
        assert_eq!(
            check.check_reply(),
            HealthCheckReply {
                name: "ready",
                valid: true
            }
        );

        hw_changing.inner_through_lock().toggle();
        assert!(!check.check());
        assert_eq!(
            check.check_reply(),
            HealthCheckReply {
                name: "ready",
                valid: false
            }
        );

        hw_changing.inner_through_lock().toggle();
        assert!(check.check());
        assert_eq!(
            check.check_reply(),
            HealthCheckReply {
                name: "ready",
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
            fn name_owned(&self) -> String {
                self.name.clone()
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

        assert!(check.check());

        assert!(check.remove_boxed(Box::new(hw0.clone())));
        assert!(!check.remove_boxed(Box::new(hw0.clone())));
        println!("check = {:?}", check);
    }
}
