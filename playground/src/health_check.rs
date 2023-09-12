use serde::Serialize;
use std::{
    alloc::Layout,
    any::{Any, TypeId},
    fmt::{Debug, Display, Formatter},
    sync::Mutex,
    time::Instant,
};

use crate::{error::HamsError, health::HealthCheckResult};

pub trait Health {
    fn name(&self) -> Result<String, HamsError>;
    fn check(&self, time: Instant) -> Result<HealthCheckResult, HamsError>;
    fn previous(&self) -> Result<bool, HamsError>;
}

/// What about creating HealthCheck just like FileHandle is created.
///  That way we can also create a valid health check element from outside our package
#[repr(C)]
pub struct HealthCheck {
    pub(crate) layout: Layout,
    pub(crate) type_id: TypeId,
    pub(crate) poisoned: bool,
    pub(crate) destroy: unsafe fn(*mut HealthCheck),
    pub(crate) name: unsafe fn(*mut HealthCheck) -> Result<String, HamsError>,
    pub(crate) check:
        unsafe fn(*mut HealthCheck, time: Instant) -> Result<HealthCheckResult, HamsError>,
    pub(crate) previous: unsafe fn(*mut HealthCheck) -> Result<bool, HamsError>,
}

impl HealthCheck {
    pub fn for_hc<W>(health_check: W) -> *mut HealthCheck
    where
        W: Health + Send + Sync + 'static,
    {
        let repr = Repr {
            base: HealthCheck::vtable::<W>(),
            health_check,
        };

        let boxed = Box::into_raw(Box::new(repr));

        boxed as *mut _
    }
    fn vtable<W: Health + 'static>() -> HealthCheck {
        let layout = Layout::new::<Repr<W>>();
        let type_id = TypeId::of::<W>();

        HealthCheck {
            layout,
            type_id,
            poisoned: false,
            destroy: destroy::<W>,
            name: name::<W>,
            check: check::<W>,
            previous: previous::<W>,
        }
    }
}

impl Eq for HealthCheck {}

impl PartialEq for HealthCheck {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
            && self.destroy == other.destroy
            && self.name == other.name
            && self.check == other.check
    }
}

unsafe fn destroy<W>(handle: *mut HealthCheck) {
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

unsafe fn name<W: Health>(handle: *mut HealthCheck) -> Result<String, HamsError> {
    auto_poison!(handle, {
        let repr = &mut *(handle as *mut Repr<W>);
        repr.health_check.name()
    })
}

unsafe fn check<W: Health>(
    handle: *mut HealthCheck,
    time: Instant,
) -> Result<HealthCheckResult, HamsError> {
    auto_poison!(handle, {
        let repr = &mut *(handle as *mut Repr<W>);
        repr.health_check.check(time)
    })
}

unsafe fn previous<W: Health>(handle: *mut HealthCheck) -> Result<bool, HamsError> {
    auto_poison!(handle, {
        let repr = &mut *(handle as *mut Repr<W>);
        repr.health_check.previous()
    })
}

#[derive(Debug)]
pub struct Poisoned(Mutex<Box<dyn Any + Send + 'static>>);

impl From<Box<dyn Any + Send + 'static>> for Poisoned {
    fn from(payload: Box<dyn Any + Send + 'static>) -> Self {
        Poisoned(Mutex::new(payload))
    }
}

// impl HamsError for Poisoned {}

impl Display for Poisoned {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let payload = self.0.lock().unwrap();

        if let Some(s) = payload.downcast_ref::<&str>() {
            write!(f, "a panic occurred: {}", s)
        } else if let Some(s) = payload.downcast_ref::<String>() {
            write!(f, "a panic occurred: {}", s)
        } else {
            write!(f, "a panic occurred")
        }
    }
}

#[repr(C)]
pub(crate) struct Repr<W> {
    // SAFETY: The HealthCheck must be the first field so we can cast between
    // *mut Repr<W> and *mut HealthCheck
    pub(crate) base: HealthCheck,
    pub(crate) health_check: W,
}
