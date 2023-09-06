use std::time::Duration;

use crate::{health_check::HealthCheck, health_kick::HealthKick};

#[no_mangle]
pub unsafe extern "C" fn new_kick() -> *mut HealthCheck {
    HealthCheck::for_hc(HealthKick::new("sausage", Duration::from_secs(3)))
}

// src/ffi.rs

#[no_mangle]
pub unsafe extern "C" fn health_destroy(handle: *mut HealthCheck) {
    let destructor = (*handle).destroy;
    destructor(handle);
}

#[cfg(test)]
mod health_tests {
    use crate::health_check::Health;

    use super::*;
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::Instant,
    };

    struct NotifyOnDrop(Arc<AtomicBool>);

    impl Drop for NotifyOnDrop {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    impl Health for NotifyOnDrop {
        fn name(&self) -> Result<String, crate::error::HamsError> {
            todo!()
        }

        fn check(
            &self,
            time: Instant,
        ) -> Result<crate::health_check::HealthCheckResult, crate::error::HamsError> {
            todo!()
        }

        fn previous(&self) -> Result<bool, crate::error::HamsError> {
            todo!()
        }
    }

    #[test]
    fn health_destructor_is_always_called() {
        let was_dropped = Arc::new(AtomicBool::new(false));
        let health_check = HealthCheck::for_hc(NotifyOnDrop(Arc::clone(&was_dropped)));
        assert!(!health_check.is_null());

        unsafe {
            health_destroy(health_check);
        }

        assert!(was_dropped.load(Ordering::SeqCst));
    }
}
