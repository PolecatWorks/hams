use crate::health_check::{Health, HealthCheck, Repr};
use std::{any::TypeId, ptr::NonNull, time::Instant};

#[derive(Debug)]
#[repr(transparent)]
pub struct OwnedHealthCheck(NonNull<HealthCheck>);

impl OwnedHealthCheck {
    /// Create a new [`OwnedFileHandle`] which wraps some [`Write`]r.
    pub fn new<W>(hc: W) -> Self
    where
        W: Health + Send + Sync + 'static,
    {
        unsafe {
            let handle = HealthCheck::for_hc(hc);
            assert!(!handle.is_null());
            OwnedHealthCheck::from_raw(handle)
        }
    }

    /// Create an [`OwnedHealthCheck`] from a `*mut HealthCheck`, taking
    /// ownership of the [`HealthCheck`].
    ///
    /// # Safety
    ///
    /// Ownership of the `handle` is given to the [`OwnedHealthCheck`] and the
    /// original pointer may no longer be used.
    ///
    /// The `handle` must be a non-null pointer which points to a valid
    /// `HealthCheck`.
    pub unsafe fn from_raw(handle: *mut HealthCheck) -> Self {
        debug_assert!(!handle.is_null());
        OwnedHealthCheck(NonNull::new_unchecked(handle))
    }

    /// Consume the [`OwnedHealthCheck`] and get a `*mut HealthCheck` that can be
    /// used from native code.
    pub fn into_raw(self) -> *mut HealthCheck {
        let ptr = self.0.as_ptr();
        std::mem::forget(self);
        ptr
    }

    /// Check if the object pointed to by a [`OwnedHeathCheck`] has type `W`.
    pub fn is<W: 'static>(&self) -> bool {
        unsafe {
            let ptr = self.0.as_ptr();
            (*ptr).type_id == TypeId::of::<W>()
        }
    }

    /// Returns a reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    pub fn downcast_ref<W: 'static>(&self) -> Option<&W> {
        if self.is::<W>() {
            unsafe {
                // SAFETY: We just did a type check
                let repr = self.0.as_ptr() as *const Repr<W>;
                Some(&(*repr).health_check)
            }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    pub fn downcast_mut<W: 'static>(&mut self) -> Option<&mut W> {
        if self.is::<W>() {
            unsafe {
                // SAFETY: We just did a type check
                let repr = self.0.as_ptr() as *mut Repr<W>;
                Some(&mut (*repr).health_check)
            }
        } else {
            None
        }
    }

    /// Attempt to downcast the [`OwnedFileHandle`] to a concrete type and
    /// extract it.
    pub fn downcast<W: 'static>(self) -> Result<W, Self> {
        if self.is::<W>() {
            unsafe {
                let ptr = self.into_raw();
                // SAFETY: We just did a type check
                let repr: *mut Repr<W> = ptr.cast();

                let unboxed = Box::from_raw(repr);
                Ok(unboxed.health_check)
            }
        } else {
            Err(self)
        }
    }
}

impl Health for OwnedHealthCheck {
    fn name(&self) -> Result<String, crate::error::HamsError> {
        unsafe {
            let ptr = self.0.as_ptr();
            let name = (*ptr).name;
            (name)(ptr)
        }
    }

    fn check(
        &self,
        time: Instant,
    ) -> Result<crate::health_check::HealthCheckResult, crate::error::HamsError> {
        unsafe {
            let ptr = self.0.as_ptr();
            let check = (*ptr).check;
            (check)(ptr, time)
        }
    }

    fn previous(&self) -> Result<bool, crate::error::HamsError> {
        unsafe {
            let ptr = self.0.as_ptr();
            let previous = (*ptr).previous;
            (previous)(ptr)
        }
    }
}

impl Drop for OwnedHealthCheck {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.0.as_ptr();
            let destroy = (*ptr).destroy;
            (destroy)(ptr)
        }
    }
}

// SAFETY: The FileHandle::for_writer() method ensure by construction that our
// object is Send + Sync.
unsafe impl Send for OwnedHealthCheck {}
unsafe impl Sync for OwnedHealthCheck {}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
        time::Duration,
    };

    use crate::{
        error::HamsError,
        health_check::HealthCheckResult,
        health_kick::HealthKick,
        // health_kick::HealthKick,
    };

    use super::*;

    #[derive(Debug, Clone)]
    struct SharedBuffer {
        pub name: Arc<Mutex<String>>,
        pub hcr: Arc<Mutex<HealthCheckResult>>,
    }

    impl SharedBuffer {
        pub fn new<S: Into<String>>(name: S) -> SharedBuffer {
            let my_string: String = name.into();
            SharedBuffer {
                name: Arc::new(Mutex::new(my_string.clone())),
                hcr: Arc::new(Mutex::new(HealthCheckResult {
                    name: my_string.clone(),
                    valid: false,
                })),
            }
        }
    }

    impl Health for SharedBuffer {
        fn name(&self) -> Result<String, crate::error::HamsError> {
            Ok("3".to_owned())
        }

        fn check(&self, time: Instant) -> Result<HealthCheckResult, crate::error::HamsError> {
            let my_hcr = self.hcr.lock().unwrap();

            Ok((*my_hcr).clone())
        }

        fn previous(&self) -> Result<bool, HamsError> {
            todo!()
        }
    }

    #[test]
    fn downcast_ref() {
        let buffer = SharedBuffer::new("apple");
        let handle = OwnedHealthCheck::new(buffer.clone());

        let got = handle.downcast_ref::<SharedBuffer>().unwrap();
        assert!(Arc::ptr_eq(&got.hcr, &buffer.hcr));
    }

    #[test]
    fn downcast_mut() {
        let buffer = SharedBuffer::new("apple");
        let mut handle = OwnedHealthCheck::new(buffer.clone());

        let got = handle.downcast_mut::<SharedBuffer>().unwrap();
        assert!(Arc::ptr_eq(&got.hcr, &buffer.hcr));
    }

    #[test]
    fn downcast_owned_doesnt_destroy_twice() {
        let handle = OwnedHealthCheck::new(HealthKick::new("apple", Duration::from_secs(3)));

        let got = handle.downcast::<SharedBuffer>();
        assert!(got.is_err());
        let handle = got.unwrap_err();

        let got = handle.downcast::<HealthKick>();
        assert!(got.is_ok());
    }

    #[derive(Debug)]
    struct Panicking {
        dropped: Arc<AtomicBool>,
    }

    impl Health for Panicking {
        fn name(&self) -> Result<String, HamsError> {
            panic!()
        }

        fn check(&self, time: Instant) -> Result<HealthCheckResult, HamsError> {
            // Err(HamsError::Message("Deliberate panic"))
            panic!()
        }

        fn previous(&self) -> Result<bool, HamsError> {
            panic!()
        }
    }

    impl Drop for Panicking {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::SeqCst);

            // Note: double-panic = abort
            if !std::thread::panicking() {
                panic!();
            }
        }
    }

    #[test]
    fn owned_handle_poisons_on_panic() {
        let was_dropped = Arc::new(AtomicBool::new(false));
        let mut checker = OwnedHealthCheck::new(Panicking {
            dropped: Arc::clone(&was_dropped),
        });

        let got = checker.check(Instant::now());

        assert!(got.is_err());

        drop(checker);

        assert!(
            !was_dropped.load(Ordering::SeqCst),
            "The destructor shouldn't have run"
        );

        // This isn't part of the test, but we need to manually decrement the
        // arc's reference count so Miri's leak detector doesn't make this test
        // fail when we deliberately wanted poisoned writers to be leaked.
        unsafe {
            let other_arc = Arc::from_raw(Arc::as_ptr(&was_dropped) as *const _);
            drop(other_arc);
        }
    }
}
