use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use futures::future;
use log::info;
use std::{mem, thread};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::error::HamsError;

/// A HaMS provides essential facilities to support a k8s microservice.
/// health, liveness, startup, shutdown, monitoring, logging
#[derive(Debug, Clone)]
pub struct Hams {
    /// A HaMS has a nmae which is used for distinguishing it on APIs
    pub name: String,
    // pub rt: tokio::runtime::Runtime,
    channels: Arc<Mutex<Vec<mpsc::Sender<()>>>>,
    handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    // so_services: Arc<Mutex<HashMap<String, Box<SoService>>>>,
    // liveness: HealthCheck,
    // readyness: HealthCheck,
    kill: Arc<Mutex<Option<Sender<()>>>>,
    /// Provide the version of the release of HaMS
    version: String,
    /// Provide the port on which to serve the HaMS readyness and liveness
    port: u16,
    thread_jh: Arc<Mutex<Option<JoinHandle<()>>>>,
    // thread_tx: Mutex<Option<sync::mpsc::Sender<()>>>,
}

impl<'a> Hams {
    /// Returns a HaMS instance with the name given
    ///
    /// # Arguments
    ///
    /// * 'name' - A string slice that holds the name of the HaMS
    pub fn new(name: &str) -> Hams {
        Hams {
            name: name.to_string(),
            thread_jh: Arc::new(Mutex::new(None)),
            // thread_tx: Mutex::new(None),
            channels: Arc::new(Mutex::new(vec![])),
            handles: Arc::new(Mutex::new(vec![])),
            kill: Arc::new(Mutex::new(None)),
            version: "v1".to_string(),
            port: 8080,
        }
    }

    pub fn start(&mut self) -> Result<(), HamsError> {
        info!("started hams {}", self.name);

        let (channel_kill, rx_kill) = mpsc::channel::<()>(1);
        *self.kill.lock().unwrap() = Some(channel_kill);
        // *self.kill = Some(Mutex::new(channel_kill));

        // let (thread_tx, thread_rx) = sync::mpsc::channel::<()>();
        // *self.thread_tx.lock().unwrap()=Some(thread_tx);

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
    async fn join(&self) {
        let mut handles = self
            .handles
            .lock()
            .expect("Could not lock mutex for handles");
        info!("Waiting for services: {:?}", handles);
        future::join_all(mem::take(&mut *handles)).await;
        info!("Services completed");
    }

    async fn start_async(&mut self, mut kill_signal: Receiver<()>) {
        info!("Starting ASYNC");

        let channels_register = self.channels.clone();

        let my_services = tokio::spawn(async move {
            // TODO: Check if this future should be waited via the join
            info!("Starting Tokio spawn");

            let mut sig_terminate =
                signal(SignalKind::terminate()).expect("Register terminate signal handler");
            let mut sig_quit = signal(SignalKind::quit()).expect("Register quit signal handler");
            let mut sig_hup = signal(SignalKind::hangup()).expect("Register hangup signal handler");

            info!("registered signal handlers");

            tokio::select! {
                _ = tokio::signal::ctrl_c() => info!("Received ctrl-c signal"),
                _ = kill_signal.recv() => info!("Received kill from library"),
                // _ = rx_http_kill.recv() => info!("Received HTTP kill signal"),
                _ = sig_terminate.recv() => info!("Received TERM signal"),
                _ = sig_quit.recv() => info!("Received QUIT signal"),
                _ = sig_hup.recv() => info!("Received HUP signal"),
            };
            info!("Signal handler triggered to start Shutdown");

            // Once signal handlers have triggered shutdowns then send the kill signal to each registered shutdown
            Hams::shutdown(channels_register).await;
            info!("my_services complete");
        });

        my_services.await.expect("Thread completes");

        self.join().await;
        info!("start_async is now complete");
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
}
