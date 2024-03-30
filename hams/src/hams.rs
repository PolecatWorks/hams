use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use crate::{
    error::HamsError,
    health::check::HealthCheck,
    tokio_tools::run_in_tokio_with_cancel,
    // healthcheck::{HealthCheck, HealthCheckResults, HealthCheckWrapper, HealthSystemResult},
};

use libc::c_void;
use log::info;
use tokio::signal::unix::signal;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::{signal::unix::SignalKind, sync::mpsc};
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
pub struct Hams {
    /// Name of the application this HaMS is for
    pub(crate) name: String,
    /// Provide the version of the application
    pub(crate) version: String,
    /// Provide the version of the release of HaMS
    pub(crate) hams_version: String,
    /// Provide the name of the package
    pub(crate) hams_name: String,

    /// Provide the port on which to serve the HaMS readyness and liveness
    port: u16,

    alive: HealthCheck,
    ready: HealthCheck,

    kill: Arc<Mutex<Option<Sender<()>>>>,
    /// Token to cancel the service
    cancellation_token: CancellationToken,

    /// Callback to be called on shutdown
    pub(crate) shutdown_cb: Arc<Mutex<Option<HamsCallback>>>,
    /// joinhandle to wait when shutting down service
    thread_jh: Arc<Mutex<Option<JoinHandle<Result<(), HamsError>>>>>,
    /// Value to indicate if service is running
    running: Arc<AtomicBool>,
}

impl Hams {
    /// Returns a HaMS instance with the name given
    ///
    /// # Arguments
    ///
    /// * 'name' - A string slice that holds the name of the HaMS
    pub fn new<S: Into<String>>(name: S) -> Hams {
        Hams {
            name: name.into(),
            version: "UNDEFINED".to_owned(),
            hams_version: env!("CARGO_PKG_VERSION").to_string(),
            hams_name: env!("CARGO_PKG_NAME").to_string(),

            thread_jh: Arc::new(Mutex::new(None)),

            // channels: Arc::new(Mutex::new(vec![])),
            // handles: Arc::new(Mutex::new(vec![])),
            kill: Arc::new(Mutex::new(None)),
            cancellation_token: CancellationToken::new(),
            port: 8079,
            alive: HealthCheck::new("alive"),
            ready: HealthCheck::new("ready"),
            shutdown_cb: Arc::new(Mutex::new(None)),
            running: Arc::new(AtomicBool::new(false)),
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

    pub fn start(&mut self) -> Result<(), HamsError> {
        info!("Starting HaMS {}", self.name);
        self.running.store(true, Ordering::Relaxed);

        let (channel_kill, rx_kill) = mpsc::channel::<()>(1);
        *self.kill.lock()? = Some(channel_kill);

        // Create a clone of self to be owned by the thread
        let mut thread_self = self.clone();
        info!("Original thread: {:?}", thread::current().id());

        // Create a new thread into which we will create the HaMS service using Tokio runtime
        let new_hams_thread = thread::spawn(move || {
            info!("HaMS thread: {:?}", thread::current().id());

            let x = run_in_tokio_with_cancel(
                thread_self.cancellation_token.clone(),
                thread_self.start_async(rx_kill),
            );

            let y = match x {
                Ok(t) => {
                    info!("Tokio thread finished: {:?}", t);

                    Ok(())
                }
                Err(e) => {
                    info!("Tokio thread finished with error: {:?}", e);
                    match e {
                        HamsError::Cancelled => {
                            info!("Cancelled");
                            Ok(())
                        }
                        _ => {
                            info!("Error: {}", e);
                            Err(e)
                        }
                    }
                }
            };
            y
        });

        *self.thread_jh.lock()? = Some(new_hams_thread);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), HamsError> {
        info!("Stopping hams {}", self.name);

        // // Get the kill channel and send a message to it
        // let mut temp_kill = self.kill.as_ref().lock()?;
        // let kill = (*temp_kill)
        //     .take()
        //     .ok_or(HamsError::OptionNone("No kill channel".to_owned()))?;

        // kill.blocking_send(())?;

        // get the thread join handle and wait for it to finish
        let mut temp_thread = self.thread_jh.lock()?;

        let thread = (*temp_thread)
            .take()
            .ok_or(HamsError::OptionNone("No thread to join".to_owned()))?;

        info!("Got thread to wait on");

        info!("About to send CT");
        self.cancellation_token.cancel();
        info!("Sent CT");

        // The ? operator returns the error if the thread join fails. The resultant return is still a result as the
        // thread itself has an Result return
        thread.join().map_err(|_e| {
            info!("Thread join error");
            HamsError::JoinError2
        })?
        // info!("thread joined");
    }

    async fn start_async(&mut self, mut kill_signal: Receiver<()>) -> Result<(), HamsError> {
        let ct = CancellationToken::new();

        info!("Starting ASYNC");

        // Put code here to spawn the service parts (ie hams service)
        // for each service get a channel to allow us to shut it down
        // and when spawning save the handle to allow us to wait on it finishing.

        let (shutdown_hams_api, shutdown_hams_api_recv) = mpsc::channel(1);
        let health_listen = webservice(self.clone(), shutdown_hams_api_recv).await;

        let my_running = self.running.clone();
        let my_shutdown_cb = self.shutdown_cb.clone();

        info!("Starting Tokio spawn");

        let mut sig_terminate = signal(SignalKind::terminate())?;
        let mut sig_quit = signal(SignalKind::quit())?;
        let mut sig_hup = signal(SignalKind::hangup())?;
        info!("registered signal handlers: TERM, QUIT, HUP");

        info!("Waiting on signal handlers");
        tokio::select! {
            _ = ct.cancelled() => {
                info!("Token cancelled");
                my_running.store(false, Ordering::Relaxed);
            },
            _ = kill_signal.recv() => {
                info!("Received kill from library");
                my_running.store(false, Ordering::Relaxed);
            },
            _ = tokio::signal::ctrl_c() => {
                info!("Received ctrl-c signal: {:?}", my_shutdown_cb);
                Hams::tigger_callback(my_shutdown_cb.clone())?;
            },
            _ = sig_terminate.recv() => {
                info!("Received TERM signal");
                Hams::tigger_callback(my_shutdown_cb.clone())?;
            },
            _ = sig_quit.recv() => {
                info!("Received QUIT signal");
                Hams::tigger_callback(my_shutdown_cb.clone())?;
            },
            _ = sig_hup.recv() => {
                info!("Received HUP signal");
                Hams::tigger_callback(my_shutdown_cb.clone())?;
            },
        };
        info!("Signal handlers completed");
        ct.cancel();
        // shutdown_hams_api.send(()).await?;

        health_listen.await?;

        info!("start_async is now complete for health");
        Ok(())
    }

    pub(crate) fn tigger_callback(
        shutdown_cb: Arc<Mutex<Option<HamsCallback>>>,
    ) -> Result<(), HamsError> {
        let hams_mg = shutdown_cb.lock()?;
        let hams_callback = hams_mg.as_ref().ok_or(HamsError::CallbackError)?;

        unsafe { (hams_callback.cb)(hams_callback.user_data) };

        Ok(())
    }
}

#[cfg(feature = "warp")]
/**
Start the port listening and exposing the service on it
*/
pub async fn webservice<'a>(
    hams: Hams,
    mut kill_recv: Receiver<()>,
) -> tokio::task::JoinHandle<()> {
    use crate::webservice::hams_service;

    let api = hams_service(hams.clone());

    let (_addr, server) =
        warp::serve(api).bind_with_graceful_shutdown(([0, 0, 0, 0], hams.port), async move {
            kill_recv.recv().await;
        });

    info!("Serving HaMS ({}) on port {}", hams.name, hams.port);
    tokio::task::spawn(server)
}

#[cfg(test)]
mod tests {
    use crate::health::probe::{manual::Manual, BoxedHealthProbe};

    use super::*;
    use std::time::Duration;

    /// This is a test to ensure that the CancellationToken can be cancelled multiple times
    #[test]
    fn test_cancel_token() {
        let ct = CancellationToken::new();
        ct.cancel();
        ct.cancel();
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_hams() {
        let mut hams = Hams::new("test");
        hams.start().expect("Started");
        thread::sleep(Duration::from_secs(1));
        hams.stop().expect("Stopped");
    }

    // Test add and remove alive and ready checks
    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_hams_health() {
        let hams = Hams::new("test");

        let probe = BoxedHealthProbe::new(Manual::new("test_probe", true));
        hams.alive.insert(probe);
        // hams.alive.remove(probe);

        let probe = BoxedHealthProbe::new(Manual::new("test_probe", true));
        hams.ready.insert(probe);
        // hams.ready.remove(probe);
    }
}
