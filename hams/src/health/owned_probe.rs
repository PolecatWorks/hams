use std::{any::TypeId, ptr::NonNull};

use super::health_probe2::{HealthProbe2, HealthProbeFuncs, Repr};

#[derive(Debug)]
#[repr(transparent)]
pub struct OwnedProbe(NonNull<HealthProbe2>);

impl OwnedProbe {
    /// Create a new [`OwnedProbe`] which wraps some [`HealthProbeFuncs`]r.
    pub fn new<P: HealthProbeFuncs + Send + Sync + 'static>(probe: P) -> Self {
        unsafe {
            let handle = HealthProbe2::for_hp(probe);
            assert!(!handle.is_null());
            OwnedProbe::from_raw(handle)
        }
    }
    /// Create an [`OwnedProbe`] from a `*mut HealthProbeFuns`, taking ownershio of the [`HealthProbeFuncs`]
    pub unsafe fn from_raw(handle: *mut HealthProbe2) -> Self {
        debug_assert!(!handle.is_null());
        OwnedProbe(NonNull::new_unchecked(handle))
    }

    /// Consume the [`OwnedProbe`] and get a `*mut HealthProbeFuncs`
    pub fn into_raw(self) -> *mut HealthProbe2 {
        let ptr = self.0.as_ptr();
        std::mem::forget(self);
        ptr
    }

    pub fn is<P: 'static>(&self) -> bool {
        unsafe {
            let ptr = self.0.as_ptr();
            (*ptr).type_id == TypeId::of::<P>()
        }
    }

    pub fn downcast_ref<P: 'static>(&self) -> Option<&P> {
        if self.is::<P>() {
            unsafe {
                let repr = self.0.as_ptr() as *const Repr<P>;
                Some(&(*repr).probe)
            }
        } else {
            None
        }
    }
    pub fn downcast_mut<P: 'static>(&mut self) -> Option<&mut P> {
        if self.is::<P>() {
            unsafe {
                let repr = self.0.as_ptr() as *mut Repr<P>;
                Some(&mut ((*repr).probe))
            }
        } else {
            None
        }
    }

    pub fn downcast<P: 'static>(self) -> Result<P, Self> {
        if self.is::<P>() {
            unsafe {
                let ptr = self.into_raw();
                let repr: *mut Repr<P> = ptr.cast();
                let unboxed = Box::from_raw(repr);
                Ok(unboxed.probe)
            }
        } else {
            Err(self)
        }
    }
}

impl HealthProbeFuncs for OwnedProbe {
    fn name(&self) -> Result<String, crate::error::HamsError> {
        unsafe {
            let ptr = self.0.as_ptr();
            let name = (*ptr).name;
            (name)(ptr)
        }
    }

    fn check(&self, time: std::time::Instant) -> Result<bool, crate::error::HamsError> {
        unsafe {
            let ptr = self.0.as_ptr();
            let check = (*ptr).check;
            (check)(ptr, time)
        }
    }
}

impl Drop for OwnedProbe {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.0.as_ptr();
            let destroy = (*ptr).destroy;
            (destroy)(ptr)
        }
    }
}

unsafe impl Send for OwnedProbe {}
unsafe impl Sync for OwnedProbe {}
