use std::{
    hash::{Hash, Hasher},
    sync::{Arc, Mutex, MutexGuard},
    time::Instant,
};

use crate::{
    error::HamsError,
    health::{Health, HealthCheckResult},
};

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
    fn check(&self, time: Instant) -> Result<HealthCheckResult, HamsError> {
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

#[cfg(test)]
mod wrapped_tests {
    use crate::health_manual::HealthManual;

    use super::*;

    #[test]
    fn create_wrapped_health() {
        let mut my_hc = HealthWrapper::new(HealthManual::new("bue0", false));

        println!("my hc = {:?}", my_hc);

        my_hc.lock().disable();
    }
}
