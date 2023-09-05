use log::{error, info};
use serde::Serialize;
use std::alloc::Layout;
use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{Error, ErrorKind};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;

use crate::error::HamsError;

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

pub trait Health {
    fn name(&self) -> Result<usize, HamsError>;
    fn check(&self) -> Result<HealthCheckResult, HamsError>;
}

// What about creating HealthCheck just like FileHandle is created.
//  That way we can also create a valid health check element from outside our package

#[repr(C)]
pub struct HealthCheck {
    pub(crate) layout: Layout,
    pub(crate) type_id: TypeId,
    pub(crate) poisoned: bool,
    pub(crate) destroy: unsafe fn(*mut HealthCheck),
    pub(crate) name: unsafe fn(*mut HealthCheck) -> Result<usize, HamsError>,
    pub(crate) check: unsafe fn(*mut HealthCheck) -> Result<HealthCheckResult, HamsError>,
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

unsafe fn name<W: Health>(handle: *mut HealthCheck) -> Result<usize, HamsError> {
    auto_poison!(handle, {
        let repr = &mut *(handle as *mut Repr<W>);
        repr.health_check.name()
    })
}

unsafe fn check<W: Health>(handle: *mut HealthCheck) -> Result<HealthCheckResult, HamsError> {
    auto_poison!(handle, {
        let repr = &mut *(handle as *mut Repr<W>);
        repr.health_check.check()
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

struct HealthProbe {
    /// vector that is shared across clones AND the objects it refers to can also be independantly shared
    detail: Arc<Mutex<HashSet<HealthCheck>>>,
    /// Previous assignment of alive to allow state change operations
    previous: Arc<AtomicBool>,
    /// enable alive reply or disable (for debug use)
    enabled: Arc<AtomicBool>,
}

impl HealthProbe {
    fn new() -> HealthProbe {
        info!("Constructing HealthProbe");

        HealthProbe {
            detail: Arc::new(Mutex::new(HashSet::new())),
            previous: Arc::new(AtomicBool::new(false)),
            enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    fn insert(&self, hc: HealthCheck) -> Result<(), HamsError> {
        println!("implement this");
        // self.detail
        //     .lock()
        //     .unwrap()
        //     .insert(hc);
        Ok(())
    }
    fn remove(&self, hc: &HealthCheck) -> Result<(), HamsError> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct HealthKick {
    pub name: String,
}

impl Health for HealthKick {
    fn name(&self) -> Result<usize, HamsError> {
        Ok(3)
    }

    fn check(&self) -> Result<HealthCheckResult, HamsError> {
        Ok(HealthCheckResult {
            name: self.name.clone(),
            valid: true,
        })
    }
}

impl HealthKick {
    /// Create an alive kicked object providing name and duration of time before triggering failure
    pub fn new<S: Into<String>>(name: S, margin: Duration) -> Self {
        Self { name: name.into() }
    }
    pub fn kick(&self) {
        println!("Kicking my {}", self.name);
    }
}

impl Drop for HealthKick {
    fn drop(&mut self) {
        println!("Dropping my kick {}", self.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashSet, time::Duration};

    // #[test]
    // fn health_create() {

    //     let kick = HealthKick::new("black", Duration::from_secs(3));

    //     let hc0 = HealthCheck::for_hc(kick);

    //     // let ben = (*hc0).name;

    // }

    #[test]
    fn kick_create_and_destroy() {
        let probe = HealthProbe::new();
    }

    #[test]
    fn kick_sample_usage() {
        let hc = HealthKick::new("banana", Duration::from_secs(10));
        hc.kick();
    }

    #[test]
    fn construct_probe_and_populate() {
        let probe = HealthProbe::new();

        let hc0 = HealthKick::new("banana0", Duration::from_secs(10));
        let hc1 = HealthKick::new("banana1", Duration::from_secs(10));
        let hc2 = HealthKick::new("banana2", Duration::from_secs(10));

        let mut myvec = Vec::new();

        hc0.kick();
        myvec.push(&hc0);
        myvec.push(&hc1);
        myvec.push(&hc2);

        println!("myvec = {:?}", myvec);

        // probe.insert(&hc);

        hc0.kick();

        println!("myvec = {:?}", myvec);
        // probe.remove(&hc);
        let me = myvec.remove(0);
        drop(me);
        drop(myvec);
        drop(hc0);
        // println!("myvec = {:?}", myvec);

        // probe.insert(&hc);

        // let hc = HealthCheck::new();
    }
}
