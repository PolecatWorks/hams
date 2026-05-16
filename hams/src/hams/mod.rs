mod check;
pub mod config;
mod webservice;

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::SystemTime,
};

use crate::{
    error::HamsError, hams::check::HealthCheck, probe::AsyncHealthProbe, tokio_tools::run_in_tokio,
};

use config::{HamsConfig, TaskConfig};
use futures::future::join_all;
use libc::c_void;
use log::{error, info};
use tokio::signal::unix::signal;

use std::ffi::CStr;
use std::fmt;
use tokio::signal::unix::SignalKind;
use tokio_util::sync::CancellationToken;

pub(crate) struct CallbackFn(pub Box<dyn Fn() + Send>);

impl fmt::Debug for CallbackFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CallbackFn")
    }
}

pub(crate) struct PrometheusFn(pub Box<dyn Fn() -> String + Send>);

impl fmt::Debug for PrometheusFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrometheusFn")
    }
}

/// Main struct for the Health and Monitoring System
#[derive(Debug, Clone)]
pub struct Hams {
    /// Name of the application this HaMS is for
    pub(crate) name: String,
    /// Provide the version of the application
    pub version: String,
    /// Provide the version of the release of HaMS
    pub hams_version: String,
    /// Provide the name of the package
    pub hams_name: String,

    /// Provide the address on which to serve the HaMS readyness and liveness
    address: SocketAddr,

    /// preflights run successfully before the service starts
    pub preflights: HealthCheck,
    /// shutdowns run after the service has been requested to stop
    pub shutdowns: HealthCheck,

    /// Liveness checks
    pub alive: HealthCheck,
    /// Readiness checks
    pub ready: HealthCheck,
    /// Startup checks
    pub startup: HealthCheck,

    /// Configurable startup tasks
    pub startup_tasks: Arc<Mutex<Vec<(Arc<dyn AsyncHealthProbe>, TaskConfig)>>>,
    /// Configurable shutdown tasks
    pub shutdown_tasks: Arc<Mutex<Vec<(Arc<dyn AsyncHealthProbe>, TaskConfig)>>>,

    /// Token to cancel the service
    cancellation_token: CancellationToken,

    /// Callback to be called on shutdown
    pub(crate) shutdown_cb: Arc<Mutex<Option<CallbackFn>>>,
    /// joinhandle to wait when shutting down service
    thread_jh: Arc<Mutex<Option<JoinHandle<Result<(), HamsError>>>>>,

    /// Callback to be called on prometheus
    pub(crate) prometheus_cb: Arc<Mutex<Option<PrometheusFn>>>,
}

impl Hams {
    /// Returns a HaMS instance with the name given
    ///
    /// # Arguments
    ///
    /// * 'name' - A string slice that holds the name of the HaMS
    pub fn new(config: HamsConfig) -> Hams {
        // Create a new CancellationToken and then cancel it so it cannot be used further but needs to be replaced
        let ct = CancellationToken::new();
        ct.cancel();
        Hams {
            name: config.name,
            version: config.version,
            hams_version: env!("CARGO_PKG_VERSION").to_string(),
            hams_name: env!("CARGO_PKG_NAME").to_string(),

            thread_jh: Arc::new(Mutex::new(None)),

            cancellation_token: ct,
            address: config.address,

            preflights: HealthCheck::new("preflights"),
            shutdowns: HealthCheck::new("shutdowns"),

            alive: HealthCheck::new("alive"),
            ready: HealthCheck::new("ready"),
            startup: HealthCheck::new("startup"),

            startup_tasks: Arc::new(Mutex::new(Vec::new())),
            shutdown_tasks: Arc::new(Mutex::new(Vec::new())),

            shutdown_cb: Arc::new(Mutex::new(None)),
            prometheus_cb: Arc::new(Mutex::new(None)),
        }
    }

    pub fn register_shutdown(
        &self,
        cb: unsafe extern "C" fn(*mut c_void),
        user_data: *mut c_void,
    ) -> Result<(), HamsError> {
        info!("Add shutdown to {}", self.name);

        let user_data_addr = user_data as usize;

        let closure = move || {
            unsafe { cb(user_data_addr as *mut c_void) };
        };

        *self.shutdown_cb.lock()? = Some(CallbackFn(Box::new(closure)));
        Ok(())
    }

    pub fn register_shutdown_closure<F>(&self, cb: F) -> Result<(), HamsError>
    where
        F: Fn() + Send + 'static,
    {
        info!("Add shutdown closure to {}", self.name);
        *self.shutdown_cb.lock()? = Some(CallbackFn(Box::new(cb)));
        Ok(())
    }

    pub fn deregister_shutdown(&self) -> Result<(), HamsError> {
        info!("Remove shutdown from {}", self.name);

        *self.shutdown_cb.lock()? = None;
        Ok(())
    }

    pub fn register_prometheus(
        &mut self,
        my_cb: extern "C" fn(ptr: *const c_void) -> *mut libc::c_char,
        my_cb_free: extern "C" fn(*mut libc::c_char),
        state: *const c_void,
    ) -> Result<(), HamsError> {
        info!("Add prometheus to {}", self.name);

        let state_addr = state as usize;

        let closure = move || -> String {
            unsafe {
                let ptr = my_cb(state_addr as *const c_void);
                if ptr.is_null() {
                    return String::new();
                }
                let c_str = CStr::from_ptr(ptr);
                let result = c_str.to_string_lossy().to_string();
                my_cb_free(ptr);
                result
            }
        };

        *self.prometheus_cb.lock()? = Some(PrometheusFn(Box::new(closure)));
        Ok(())
    }

    pub fn register_prometheus_closure<F>(&self, cb: F) -> Result<(), HamsError>
    where
        F: Fn() -> String + Send + 'static,
    {
        info!("Add prometheus closure to {}", self.name);
        *self.prometheus_cb.lock()? = Some(PrometheusFn(Box::new(cb)));
        Ok(())
    }

    /// Deregister Prometheus
    pub fn deregister_prometheus(&mut self) -> Result<(), HamsError> {
        info!("Remove prometheus from {}", self.name);

        *self.prometheus_cb.lock()? = None;
        Ok(())
    }

    pub fn startup_task_insert(&mut self, probe: Arc<dyn AsyncHealthProbe>, config: TaskConfig) {
        self.startup_tasks.lock().unwrap().push((probe, config));
    }

    pub fn shutdown_task_insert(&mut self, probe: Arc<dyn AsyncHealthProbe>, config: TaskConfig) {
        self.shutdown_tasks.lock().unwrap().push((probe, config));
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

    /// Insert probe to startup checks. Use BoxedHealthProbe to allow for FFI
    pub fn startup_insert(&mut self, probe: Box<dyn AsyncHealthProbe + 'static>) -> bool {
        self.startup.insert(probe)
    }

    /// Remove probe from startup checks. Use BoxedHealthProbe to allow for FFI
    pub fn startup_remove(&mut self, probe: &Box<dyn AsyncHealthProbe + 'static>) -> bool {
        self.startup.remove(probe)
    }

    /// Insert probe to preflight checks.
    pub fn preflight_insert(&mut self, probe: Box<dyn AsyncHealthProbe + 'static>) -> bool {
        self.preflights.insert(probe)
    }

    /// Insert probe to shutdown checks.
    pub fn shutdown_insert(&mut self, probe: Box<dyn AsyncHealthProbe + 'static>) -> bool {
        self.shutdowns.insert(probe)
    }

    async fn run_tasks(
        &self,
        tasks_mutex: &Arc<Mutex<Vec<(Arc<dyn AsyncHealthProbe>, TaskConfig)>>>,
        phase: &str,
    ) -> Result<(), HamsError> {
        // We use std::sync::Mutex here because we do NOT hold the lock across any .await points.
        // The critical section below (cloning probes/config and spawning tasks) is purely synchronous
        // and non-blocking, so the standard Mutex is more efficient than tokio::sync::Mutex.
        let tasks = tasks_mutex.lock().unwrap();

        if tasks.is_empty() {
            return Ok(());
        }

        info!("Running {} {} tasks", tasks.len(), phase);

        let mut futures = Vec::new();

        for (probe, config) in tasks.iter() {
            let probe = probe.clone();
            let config = *config;
            let phase = phase.to_string();

            futures.push(tokio::spawn(async move {
                let mut attempts = 0;
                let timeout_duration = std::time::Duration::from_millis(config.timeout_ms);

                loop {
                    attempts += 1;

                    let check_future = probe.check(SystemTime::now());
                    let result = tokio::time::timeout(timeout_duration, check_future).await;

                    let success = match result {
                        Ok(Ok(true)) => true,
                        Ok(Ok(false)) => false,
                        Ok(Err(_)) => false,
                        Err(_) => false, // Timeout
                    };

                    if success {
                        info!(
                            "Task {}/{} succeeded",
                            phase,
                            probe.name().unwrap_or_default()
                        );
                        return Ok(());
                    } else {
                        if attempts >= config.retries {
                            error!(
                                "Task {}/{} failed after {} attempts",
                                phase,
                                probe.name().unwrap_or_default(),
                                attempts
                            );
                            return Err(HamsError::Message(format!(
                                "Task {} failed",
                                probe.name().unwrap_or_default()
                            )));
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(config.sleep_ms)).await;
                    }
                }
            }));
        }

        // Explicitly drop the lock before we await anything.
        // This ensures we don't hold a sync mutex across an await point.
        drop(tasks);

        let results = join_all(futures).await;

        for res in results {
            match res {
                Ok(Ok(())) => continue,
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(HamsError::JoinError(e.into())),
            }
        }

        info!("All {} tasks completed successfully", phase);
        Ok(())
    }

    async fn start_async(&mut self, ct: CancellationToken) -> Result<(), HamsError> {
        info!("Starting ASYNC");

        // Run preflight checks. These must all pass before we continue.
        let preflight_results = self.preflights.check(SystemTime::now()).await;
        if !preflight_results.valid {
            error!("Preflight checks failed: {:?}", preflight_results);
            return Err(HamsError::PreflightCheck);
        }

        self.run_tasks(&self.startup_tasks, "Startup").await?;

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

        Hams::call_shutdown_callback(my_shutdown_cb.lock()?.as_ref())?;

        self.run_tasks(&self.shutdown_tasks, "Shutdown").await?;

        // Run final shutdown checks.
        let shutdown_results = self.shutdowns.check(SystemTime::now()).await;
        if !shutdown_results.valid {
            error!("Shutdown checks failed: {:?}", shutdown_results);
            // We don't return error here because we're already shutting down,
            // but we log it.
        }

        info!("start_async is now complete for HaMS {}", self.name);
        Ok(())
    }

    pub(crate) fn call_shutdown_callback(
        shutdown_cb: Option<&CallbackFn>,
    ) -> Result<(), HamsError> {
        match shutdown_cb {
            Some(callback) => {
                info!("Triggering shutdown callback");
                (callback.0)();
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

    let server = warp::serve(api)
        .bind(hams.address)
        .await
        .graceful(ct.cancelled_owned());
    // .run().await;

    info!("Serving HaMS ({}) on address {}", hams.name, hams.address);
    tokio::task::spawn(server.run())
}

#[cfg(test)]
mod tests {

    use crate::probe::{FFIProbe, manual::Manual};

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

        let state = "splat".to_string();

        hams.register_prometheus(
            prometheus,
            prometheus_free,
            &state as *const String as *const c_void,
        )
        .expect("Registered prometheus");

        let prometheus_cb = hams.prometheus_cb.lock().unwrap();
        let prometheus_cb = prometheus_cb.as_ref().unwrap();

        let result = (prometheus_cb.0)();
        assert_eq!(result, "test splat");
    }

    #[test]
    fn test_prometheus_closure() {
        let mut hams = Hams::new(HamsConfig::default());

        hams.register_prometheus_closure(|| "test closure".to_string())
            .expect("Registered prometheus closure");

        let prometheus_cb = hams.prometheus_cb.lock().unwrap();
        let prometheus_cb = prometheus_cb.as_ref().unwrap();

        let result = (prometheus_cb.0)();
        assert_eq!(result, "test closure");
    }

    /// Create a hams then start and stop it
    #[cfg_attr(miri, ignore)]
    #[test]
    fn obj_hams_start_stop() {
        let mut config = HamsConfig::default();
        config.address = "0.0.0.0:0".parse().unwrap();
        let mut hams = Hams::new(config);
        hams.start().expect("Started");
        thread::sleep(Duration::from_secs(1));
        hams.stop().expect("Stopped");

        hams.start().expect("Started");
        hams.stop().expect("Stopped");

        hams.start().expect("Started");
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

        assert_eq!(hams.startup.len(), 0);
        assert!(hams.startup_insert(FFIProbe::from(probe0.clone()).into()));
        assert_eq!(hams.startup.len(), 1);

        assert!(hams.startup.insert(FFIProbe::from(probe1.clone()).into()));
        assert_eq!(hams.startup.len(), 2);

        assert!(hams.startup_remove(&FFIProbe::from(probe0.clone()).into()));
        assert_eq!(hams.startup.len(), 1);
    }

    /// Test shutdown callback updating the state

    #[test]
    fn test_hams_shutdown_callback_state() {
        let mut hams = Hams::new(HamsConfig::default());

        let mut state = 0;

        extern "C" fn shutdown_cb(ptr: *mut c_void) {
            let state = unsafe { &mut *(ptr as *mut i32) };
            *state += 1;
        }

        hams.register_shutdown(shutdown_cb, &mut state as *mut i32 as *mut c_void)
            .expect("Registered shutdown");

        Hams::call_shutdown_callback(hams.shutdown_cb.lock().unwrap().as_ref())
            .expect("Called shutdown");

        assert_eq!(state, 1);

        hams.deregister_shutdown().expect("Deregistered shutdown");
        Hams::call_shutdown_callback(hams.shutdown_cb.lock().unwrap().as_ref())
            .expect("Called shutdown");

        assert_eq!(state, 1);
    }

    #[test]
    fn test_hams_shutdown_closure_state() {
        let mut hams = Hams::new(HamsConfig::default());

        let state = Arc::new(Mutex::new(0));
        let state_clone = state.clone();

        hams.register_shutdown_closure(move || {
            let mut s = state_clone.lock().unwrap();
            *s += 1;
        })
        .expect("Registered shutdown closure");

        Hams::call_shutdown_callback(hams.shutdown_cb.lock().unwrap().as_ref())
            .expect("Called shutdown");

        assert_eq!(*state.lock().unwrap(), 1);
    }

    /// Test that startup and shutdown tasks actually run
    #[test]
    fn test_startup_shutdown_execution() {
        let mut config = HamsConfig::default();
        config.address = "0.0.0.0:0".parse().unwrap();
        let mut hams = Hams::new(config);

        // We use a manual probe that starts "true" (healthy)
        let startup_probe = Manual::new("startup_probe", true);
        let shutdown_probe = Manual::new("shutdown_probe", true);

        // Add startup task (should pass immediately)
        // Wrap in FFIProbe to adapt sync HealthProbe to AsyncHealthProbe
        hams.startup_task_insert(
            Arc::new(FFIProbe::from(startup_probe)),
            TaskConfig {
                // Short timeout, sleep, and 1 retry
                retries: 1,
                sleep_ms: 10,
                timeout_ms: 500,
            },
        );

        // Add shutdown task
        hams.shutdown_task_insert(
            Arc::new(FFIProbe::from(shutdown_probe)),
            TaskConfig {
                retries: 1,
                sleep_ms: 10,
                timeout_ms: 500,
            },
        );

        // Start hams (this runs startup tasks in async)
        hams.start().expect("Started");

        // Wait a bit for startup tasks to complete
        thread::sleep(Duration::from_millis(100));

        // Stop hams (this runs shutdown tasks)
        hams.stop().expect("Stopped");

        // Since we can't easily introspect the internal completion logs without mocking logger,
        // we're relying on the fact that start/stop didn't error.
        // A more robust test might check side effects if we had a probe that caused them.
    }
}
