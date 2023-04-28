use std::{
    mem,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use futures::future;
use tokio::signal::unix::signal;
use tokio::{
    signal::{self, unix::SignalKind},
    sync::mpsc,
};

use log::info;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::sampleerror::SampleError;

#[derive(Debug, Clone)]
pub struct Sample {
    count: Arc<Mutex<i32>>,
    name: String,
    port: u16,

    channels: Arc<Mutex<Vec<mpsc::Sender<()>>>>,
    handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    kill: Arc<Mutex<Option<Sender<()>>>>,
    /// Provide the port on which to serve the HaMS readyness and liveness

    /// joinhandle to wait when shutting down service
    thread_jh: Arc<Mutex<Option<JoinHandle<()>>>>,
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
    sample: Sample,
    mut kill_recv: Receiver<()>,
) -> tokio::task::JoinHandle<()> {
    use warp::Filter;

    let temp_hams = sample.clone();
    // TODO:  use a direct clone not temp
    let api = warp_filters::sample_service(temp_hams);

    let routes = api.with(warp::log("hams"));

    let (_addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], sample.port), async move {
            kill_recv.recv().await;
        });

    info!("Serving service ({}) on port {}", sample.name, sample.port);
    tokio::task::spawn(server)
}

impl Sample {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Sample {
            count: Arc::new(Mutex::new(0)),
            name: name.into(),

            kill: Arc::new(Mutex::new(None)),
            port: 8080,
            thread_jh: Arc::new(Mutex::new(None)),
            channels: Arc::new(Mutex::new(vec![])),
            handles: Arc::new(Mutex::new(vec![])),
        }
    }

    fn increment(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
    }

    fn get_count(&self) -> i32 {
        *self.count.lock().unwrap()
    }

    pub fn get_kill(&self) -> Arc<Mutex<Option<Sender<()>>>> {
        self.kill.clone()
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

    async fn start_async(&mut self, mut kill_signal: Receiver<()>) {
        info!("Starting ASYNC");
        let (channel_health, kill_recv_health) = mpsc::channel(1);
        let health_listen = service_listen(self.clone(), kill_recv_health).await;
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

            // Once any of the signal handlers have completed then send the kill signal to each registered shutdown
            shutdown(channels_register).await;
            info!("my_services complete");
        });

        my_services
            .await
            .expect("Barried completes from a signal or service shutdown (explicit kill)");

        self.join().await;
        info!("start_async is now complete");
    }
    async fn join(&self) {
        let handle_list = mem::take(&mut *(self.handles.lock().expect("lock mutex for handles")));
        future::join_all(handle_list).await;

        info!("Services completed");
    }

    pub fn start(&mut self) -> Result<(), SampleError> {
        info!("Started sample {}", self.name);

        let (channel_kill, rx_kill) = mpsc::channel::<()>(1);
        *self.kill.lock().unwrap() = Some(channel_kill);
        // *self.kill = Some(Mutex::new(channel_kill));

        // let (thread_tx, thread_rx) = sync::mpsc::channel::<()>();
        // *self.thread_tx.lock().unwrap()=Some(thread_tx);

        // Create a clone of self to be owned by the thread
        let mut thread_self = self.clone();
        info!("Original thread: {:?}", thread::current().id());

        let new_thread = thread::spawn(move || {
            println!("Hello from thread");
            println!("Have thread_self here {:?}", thread_self);
            thread_self.start_tokio(rx_kill);

            info!("Thread loop is complete");
        });

        *self.thread_jh.lock().unwrap() = Some(new_thread);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), SampleError> {
        info!("stopped sample {}", self.name);

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
}

#[cfg(feature = "warp")]
mod warp_filters {
    use warp::Filter;

    use super::{warp_handlers, Sample};

    pub fn sample_service(
        sample: Sample,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let name = warp::path("name")
            .and(warp::get())
            .and(with_sample(sample.clone()))
            .and_then(warp_handlers::name_handler);

        warp::path("abc").and(
            name, // .or(name)
                 // .or(alive)
                 // .or(ready)
        )
    }

    fn with_sample(
        sample: Sample,
    ) -> impl Filter<Extract = (Sample,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || sample.clone())
    }
}

#[cfg(feature = "warp")]
mod warp_handlers {
    use std::convert::Infallible;

    use serde::Serialize;

    use super::Sample;

    /// Reply structure for Version endpoint
    #[derive(Serialize)]
    struct VersionReply {
        version: String,
    }

    /// Reply structure for Name endpoint
    #[derive(Serialize)]
    struct NameReply {
        name: String,
    }

    /// Handler for name endpoint
    pub async fn name_handler(hams: Sample) -> Result<impl warp::Reply, Infallible> {
        let name_reply = NameReply { name: hams.name };
        Ok(warp::reply::json(&name_reply))
    }
}
