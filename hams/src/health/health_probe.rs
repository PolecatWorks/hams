//! HealthChecks in Hams

use crate::utils::{AsAny, DynEq, DynHash};
use serde::Serialize;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

/// Detail structure for replies from ready and alive for a single probe
#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct HealthProbeResult<'a> {
    /// Name of health Reply
    pub name: &'a str,
    /// Return value of health Reply
    pub valid: bool,
}

impl<'a> Display for HealthProbeResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.valid)
    }
}

/// A HealthProbeInner requires a get_name and check method are implemented.
///
/// check returns true if the probe is valid else false
/// get_name returns the name of the probe
pub trait HealthProbeInner: Debug + Send {
    /// Get name of HealthCheck
    fn name(&self) -> &str;

    fn name_owned(&self) -> String;
    /// Check if the HealthCheck is valid
    fn check_reply(&self, time: Instant) -> HealthProbeResult;
    fn check(&self, time: Instant) -> bool;
}

/// Trait to define the health check functionality
/// Wrapper around health check to give it a type
#[derive(Debug)]
pub struct HealthProbeWrapper(pub Box<dyn HealthProbeInner>);

impl HealthProbeWrapper {
    /// get the name of HealthCheck
    pub fn get_name(&self) -> &str {
        self.0.name()
    }
    /// Check if the HealthCheck is valid
    pub fn check(&self, time: Instant) -> HealthProbeResult {
        self.0.check_reply(time)
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

// ============= HpW ================

/// Health Probe Wrapper provides a wrapper around the [HealthProbeInner] to create a
/// Arc<Mutex<T>> for it and to provide some methods.
/// It also implements the interface to allwo it to used in a HashSet
#[derive(Debug)]
pub struct HpW<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> HpW<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(value)),
        }
    }
}

impl<T> Clone for HpW<T> {
    fn clone(&self) -> Self {
        HpW {
            inner: self.inner.clone(),
        }
    }
}

impl<T: HealthProbeInner> HpW<T> {
    /// Get the name (as String)
    ///
    /// This is a copy of string so NOT cheap
    pub fn name(&self) -> String {
        self.inner.lock().unwrap().name().to_owned()
    }
}

impl<T: Hash> Hash for HpW<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.lock().unwrap().hash(state);
    }
}
impl<T: Eq> PartialEq for HpW<T> {
    fn eq(&self, other: &Self) -> bool {
        // If we just compare on content then we get a mutex deadlock when we compare an object with itself.
        // This occurs when we are trying to remove a value from a HashSet
        if Arc::ptr_eq(&self.inner, &other.inner) {
            true
        } else {
            *self.inner.lock().unwrap() == *other.inner.lock().unwrap()
        }
    }
}
impl<T: Eq> Eq for HpW<T> {}

impl<T: HealthProbeInner + Eq + Hash + 'static> HealthProbe for HpW<T> {
    fn name(&self) -> &str {
        // self.inner.lock().unwrap().name()
        todo!()
    }
    fn name_owned(&self) -> String {
        self.inner.lock().unwrap().name_owned()
    }

    fn check(&self, time: Instant) -> bool {
        self.inner.lock().unwrap().check(time)
    }
}

impl<T> HpW<T> {
    /// Access the inner via a lock so that we can all methods on it.
    ///
    /// We cannot/should not add a Deref here as it would be too simplistic
    /// to access the &T and hides that there is a Mutex lock in the call.
    /// Better to be explicit than implict here.
    /// Some close examples: https://stackoverflow.com/questions/68138511/why-cant-i-implement-deref-for-a-specific-lifetime
    pub fn inner_through_lock(&self) -> MutexGuard<T> {
        self.inner.lock().unwrap()
    }
}

/// Comparison function for equality of two HealthChecks
/// We can only use the methods available in the HealthCheck for
/// implementation of equality so that limits us to name for comparison
impl PartialEq<dyn HealthProbeInner> for dyn HealthProbeInner {
    fn eq(&self, other: &dyn HealthProbeInner) -> bool {
        self.name() == other.name()
    }
}

/// Trait to describe the external interface of the HealthProbe
///
/// This trait is object safe so can be used in a Box to be stored in HashSet
pub trait HealthProbe: DynEq + DynHash + AsAny {
    /// return the name of the [HealthProbe]
    fn name(&self) -> &str;

    fn name_owned(&self) -> String;
    /// check if the healthCheck is valid. True is valid
    fn check(&self, now: Instant) -> bool;
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
        write!(f, "{}/{}", self.name_owned(), self.check(Instant::now()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    // Create a HealthProbe that is a HpW<T> of a couple of Inners and then load them into a HashSet

    #[test]
    fn equality_for_inner() {
        #[derive(Debug, Hash, PartialEq)]
        struct InnerI {
            name: String,
            count: u32,
        }
        impl Eq for InnerI {}

        let mut hpi0 = InnerI {
            name: "Hpi0".to_owned(),
            count: 3,
        };
        let mut hpi1 = InnerI {
            name: "Hpi1".to_owned(),
            count: 2,
        };
        let mut hpi2 = InnerI {
            name: "Hpi0".to_owned(),
            count: 3,
        };

        assert_ne!(hpi0, hpi1);
        assert_eq!(hpi0, hpi2);

        assert_eq!(hpi0, hpi0);

        let hpw0 = HpW::new(hpi0);
        let hpw1 = HpW::new(hpi1);
        let hpw2 = HpW::new(hpi2);

        assert_ne!(hpw0, hpw1);
        assert_eq!(hpw0, hpw2);

        assert_eq!(hpw0, hpw0);
    }

    #[test]
    fn try_out_healthprobe_of_hpw() {
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
            fn check_reply(&self, time: Instant) -> HealthProbeResult {
                todo!()
            }

            fn check(&self, time: Instant) -> bool {
                self.count < 10
            }
        }

        #[derive(Debug, Hash, PartialEq)]
        struct InnerJ {
            name: String,
            count: u32,
        }
        impl Eq for InnerJ {}

        impl InnerJ {
            pub fn kick(&mut self) {
                self.count += 1
            }
        }

        impl HealthProbeInner for InnerJ {
            fn name(&self) -> &str {
                &self.name
            }
            fn name_owned(&self) -> String {
                self.name.clone()
            }
            fn check_reply(&self, time: Instant) -> HealthProbeResult {
                todo!()
            }

            fn check(&self, time: Instant) -> bool {
                self.count < 10
            }
        }

        let mut hpi0 = InnerI {
            name: "Hpi0".to_owned(),
            count: 3,
        };
        let mut hpj0 = InnerJ {
            name: "Hpj0".to_owned(),
            count: 2,
        };

        println!("{:?} is called {}", hpi0, hpi0.name());

        let hw0 = HpW::new(hpi0);
        let hw1 = HpW::new(hpj0);

        println!("Getting hw0 check {}", hw0.check(Instant::now()));
        println!("Getting hw1 check {}", hw1.check(Instant::now()));

        let mut hc0: HashSet<Box<dyn HealthProbe>> = HashSet::new();

        hc0.insert(Box::new(hw0.clone()));
        hc0.insert(Box::new(hw1.clone()));
    }

    #[test]
    fn try_out_healthprobeinner() {
        #[derive(Debug)]
        struct InnerI {
            name: String,
            count: u32,
        }
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
            fn check_reply(&self, time: Instant) -> HealthProbeResult {
                todo!()
            }

            fn check(&self, time: Instant) -> bool {
                todo!()
            }
        }

        let mut hpi0 = InnerI {
            name: "Hpi0".to_owned(),
            count: 3,
        };

        println!("{:?} is called {}", hpi0, hpi0.name());

        let hw0 = HpW::new(hpi0);

        println!("hw {:?} is called {}", hw0, hw0.name());
        hw0.inner_through_lock().kick();
        println!("hw {:?} is called {}", hw0, hw0.name());
        hw0.inner_through_lock().kick();
        println!("hw {:?} is called {}", hw0, hw0.name());
        hw0.inner_through_lock().kick();
        println!("hw {:?} is called {}", hw0, hw0.name());
    }

    #[test]
    fn try_out_healthprobe() {
        #[derive(Hash, PartialEq, Debug)]
        struct ExampleI {
            name: String,
            count: i64,
        }
        impl Eq for ExampleI {}
        impl HealthProbe for ExampleI {
            fn check(&self, now: Instant) -> bool {
                self.count < 10
            }
            fn name(&self) -> &str {
                &self.name
            }
            fn name_owned(&self) -> String {
                self.name.clone()
            }
        }

        #[derive(Hash, PartialEq, Debug)]
        struct ExampleJ {
            name: String,
            count: i64,
        }
        impl Eq for ExampleJ {}
        impl HealthProbe for ExampleJ {
            fn check(&self, now: Instant) -> bool {
                self.count < 10
            }
            fn name(&self) -> &str {
                &self.name
            }
            fn name_owned(&self) -> String {
                self.name.clone()
            }
        }

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
        fn name(&self) -> &str {
            println!("HealthCheck for I {}", self.name);
            &self.name
        }
        fn name_owned(&self) -> String {
            self.name.clone()
        }
        fn check_reply(&self, time: std::time::Instant) -> HealthProbeResult {
            todo!()
        }

        fn check(&self, time: Instant) -> bool {
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
