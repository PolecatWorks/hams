mod check;
pub mod config;
mod webservice;

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::{
    error::HamsError, hams::check::HealthCheck, probe::AsyncHealthProbe, tokio_tools::run_in_tokio,
};

use config::HamsConfig;
use libc::c_void;
use log::info;
use tokio::signal::unix::signal;

use tokio::signal::unix::SignalKind;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub(crate) struct HamsCallback {
    user_data: *mut c_void,
    cb: unsafe extern "C" fn(*mut c_void),
}

/// Manually provide the Send impl for HamsCallBack to indicate it is thread safe.
/// This is required because HamsCallback cannot automatically derive if it can support
/// Send impl.
unsafe impl Send for HamsCallback {}

#[derive(Debug, Clone)]
pub struct PrometheusCallback {
    pub my_cb: extern "C" fn(ptr: *const c_void) -> *mut libc::c_char,
    pub my_cb_free: extern "C" fn(*mut libc::c_char),
    pub state: *const c_void,
}

unsafe impl Send for PrometheusCallback {}

#[derive(Debug, Clone)]
pub struct Hams {
    /// Name of the application this HaMS is for
    pub(crate) name: String,
    /// Provide the version of the application
    pub(crate) version: String,
    /// Provide the version of the release of HaMS
    pub(crate) hams_version: String,
    /// Provide the name of the package
    pub(crate) hams_name: String,

    /// Provide the address on which to serve the HaMS readyness and liveness
    address: SocketAddr,

    // preflights run successfully before the service starts
    pub preflights: HealthCheck,
    // shutdowns run after the service has been requested to stop
    pub shutdowns: HealthCheck,

    pub alive: HealthCheck,
    pub ready: HealthCheck,

    /// Token to cancel the service
    cancellation_token: CancellationToken,

    /// Callback to be called on shutdown
    pub(crate) shutdown_cb: Arc<Mutex<Option<HamsCallback>>>,
    /// joinhandle to wait when shutting down service
    thread_jh: Arc<Mutex<Option<JoinHandle<Result<(), HamsError>>>>>,

    /// Callback to be called on prometheus
    pub(crate) prometheus_cb: Arc<Mutex<Option<PrometheusCallback>>>,
    // /// Tokio runtime
    // pub(crate) rt: Arc<Mutex<Option<tokio::runtime::Runtime>>>,
}

impl Hams {
    /// Returns a HaMS instance with the name given
    ///
    /// # Arguments
    ///
    /// * 'name' - A string slice that holds the name of the HaMS
    pub fn new(config: HamsConfig) -> Hams {
        let ct = CancellationToken::new();
        ct.cancel();
        Hams {
            name: config.name,
            version: "UNDEFINED".to_owned(),
            hams_version: env!("CARGO_PKG_VERSION").to_string(),
            hams_name: env!("CARGO_PKG_NAME").to_string(),

            thread_jh: Arc::new(Mutex::new(None)),

            cancellation_token: ct,
            address: config.address,

            preflights: HealthCheck::new("preflights"),
            shutdowns: HealthCheck::new("shutdowns"),

            alive: HealthCheck::new("alive"),
            ready: HealthCheck::new("ready"),
            shutdown_cb: Arc::new(Mutex::new(None)),
            // prometheus_cb: None,
            prometheus_cb: Arc::new(Mutex::new(None)),
            // rt: Arc::new(Mutex::new(None)),
        }
    }

    pub fn register_shutdown(
        &self,
        user_data: *mut c_void,
        cb: unsafe extern "C" fn(*mut c_void),
    ) -> Result<(), HamsError> {
        println!("Add shutdown to {}", self.name);

        *self.shutdown_cb.lock()? = Some(HamsCallback { user_data, cb });
        Ok(())
    }

    pub fn register_prometheus(
        &mut self,
        my_cb: extern "C" fn(ptr: *const c_void) -> *mut libc::c_char,
        my_cb_free: extern "C" fn(*mut libc::c_char),
        state: *const c_void,
    ) -> Result<(), HamsError> {
        println!("Add prometheus to {}", self.name);

        // self.prometheus_cb = Some(PrometheusCallback { my_cb, my_cb_free, state });
        *self.prometheus_cb.lock()? = Some(PrometheusCallback {
            my_cb,
            my_cb_free,
            state,
        });
        Ok(())
    }

    /// Deregister Prometheus
    pub fn deregister_prometheus(&mut self) -> Result<(), HamsError> {
        println!("Remove prometheus from {}", self.name);

        // self.prometheus_cb = None;
        *self.prometheus_cb.lock()? = None;
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), HamsError> {
        info!("Starting HaMS {}", self.name);

        if !self.cancellation_token.is_cancelled() {
            return Err(HamsError::AlreadyRunning);
        }
        self.cancellation_token = CancellationToken::new();

        // Create a clone of self to be owned by the thread
        let mut self_thread = self.clone();
        info!("Original thread: {:?}", thread::current().id());

        // Create a new thread into which we will create the HaMS service using Tokio runtime
        let thread_hams = thread::spawn(move || {
            info!("HaMS thread: {:?}", thread::current().id());

            run_in_tokio(self_thread.start_async(self_thread.cancellation_token.clone()))
        });

        *self.thread_jh.lock()? = Some(thread_hams);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), HamsError> {
        info!("Stopping hams {}", self.name);

        // Check if the cancellation token is already cancelled
        if self.cancellation_token.is_cancelled() {
            return Err(HamsError::NotRunning);
        }

        // get the thread join handle and wait for it to finish
        let mut temp_thread = self.thread_jh.lock()?;

        let thread = (*temp_thread).take().ok_or(HamsError::NoThread)?;

        info!("Got thread to wait on");

        self.cancellation_token.cancel();
        info!("Sent CT");

        // The ? operator returns the error if the thread join fails. The resultant return is still a result as the
        // thread itself has an Result return
        thread.join().map_err(|_e| {
            info!("Thread join error");
            HamsError::JoinError2
        })?
    }

    /// Insert probe to alive checks. Use BoxedHealthProbe to allow for FFI
    pub fn alive_insert(&mut self, probe: Box<dyn AsyncHealthProbe + 'static>) -> bool {
        self.alive.insert(probe)
    }

    /// Remove probe from alive checks, Use BoxedHealthProbe to allow for FFI
    pub fn alive_remove(&mut self, probe: &Box<dyn AsyncHealthProbe + 'static>) -> bool {
        self.alive.remove(probe)
    }

    /// Insert probe to ready checks. Use BoxedHealthProbe to allow for FFI
    pub fn ready_insert(&mut self, probe: Box<dyn AsyncHealthProbe + 'static>) -> bool {
        self.ready.insert(probe)
    }

    /// Remove probe from ready checks. Use BoxedHealthProbe to allow for FFI
    pub fn ready_remove(&mut self, probe: &Box<dyn AsyncHealthProbe + 'static>) -> bool {
        self.ready.remove(probe)
    }

    async fn start_async(&mut self, ct: CancellationToken) -> Result<(), HamsError> {
        info!("Starting ASYNC");

        // Put code here to spawn the service parts (ie hams service)
        // for each service get a channel to allow us to shut it down
        // and when spawning save the handle to allow us to wait on it finishing.

        let hams_webservice = webservice(self.clone(), ct.clone()).await;

        let my_shutdown_cb = self.shutdown_cb.clone();

        info!("Starting Tokio spawn");

        let mut sig_terminate = signal(SignalKind::terminate())?;
        let mut sig_quit = signal(SignalKind::quit())?;
        let mut sig_hup = signal(SignalKind::hangup())?;
        info!("registered signal handlers: TERM, QUIT, HUP");

        info!("Waiting on signal handlers");
        tokio::select! {
            ws = hams_webservice => {
                info!("Hams webservice completed");
                ws?;
            },
            _ = ct.cancelled() => {
                info!("Cancellation Token cancelled");
            },
            _ = tokio::signal::ctrl_c() => {
                info!("Received ctrl-c signal: {:?}", my_shutdown_cb);
            },
            _ = sig_terminate.recv() => {
                info!("Received SIGTERM");
            },
            _ = sig_quit.recv() => {
                info!("Received SIGQUIT");
            },
            _ = sig_hup.recv() => {
                info!("Received SIGHUP");
            },
        };
        info!("Signal handlers completed");
        // Send ct.cancel() in case we exited the select based on a signal
        ct.cancel();

        Hams::tigger_callback(my_shutdown_cb.clone())?;

        info!("start_async is now complete for HaMS {}", self.name);
        Ok(())
    }

    pub(crate) fn tigger_callback(
        shutdown_cb: Arc<Mutex<Option<HamsCallback>>>,
    ) -> Result<(), HamsError> {
        let hams_mg = shutdown_cb.lock()?;
        match hams_mg.as_ref() {
            Some(hams_callback) => {
                info!("Triggering shutdown callback");
                unsafe { (hams_callback.cb)(hams_callback.user_data) };
                info!("Completed shutdown callback")
            }
            None => {
                info!("No shutdown callback to trigger");
            }
        }

        Ok(())
    }
}

#[cfg(feature = "warp")]
/**
Start the port listening and exposing the service on it
*/
pub async fn webservice<'a>(hams: Hams, ct: CancellationToken) -> tokio::task::JoinHandle<()> {
    use webservice::hams_service;

    let api = hams_service(hams.clone());

    let (_addr, server) =
        warp::serve(api).bind_with_graceful_shutdown(hams.address, ct.cancelled_owned());

    info!("Serving HaMS ({}) on address {}", hams.name, hams.address);
    tokio::task::spawn(server)
}

#[cfg(test)]
mod tests {

    use crate::probe::{manual::Manual, FFIProbe};

    use super::*;
    use std::time::Duration;

    /// Create a hams then assign the prometheus callback
    /// Check that callback is responding correctly
    #[test]
    fn test_prometheus_callback() {
        let mut hams = Hams::new(HamsConfig::default());

        extern "C" fn prometheus(ptr: *const c_void) -> *mut libc::c_char {
            let state = unsafe { &*(ptr as *const String) };

            let prometheus = format!("test {state}");
            let c_str_prometheus = std::ffi::CString::new(prometheus).unwrap();

            c_str_prometheus.into_raw()
        }

        extern "C" fn prometheus_free(ptr: *mut libc::c_char) {
            unsafe {
                if !ptr.is_null() {
                    drop(std::ffi::CString::from_raw(ptr));
                }
            }
        }

        let prometheus_cb = PrometheusCallback {
            my_cb: prometheus,
            my_cb_free: prometheus_free,
            state: &"splat".to_string() as *const String as *const c_void,
        };

        hams.register_prometheus(
            prometheus_cb.my_cb,
            prometheus_cb.my_cb_free,
            prometheus_cb.state,
        )
        .expect("Registered prometheus");

        let prometheus_cb = hams.prometheus_cb.lock().unwrap();
        let prometheus_cb = prometheus_cb.as_ref().unwrap();

        let ptr = (prometheus_cb.my_cb)(prometheus_cb.state);
        let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
        let str_slice = c_str.to_str().unwrap();
        assert_eq!(str_slice, "test splat");

        (prometheus_cb.my_cb_free)(ptr);
    }

    /// Create a hams then start and stop it
    #[cfg_attr(miri, ignore)]
    #[test]
    fn obj_hams_start_stop() {
        let mut hams = Hams::new(HamsConfig::default());
        hams.start().expect("Started");
        thread::sleep(Duration::from_secs(1));
        hams.stop().expect("Stopped");
    }

    /// Test add and remove alive and ready checks
    #[test]
    fn test_hams_health() {
        let mut hams = Hams::new(HamsConfig::default());

        let probe0 = Manual::new("test_probe0", true);
        let probe1 = Manual::new("test_probe1", true);

        assert_eq!(hams.alive.len(), 0);
        assert!(hams.alive_insert(FFIProbe::from(probe0.clone()).into()));
        assert_eq!(hams.alive.len(), 1);

        assert!(hams.alive.insert(FFIProbe::from(probe1.clone()).into()));
        assert_eq!(hams.alive.len(), 2);

        assert_eq!(hams.ready.len(), 0);
        assert!(hams.ready_insert(FFIProbe::from(probe0.clone()).into()));
        assert_eq!(hams.ready.len(), 1);

        assert!(hams.ready.insert(FFIProbe::from(probe1.clone()).into()));
        assert_eq!(hams.ready.len(), 2);

        assert!(hams.alive_remove(&FFIProbe::from(probe0.clone()).into()));
        assert_eq!(hams.alive.len(), 1);

        assert!(hams.ready_remove(&FFIProbe::from(probe0.clone()).into()));
        assert_eq!(hams.ready.len(), 1);
    }
}
