use std::hash::Hash;
use std::{
    collections::HashSet,
    hash::Hasher,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
    time::Instant,
};

use log::info;

use crate::health_check::HealthCheckResult;

/** Health trait requires that the object implements the check function that returns a HealthCheckResult
 ** suitable for inclusion in a k8s health probe (eg ready or alive)
 */
pub trait Health {
    fn check(&self, time: Instant) -> HealthCheckResult;
}

#[derive(Debug)]
struct HealthWrapper<MyType>
// where MyType: Health
{
    inner: Arc<Mutex<MyType>>,
}

impl<MyType> HealthWrapper<MyType> {
    fn new(value: MyType) -> Self {
        HealthWrapper {
            inner: Arc::new(Mutex::new(value)),
        }
    }

    pub fn lock(&mut self) -> MutexGuard<MyType> {
        self.inner.lock().unwrap()
    }
}

impl<MyType> Clone for HealthWrapper<MyType> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<MyType> Health for HealthWrapper<MyType>
where
    MyType: Health,
{
    fn check(&self, time: Instant) -> HealthCheckResult {
        self.inner.lock().unwrap().check(time)
    }
}

impl<MyType> PartialEq for HealthWrapper<MyType> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.inner.as_ref(), other.inner.as_ref())
    }
}
impl<MyType> Hash for HealthWrapper<MyType> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.inner.as_ref(), state);
    }
}
impl<MyType> Eq for HealthWrapper<MyType> {}

/** HealthProbe describes a list of health checks each of which contributes to the outcome and content of the probe */
#[derive(Debug)]
struct HealthProbe<HealthCheck> {
    /// vector that is shared across clones AND the objects it refers to can also be independantly shared
    detail: Arc<Mutex<HashSet<HealthCheck>>>,
    /// enable alive reply or disable (for debug use)
    enabled: Arc<AtomicBool>,
}

impl<HealthCheck> HealthProbe<HealthCheck>
where
    HealthCheck: Eq + Hash + Health + std::fmt::Debug,
{
    fn new(enabled: bool) -> HealthProbe<HealthCheck> {
        info!("Constructing HealthProbe");

        HealthProbe {
            detail: Arc::new(Mutex::new(HashSet::new())),
            enabled: Arc::new(AtomicBool::new(enabled)),
        }
    }

    fn enable(&self) {
        self.setEnabled(false)
    }
    fn disable(&self) {
        self.setEnabled(true)
    }
    fn setEnabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed)
    }

    fn insert(&self, hc: HealthCheck) -> bool {
        self.detail.lock().unwrap().insert(hc)
    }
    fn remove(&self, hc: HealthCheck) -> bool {
        self.detail.lock().unwrap().remove(&hc)
    }

    pub fn check_all(&self, time: Instant) -> (bool, Vec<HealthCheckResult>) {
        let my_lock = self.detail.lock().unwrap();
        let detail = my_lock.iter().map(|health| health.check(time));
        println!("enableds are {:?} from {:?}", self.enabled, self.detail);
        let valid =
            !self.enabled.load(Ordering::Relaxed) || detail.clone().all(|result| result.valid);
        let xxx = detail.collect();
        (valid, xxx)
        // (valid, detail.collect())
    }
}

#[cfg(test)]
mod tests {

    use crate::health_manual::HealthManual;

    use super::*;

    #[test]
    fn probe_enabled_using_health_manual() {
        let my_probe = HealthProbe::new(true);

        let mut my_hc = HealthWrapper::new(HealthManual::new("sample", true));
        my_probe.insert(my_hc.clone());

        assert!(
            my_probe.check_all(Instant::now()).0,
            "true reponse on reply"
        );
        my_hc.lock().disable();
        assert!(!my_probe.check_all(Instant::now()).0, "set to false");
        my_hc.lock().enable();
        assert!(
            my_probe.check_all(Instant::now()).0,
            "true reponse on reply"
        );
        my_hc.lock().set(false);
        assert!(!my_probe.check_all(Instant::now()).0, "set to false");
    }

    #[test]
    fn example_functionality_of_check_and_probe() {
        let my_probe = HealthProbe::new(true);

        let mut my_hc = HealthWrapper::new(HealthManual::new("blue", false));

        my_probe.insert(my_hc.clone());

        println!("i still have my_hc {my_hc:?}");

        let check_reply = my_probe.check_all(Instant::now());
        println!("reply = {:?}", check_reply.1);

        my_hc.lock().state = true;

        let check_reply = my_probe.check_all(Instant::now());
        println!("reply = {:?}", check_reply.1);
    }

    #[test]
    fn probe_add_remove() {
        let my_probe = HealthProbe::new(true);

        let my_hc0 = HealthWrapper::new(HealthManual::new("blue0", false));
        let my_hc1 = HealthWrapper::new(HealthManual::new("blue1", false));
        let my_hc2 = HealthWrapper::new(HealthManual::new("blue2", false));

        my_probe.insert(my_hc0);
        my_probe.insert(my_hc1.clone());
        my_probe.insert(my_hc2);

        assert!(my_probe.check_all(Instant::now()).1.len() == 3);

        assert!(my_probe.remove(my_hc1));

        assert!(my_probe.check_all(Instant::now()).1.len() == 2);

        let reply = my_probe.check_all(Instant::now());
        println!("reply was {:?}", my_probe.check_all(Instant::now()));
        assert!(reply.1[0].name == "blue0");
        assert!(reply.1[1].name == "blue2");
    }

    // #[test]
    // fn health_create() {

    //     let kick = HealthKick::new("black", Duration::from_secs(3));

    //     let hc0 = HealthCheck::for_hc(kick);

    //     // let ben = (*hc0).name;

    // }

    // #[test]
    // fn probe_create_and_destroy() {
    //     let probe = HealthProbe::<u32>::new();
    // }

    #[test]
    fn reuse_iter() {
        let my_vec = vec![1, 3, 5, 7, 9];

        let my_iter = my_vec.iter().map(|val| val * val + 3);

        let other_iter = my_iter.clone();

        let total = my_iter.count();

        let sum: u32 = other_iter.sum();

        println!("i have got {total} records summing to {sum}");
    }

    // #[test]
    // fn probe_add_remove() {
    //     let my_probe = HealthProbe::new();

    //     let mut my_box0 = Box::new(123);

    //     let mut my_box1 = my_box0.clone();

    //     let my_box2 = my_box0.as_ref();

    //     println!("my box 0 = {}", my_box0);
    //     println!("my box 1 = {}", my_box1);
    //     println!("my box 2 = {}", my_box2);

    //     // *my_box0=55;

    //     println!("my box 0 = {}", my_box0);
    //     println!("my box 1 = {}", my_box1);
    //     println!("my box 2 = {}", my_box2);

    //     let mut my_hc0 = 33;
    //     let mut my_hc1 = 34;

    //     let my_insert = my_probe.insert(my_hc0.clone());
    //     println!("insert = {}", my_insert);
    //     let my_insert = my_probe.insert(my_hc1.clone());
    //     println!("insert = {}", my_insert);

    //     println!("Hams = {:?}", my_probe);

    //     my_hc0 = 22;
    //     println!("Hams = {:?}", my_probe);
    //     let my_remove = my_probe.remove(my_hc0);

    //     println!("remove = {}", my_remove);
    //     println!("Hams = {:?}", my_probe);

    // }

    // #[test]
    // fn construct_prob_with_healthcheck() {
    //     let hk0 = HealthKick::new("pear0", Duration::from_secs(10));
    //     let hk1 = HealthKick::new("pear1", Duration::from_secs(10));
    //     let hk2 = HealthKick::new("pear2", Duration::from_secs(10));

    //     let hc0 = OwnedHealthCheck::new(hk0.clone());
    //     let hc1 = OwnedHealthCheck::new(hk1.clone());
    //     // let hc0 = HealthCheck::for_hc(hk0.clone());
    //     let mut myvec = HashSet::new();

    //     myvec.insert(hc0);
    //     myvec.insert(hc1);

    //     println!("My HasSet is {}", myvec.len());
    //     println!("My list: {:?}", myvec);

    //     let rem = myvec.remove(&OwnedHealthCheck::new(hk0.clone()));
    //     println!("did i remove it {}", rem);
    //     println!("My HasSet is {}", myvec.len());

    //     hk0.kick();

    // }

    // #[test]
    // fn construct_probe_and_populate_to_array_with_pointer() {
    //     let probe = HealthProbe::new();

    //     let hc0 = HealthKick::new("banana0", Duration::from_secs(10));
    //     let hc1 = HealthKick::new("banana1", Duration::from_secs(10));
    //     let hc2 = HealthKick::new("banana2", Duration::from_secs(10));

    //     let mut myvec = Vec::new();

    //     hc0.kick();
    //     myvec.push(&hc0);
    //     myvec.push(&hc1);
    //     myvec.push(&hc2);

    //     println!("myvec = {:?}", myvec);

    //     // probe.insert(&hc);

    //     hc0.kick();

    //     println!("myvec = {:?}", myvec);
    //     // probe.remove(&hc);
    //     let me = myvec.remove(0);
    //     drop(me);
    //     drop(myvec);
    //     drop(hc0);
    //     // println!("myvec = {:?}", myvec);

    //     // probe.insert(&hc);

    //     // let hc = HealthCheck::new();
    // }
}
