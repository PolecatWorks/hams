use std::{
    alloc::Layout,
    any::{Any, TypeId},
    fmt::{Debug, Display, Formatter},
    sync::Mutex,
    time::Instant,
};

use crate::error::HamsError;

/// Describe the interface required to be provided for a Health Probe
pub trait HealthProbeFuncs {
    /// Return the name of the probe
    fn name(&self) -> Result<String, HamsError>;
    /// check the state of the probe
    fn check(&self, time: Instant) -> Result<bool, HamsError>;
}

#[repr(C)]
pub struct HealthProbe2 {
    pub(crate) layout: Layout,
    pub(crate) type_id: TypeId,
    pub(crate) poisoned: bool,
    pub(crate) destroy: unsafe fn(*mut Self),
    pub(crate) name: unsafe fn(*mut Self) -> Result<String, HamsError>,
    pub(crate) check: unsafe fn(*mut Self, time: Instant) -> Result<bool, HamsError>,
}

impl HealthProbe2 {
    /// Constructor for HP2
    pub fn for_hp<W>(probe: W) -> *mut Self
    where
        W: HealthProbeFuncs + Send + Sync + 'static,
    {
        let repr = Repr {
            base: Self::vtable::<W>(),
            probe,
        };

        let boxed = Box::into_raw(Box::new(repr));

        boxed as *mut _
    }

    fn vtable<W: HealthProbeFuncs + 'static>() -> Self {
        let layout = Layout::new::<Repr<W>>();
        let type_id = TypeId::of::<W>();

        Self {
            layout,
            type_id,
            poisoned: false,
            destroy: destroy::<W>,
            name: name::<W>,
            check: check::<W>,
        }
    }
}

unsafe fn destroy<W>(handle: *mut HealthProbe2) {
    if handle.is_null() {
        return;
    }

    let repr = handle as *mut Repr<W>;

    if (*handle).poisoned {
        let layout = (*handle).layout;
        std::alloc::dealloc(repr.cast(), layout);
    } else {
        let _ = Box::from_raw(repr);
    }
}

macro_rules! auto_poison {
    ($handle:expr, $body:block) => {{
        if (*$handle).poisoned {
            Err(HamsError::InvalidData(
                "A panic occurred and this object is now poisoned",
            ))
        } else {
            let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || $body));
            match got {
                Ok(value) => value,
                Err(payload) => {
                    (*$handle).poisoned = true;
                    Err(HamsError::Poisoned(Poisoned::from(payload)))
                }
            }
        }
    }};
}

unsafe fn name<W: HealthProbeFuncs>(handle: *mut HealthProbe2) -> Result<String, HamsError> {
    auto_poison!(handle, {
        let repr = &mut *(handle as *mut Repr<W>);
        repr.probe.name()
    })
}

unsafe fn check<W: HealthProbeFuncs>(
    handle: *mut HealthProbe2,
    time: Instant,
) -> Result<bool, HamsError> {
    auto_poison!(handle, {
        let repr = &mut *(handle as *mut Repr<W>);
        repr.probe.check(time)
    })
}

#[derive(Debug)]
pub struct Poisoned(Mutex<Box<dyn Any + Send + 'static>>);

impl From<Box<dyn Any + Send + 'static>> for Poisoned {
    fn from(payload: Box<dyn Any + Send + 'static>) -> Self {
        Poisoned(Mutex::new(payload))
    }
}

#[repr(C)]
pub(crate) struct Repr<W> {
    // SAFETY: The HealthCheck must be the first field so we can cast between
    // *mut Repr<W> and *mut HealthCheck
    pub(crate) base: HealthProbe2,
    pub(crate) probe: W,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ABC_for_HP2() {
        struct MyProbe0 {
            name: String,
            checker: bool,
        }

        impl HealthProbeFuncs for MyProbe0 {
            fn name(&self) -> Result<String, HamsError> {
                Ok(self.name.clone())
            }

            fn check(&self, time: Instant) -> Result<bool, HamsError> {
                Ok(self.checker)
            }
        }

        struct MyProbe1 {
            name: String,
            checker: bool,
        }

        impl HealthProbeFuncs for MyProbe1 {
            fn name(&self) -> Result<String, HamsError> {
                Ok(self.name.clone())
            }

            fn check(&self, time: Instant) -> Result<bool, HamsError> {
                Ok(self.checker)
            }
        }

        let probe0 = MyProbe0 {
            name: "probe0".to_owned(),
            checker: false,
        };

        let probe1 = MyProbe0 {
            name: "probe1".to_owned(),
            checker: false,
        };
        let probe2 = MyProbe1 {
            name: "probe2".to_owned(),
            checker: false,
        };

        let my_hp0 = HealthProbe2::for_hp(probe0);
        let my_hp1 = HealthProbe2::for_hp(probe1);
        let my_hp2 = HealthProbe2::for_hp(probe2);

        let mut my_vec = vec![];

        // let x = unsafe {((*my_hp0).check)()};

        my_vec.push(my_hp0);
        my_vec.push(my_hp1);
        my_vec.push(my_hp2);

        for hp in my_vec {
            let name = unsafe { ((*hp).name)(hp) };
            println!("name: {:?}", name);
            // assert_eq!(name.unwrap(), "probe0");
        }

        assert!(false, "Deliberate fail to see progress");
    }
}
