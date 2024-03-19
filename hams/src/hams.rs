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
    // healthcheck::{HealthCheck, HealthCheckResults, HealthCheckWrapper, HealthSystemResult},
};

use libc::c_void;
use log::{error, info};
use std::mem;
use tokio::signal::unix::signal;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::{signal::unix::SignalKind, sync::mpsc};

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

    /// Callback to be called on shutdown
    pub(crate) shutdown_cb: Arc<Mutex<Option<HamsCallback>>>,
    /// joinhandle to wait when shutting down service
    thread_jh: Arc<Mutex<Option<JoinHandle<()>>>>,
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
            port: 8079,
            alive: HealthCheck::new("alive"),
            ready: HealthCheck::new("ready"),
            shutdown_cb: Arc::new(Mutex::new(None)),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn register_shutdown(&self, user_data: *mut c_void, cb: unsafe extern "C" fn(*mut c_void)) {
        println!("Add shutdown to {}", self.name);
        *self.shutdown_cb.lock().unwrap() = Some(HamsCallback { user_data, cb });
    }

    pub fn start(&mut self) -> Result<(), HamsError> {
        info!("started hams {}", self.name);
        self.running.store(true, Ordering::Relaxed);

        let (channel_kill, rx_kill) = mpsc::channel::<()>(1);
        *self.kill.lock().unwrap() = Some(channel_kill);
        // *self.kill = Some(Mutex::new(channel_kill));

        // let (thread_tx, thread_rx) = sync::mpsc::channel::<()>();
        // *self.thread_tx.lock().unwrap()=Some(thread_tx);

        // Create a clone of self to be owned by the thread
        let mut thread_self = self.clone();
        info!("Original thread: {:?}", thread::current().id());

        let new_hams_thread = thread::spawn(move || {
            println!("Hello from thread");
            println!("Have thread_self here {:?}", thread_self);
            thread_self.start_tokio(rx_kill);

            info!("Thread loop is complete");
        });

        *self.thread_jh.lock().unwrap() = Some(new_hams_thread);

        Ok(())
    }

    fn start_tokio(&mut self, rx_kill: Receiver<()>) {
        info!("starting Tokio");

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Runtime created in current thread");
        let _guard = rt.enter();

        rt.block_on(self.start_async(rx_kill));

        info!("Tokio ended");
    }

    pub fn stop(&mut self) -> Result<(), HamsError> {
        info!("stopped hams {}", self.name);

        // let ben = self.thread_tx.lock().unwrap().as_ref().unwrap().clone();
        // ben.send(()).expect("Sent close message");
        info!("Close sent");

        let mut tempval = self.thread_jh.lock().expect("got thread");
        let old_thread = mem::replace(&mut *tempval, None);

        let mut temp_kill = self.kill.as_ref().lock().expect("got the kill");
        let old_kill = mem::replace(&mut *temp_kill, None);

        info!("Sending soft KILL signal");
        old_kill
            .unwrap()
            .blocking_send(())
            .expect("Send close to async");

        match old_thread {
            Some(jh) => {
                println!("have found a thread joinhandle");
                jh.join().expect("Thread is joined");
            }
            None => println!("Thread not started"),
        }

        Ok(())
    }

    async fn start_async(&mut self, mut kill_signal: Receiver<()>) {
        info!("Starting ASYNC");

        // Put code here to spawn the service parts (ie hams service)
        // for each servcie get a channel to allow us to shut it down
        // and when spawning save the handle to allow us to wait on it finishing.

        let (shutdown_health, shutdown_health_recv) = mpsc::channel(1);
        let health_listen = service_listen(self.clone(), shutdown_health_recv).await;

        // let channels_register = self.channels.clone();

        let my_running = self.running.clone();
        let my_shutdown_cb = self.shutdown_cb.clone();

        info!("Starting Tokio spawn");

        let mut sig_terminate =
            signal(SignalKind::terminate()).expect("Register terminate signal handler");
        let mut sig_quit = signal(SignalKind::quit()).expect("Register quit signal handler");
        let mut sig_hup = signal(SignalKind::hangup()).expect("Register hangup signal handler");
        info!("registered signal handlers");

        while my_running.load(Ordering::Relaxed) {
            info!("Waiting on signal handlers");
            tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        info!("Received ctrl-c signal: {:?}", my_shutdown_cb);
                        Hams::tigger_callback(my_shutdown_cb.clone());
                        // self.tigger_callback();
                    },
                    _ = kill_signal.recv() => {
                        info!("Received kill from library");
                        my_running.store(false, Ordering::Relaxed);
                    },
                    // _ = rx_http_kill.recv() => info!("Received HTTP kill signal"),
                    _ = sig_terminate.recv() => {
                        info!("Received TERM signal");
                        Hams::tigger_callback(my_shutdown_cb.clone());
                    },
                    _ = sig_quit.recv() => {
                        info!("Received QUIT signal");
                    Hams::tigger_callback(my_shutdown_cb.clone());
                    },
                    _ = sig_hup.recv() => {
                        info!("Received HUP signal");
                        Hams::tigger_callback(my_shutdown_cb.clone());
                    },
            };
        }

        shutdown_health
            .send(())
            .await
            .expect("Shutdown sent to listen");

        health_listen.await.expect("health_listen is complete");
        info!("start_async is now complete for health");
    }

    pub fn tigger_callback(shutdown_cb: Arc<Mutex<Option<HamsCallback>>>) {
        match shutdown_cb.lock().unwrap().as_ref() {
            Some(hams_callback) => {
                info!("Executing CB");
                unsafe { (hams_callback.cb)(hams_callback.user_data) };
            }
            None => error!("Call shutdown CB with None"),
        }
    }
}

async fn shutdown(channels: Arc<Mutex<Vec<mpsc::Sender<()>>>>) {
    let channels = channels.lock().unwrap().clone();

    for channel in channels.iter() {
        let channel_rx = channel.send(()).await;
        match channel_rx {
            Ok(_v) => info!("Shutdown signal sent"),
            Err(e) => info!("Error sending close signal: {:?}", e),
        }
    }
}

#[cfg(feature = "warp")]
/**
Start the port listening and exposing the service on it
*/
pub async fn service_listen<'a>(
    hams: Hams,
    mut kill_recv: Receiver<()>,
) -> tokio::task::JoinHandle<()> {
    use crate::webservice::hams_service;

    let temp_hams = hams.clone();
    // TODO:  use a direct clone not temp
    let api = hams_service(temp_hams);

    let (_addr, server) =
        warp::serve(api).bind_with_graceful_shutdown(([0, 0, 0, 0], hams.port), async move {
            kill_recv.recv().await;
        });

    info!("Serving service ({}) on port {}", hams.name, hams.port);
    tokio::task::spawn(server)
}

#[cfg(test)]
mod tests {
    use crate::health::probe::{manual::Manual, BoxedHealthProbe};

    use super::*;
    use std::time::Duration;

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
