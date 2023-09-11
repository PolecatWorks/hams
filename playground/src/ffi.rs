use std::{
    ffi::{c_char, c_int, CString},
    time::Duration,
};

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

#[no_mangle]
pub unsafe extern "C" fn health_name(handle: *mut HealthCheck) -> *mut c_char {
    let name = (*handle).name;

    match name(handle) {
        Ok(my_name) => {
            let c_str_name = CString::new(my_name).unwrap();
            c_str_name.into_raw()
        }
        Err(_) => todo!(),
    }
}

#[no_mangle]
pub extern "C" fn health_name_free(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        CString::from_raw(s)
    };
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

    #[test]
    fn create_kick_and_destroy_it() {
        unsafe {
            let handle = new_kick();
            assert!(!handle.is_null());

            health_destroy(handle);
        }
    }

    #[test]
    fn create_kick_and_get_name() {
        unsafe {
            let handle = new_kick();
            assert!(!handle.is_null());

            let name = health_name(handle);

            health_name_free(name);

            health_destroy(handle);
        }
    }

    #[test]
    fn get_health_name_and_release() {}
}
