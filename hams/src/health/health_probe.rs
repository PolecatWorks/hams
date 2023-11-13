//! HealthChecks in Hams

use crate::health::HealthProbeResult;
use crate::utils::{AsAny, DynEq, DynHash};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::time::Instant;

pub trait HealthProbeInner: Debug + Send {
    /// Get name of HealthCheck
    fn get_name(&self) -> &str;
    /// Check if the HealthCheck is valid
    fn check(&self, time: Instant) -> HealthProbeResult;
}

/// Trait to define the health check functionality
/// Wrapper around health check to give it a type
#[derive(Debug)]
pub struct HealthProbeWrapper(pub Box<dyn HealthProbeInner>);

impl HealthProbeWrapper {
    /// get the name of HealthCheck
    pub fn get_name(&self) -> &str {
        self.0.get_name()
    }
    /// Check if the HealthCheck is valid
    pub fn check(&self, time: Instant) -> HealthProbeResult {
        self.0.check(time)
    }
}
impl Eq for HealthProbeWrapper {}

impl PartialEq for HealthProbeWrapper {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0.as_ref(), other.0.as_ref())
    }
}

impl Hash for HealthProbeWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0.as_ref(), state);
    }
}

/// Comparison function for equality of two HealthChecks
/// We can only use the methods available in the HealthCheck for
/// implementation of equality so that limits us to name for comparison
impl PartialEq<dyn HealthProbeInner> for dyn HealthProbeInner {
    fn eq(&self, other: &dyn HealthProbeInner) -> bool {
        println!("IM IN PartialEq");
        println!("Comparing {} and {}", self.get_name(), other.get_name());
        self.get_name() == other.get_name()
    }
}

/// Trait to describe the external interface of the HealthProbe
pub trait HealthProbe: DynEq + DynHash + AsAny {
    /// return the name of the [HealthProbe]
    fn name(&self) -> &str;
    /// check if the healthCheck is valid. True is valid
    fn check(&self) -> bool;
}
/// Implement PartialEq for HealthProbe to allow Eq to derive the PartialEq for this trait
impl PartialEq for dyn HealthProbe {
    fn eq(&self, other: &dyn HealthProbe) -> bool {
        DynEq::dyn_eq(self, other.as_any())
    }
}
impl Hash for dyn HealthProbe {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}
impl Eq for dyn HealthProbe {}
impl std::fmt::Debug for dyn HealthProbe {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.name(), self.check())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[derive(Hash, PartialEq, Debug)]
    struct ExampleI {
        name: String,
        count: i64,
    }
    impl Eq for ExampleI {}
    impl HealthProbe for ExampleI {
        fn check(&self) -> bool {
            self.count < 10
        }
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[derive(Hash, PartialEq, Debug)]
    struct ExampleJ {
        name: String,
        count: i64,
    }
    impl Eq for ExampleJ {}
    impl HealthProbe for ExampleJ {
        fn check(&self) -> bool {
            self.count < 10
        }
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn try_out_healthprobe() {
        let i0 = ExampleI {
            name: "I0".to_owned(),
            count: 2,
        };

        println!("i0 = {:?}", i0);

        let j0 = ExampleJ {
            name: "J0".to_owned(),
            count: 1,
        };
        let j1 = ExampleJ {
            name: "J1".to_owned(),
            count: 12,
        };

        let mut hc0: HashSet<Box<dyn HealthProbe>> = HashSet::new();

        hc0.insert(Box::new(i0));
        hc0.insert(Box::new(j0));
        hc0.insert(Box::new(j1));

        println!("hc0 = {:?}", hc0);

        // assert!(false);
    }

    #[derive(Debug, Clone)]
    struct I {
        name: String,
        count: i64,
    }

    impl HealthProbeInner for I {
        fn get_name(&self) -> &str {
            println!("HealthCheck for I {}", self.name);
            &self.name
        }

        fn check(&self, time: std::time::Instant) -> HealthProbeResult {
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

        assert_eq!(&hc0 as &dyn HealthProbeInner, &hc2 as &dyn HealthProbeInner);
        assert_ne!(&hc0 as &dyn HealthProbeInner, &hc1 as &dyn HealthProbeInner);

        let hclist: Vec<Box<dyn HealthProbeInner>> =
            vec![Box::new(hc0), Box::new(hc1), Box::new(hc2)];

        let ben = *hclist[0] == *hclist[1];

        assert_eq!(*hclist[0], *hclist[2]);
        assert_ne!(*hclist[0], *hclist[1]);

        let hc0_ref: *const dyn HealthProbeInner = hclist[0].as_ref();
        let hc1_ref: *const dyn HealthProbeInner = hclist[1].as_ref();
        let hc2_ref: *const dyn HealthProbeInner = hclist[2].as_ref();

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
