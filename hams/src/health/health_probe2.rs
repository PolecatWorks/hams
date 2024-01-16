use std::{
    alloc::Layout,
    any::{Any, TypeId},
    fmt::{Debug, Display, Formatter},
    sync::Mutex,
    time::Instant,
};

use crate::error::HamsError;

pub trait XX {
    fn name(&self) -> Result<String, HamsError>;
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
    pub fn for_hc<W>(probe: W) -> *mut Self
    where
        W: XX + Send + Sync + 'static,
    {
        let repr = Repr {
            base: Self::vtable::<W>(),
            probe,
        };

        let boxed = Box::into_raw(Box::new(repr));

        boxed as *mut _
    }

    fn vtable<W: XX + 'static>() -> Self {
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

unsafe fn name<W: XX>(handle: *mut HealthProbe2) -> Result<String, HamsError> {
    auto_poison!(handle, {
        let repr = &mut *(handle as *mut Repr<W>);
        repr.probe.name()
    })
}

unsafe fn check<W: XX>(handle: *mut HealthProbe2, time: Instant) -> Result<bool, HamsError> {
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
