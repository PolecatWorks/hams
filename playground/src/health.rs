use std::fmt::Display;
use std::hash::Hash;
use std::{
    hash::Hasher,
    sync::{Arc, Mutex, MutexGuard},
    time::Instant,
};

use serde::Serialize;

/// Detail structure for replies from ready and alive
#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct HealthCheckResult {
    /// Name of health Reply
    pub name: String,
    /// Return value of health Reply
    pub valid: bool,
}

impl<'a> Display for HealthCheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.valid)
    }
}

/** Health trait requires that the object implements the check function that returns a HealthCheckResult
 ** suitable for inclusion in a k8s health probe (eg ready or alive)
 */
pub trait Health {
    fn check(&self, time: Instant) -> HealthCheckResult;
}

#[derive(Debug)]
pub struct HealthWrapper<MyType>
// where MyType: Health
{
    inner: Arc<Mutex<MyType>>,
}

impl<MyType> HealthWrapper<MyType> {
    pub fn new(value: MyType) -> Self {
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
