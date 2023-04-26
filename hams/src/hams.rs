use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Instant,
};

use crate::{
    error::HamsError,
    healthcheck::{HealthCheck, HealthCheckWrapper, HealthSystemResult},
};
use futures::future;
use libc::c_void;
use log::{error, info, warn};
use std::mem;
use tokio::signal::unix::signal;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::{signal::unix::SignalKind, sync::mpsc};

#[derive(Debug)]
struct HamsCallback {
    user_data: *mut c_void,
    cb: unsafe extern "C" fn(*mut c_void),
}
/// Manually provide the Send impl for HamsCallBack to indicate it is thread safe.
/// This is required because HamsCallback cannot automatically derive if it can support
/// Send impl.
unsafe impl Send for HamsCallback {}

#[derive(Debug, Clone)]
pub struct Hams {
    /// A HaMS has a nmae which is used for distinguishing it on APIs
    pub name: String,

    // pub rt: tokio::runtime::Runtime,
    channels: Arc<Mutex<Vec<mpsc::Sender<()>>>>,
    handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,

    // Alive is a vector that is shared across clones AND the objects it refers to can also be independantly shared
    alive: Arc<Mutex<HashSet<HealthCheckWrapper>>>,
    ready: Arc<Mutex<HashSet<HealthCheckWrapper>>>,
    // ready: Arc<Mutex<Vec<Box<dyn HealthCheck>>>>,
    kill: Arc<Mutex<Option<Sender<()>>>>,

    /// Provide the version of the api
    api_version: String,

    /// Provide the version of the release of HaMS
    version: String,
    /// Provide the name of the package
    package: String,

    /// Callback to be called on shutdown
    shutdown_cb: Arc<Mutex<Option<HamsCallback>>>,

    /// Provide the port on which to serve the HaMS readyness and liveness
    port: u16,

    /// joinhandle to wait when shutting down service
    thread_jh: Arc<Mutex<Option<JoinHandle<()>>>>,
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
            thread_jh: Arc::new(Mutex::new(None)),

            channels: Arc::new(Mutex::new(vec![])),
            handles: Arc::new(Mutex::new(vec![])),
            kill: Arc::new(Mutex::new(None)),
            api_version: "v1".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            package: env!("CARGO_PKG_NAME").to_string(),
            port: 8079,
            alive: Arc::new(Mutex::new(HashSet::new())),
            ready: Arc::new(Mutex::new(HashSet::new())),
            shutdown_cb: Arc::new(Mutex::new(None)),
            // ready: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn register_shutdown(&self, user_data: *mut c_void, cb: unsafe extern "C" fn(*mut c_void)) {
        println!("Add shutdown to {}", self.name);
        *self.shutdown_cb.lock().unwrap() = Some(HamsCallback { user_data, cb });
    }

    fn add_ready(&self, newval: Box<dyn HealthCheck>) {
        self.ready
            .lock()
            .unwrap()
            .insert(HealthCheckWrapper(newval));
    }

    fn remove_ready(&mut self, ready: Box<dyn HealthCheck>) -> bool {
        let mut readys = self.ready.lock().unwrap();
        readys.remove(&HealthCheckWrapper(ready))
    }

    // fn add_ready(&self, newval: Box<dyn HealthCheck>) {
    //     self.ready.lock().unwrap().push(newval);
    // }

    // fn remove_ready(&mut self, my_val: Box<dyn HealthCheck>) -> Box<dyn HealthCheck> {

    //     let ready_locked = self.ready.lock().unwrap();

    //     let xyz = ready_locked.iter().enumerate() {}

    //     let remove_index = 12;
    //     ready_locked.remove(remove_index)

    //     // let remove_range = ready_locked.iter().map(f)

    //     // self.ready.lock().unwrap().drain_filter( |x|  {
    //     //     *my_val == **x
    //     // }).collect::<Vec<_>>()
    // }
    pub fn check_ready(&self) -> (bool, String) {
        let my_now = Instant::now();

        let my_lock = self.ready.lock().unwrap();

        let detail = my_lock
            .iter()
            .map(|health| health.check(my_now))
            .collect::<Vec<_>>();

        let valid = detail.iter().all(|result| result.valid);

        (
            valid,
            serde_json::to_string(&HealthSystemResult {
                name: "ready",
                valid,
                detail,
            })
            .unwrap(),
        )
    }
    fn print_names_ready(&self) {
        println!("Show ready:");
        let mylist = self.ready.lock().unwrap();
        for x in &*mylist {
            println!("> Ready: {}", x.get_name());
        }
    }

    pub fn add_alive(&self, newval: Box<dyn HealthCheck>) {
        self.alive
            .lock()
            .unwrap()
            .insert(HealthCheckWrapper(newval));
    }

    pub fn remove_alive(&mut self, alive: Box<dyn HealthCheck>) -> bool {
        let mut alives = self.alive.lock().unwrap();
        alives.remove(&HealthCheckWrapper(alive))
    }

    // fn remove_alive(&mut self, my_val: Box<dyn HealthCheck>) -> Vec<Box<dyn HealthCheck>> {

    //     self.alive.lock().unwrap().drain_filter( |x|  {
    //         *my_val == **x
    //     }).collect::<Vec<_>>()
    // }

    pub fn check_alive(&self) -> (bool, String) {
        let my_now = Instant::now();

        let my_lock = self.alive.lock().unwrap();

        let detail = my_lock
            .iter()
            .map(|health| health.check(my_now))
            .collect::<Vec<_>>();

        let valid = detail.iter().all(|result| result.valid);

        (
            valid,
            serde_json::to_string(&HealthSystemResult {
                name: "alive",
                valid,
                detail,
            })
            .unwrap(),
        )
    }
    fn print_names_alive(&self) {
        println!("Show alive:");
        let mylist = self.alive.lock().unwrap();
        for x in &*mylist {
            println!("> Alive: {}", x.get_name());
        }
    }

    pub fn start(&mut self) -> Result<(), HamsError> {
        info!("started hams {}", self.name);

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

        let (channel_health, kill_recv_health) = mpsc::channel(1);
        let health_listen = service_listen(self.clone(), kill_recv_health).await;

        // self.add_picosvc(channel_health, health_listen);

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

    /// Join all the services threads that have been added to the handle_list
    async fn join(&self) {
        let handle_list = mem::take(&mut *(self.handles.lock().expect("lock mutex for handles")));
        future::join_all(handle_list).await;

        // future::join_all(mem::take(&mut *(self.handles.lock().expect("lock mutex for handles")))).await;

        // let mut handles = self
        //     .handles
        //     .lock()
        //     .expect("Could not lock mutex for handles");
        // // info!("Waiting for services: {:?}", handles);
        // future::join_all(mem::take(&mut *handles)).await;
        info!("Services completed");
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
    use warp::Filter;

    let temp_hams = hams.clone();
    // TODO:  use a direct clone not temp
    let api = warp_filters::hams_service(temp_hams);

    let routes = api.with(warp::log("hams"));

    let (_addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], hams.port), async move {
            kill_recv.recv().await;
        });

    info!("Serving service ({}) on port {}", hams.name, hams.port);
    tokio::task::spawn(server)
}

#[cfg(feature = "axum")]
/// Start the port listening and exposing the service on it
pub async fn service_listen<'a>(
    hams: Hams,
    mut kill_recv: Receiver<()>,
) -> tokio::task::JoinHandle<()> {
    let temp_hams = hams.clone();

    let api = hams_service(temp_hams);

    let routes = api.with(warp::log("hams"));

    let (_addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], hams.port), async move {
            kill_recv.recv().await;
        });

    info!("Serving service ({}) on port {}", hams.name, hams.port);
    tokio::task::spawn(server)
}

#[cfg(feature = "warp")]
mod warp_filters {
    use warp::Filter;

    use super::{warp_handlers, Hams};

    pub fn hams_service(
        hams: Hams,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let name = warp::path("name")
            .and(warp::get())
            .and(with_hams(hams.clone()))
            .and_then(warp_handlers::name_handler);

        let shutdown = warp::path("shutdown")
            .and(with_hams(hams.clone()))
            .and_then(warp_handlers::shutdown_handler);

        let alive = warp::path("alive")
            .and(with_hams(hams.clone()))
            .and_then(warp_handlers::alive_handler);

        let ready = warp::path("ready")
            .and(with_hams(hams.clone()))
            .and_then(warp_handlers::ready_handler);

        let version = warp::path("version")
            .and(warp::get())
            .and(with_hams(hams.clone()))
            .and_then(warp_handlers::version_handler);

        warp::path("health").and(name.or(version).or(alive).or(ready).or(shutdown))
    }

    fn with_hams(
        hams: Hams,
    ) -> impl Filter<Extract = (Hams,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || hams.clone())
    }
}

#[cfg(feature = "warp")]
mod warp_handlers {
    use std::convert::Infallible;

    use log::{error, info};
    use serde::Serialize;

    use super::Hams;

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

    /// Reply structure for Name endpoint
    #[derive(Serialize)]
    struct VersionNameReply {
        name: String,
        version: String,
    }

    /// Handler for name endpoint
    pub async fn name_handler(hams: Hams) -> Result<impl warp::Reply, Infallible> {
        let name_reply = NameReply { name: hams.name };
        Ok(warp::reply::json(&name_reply))
    }

    /// Handler for version endpoint
    pub async fn version_handler(hams: Hams) -> Result<impl warp::Reply, Infallible> {
        let version_reply = VersionReply {
            version: hams.version,
        };
        Ok(warp::reply::json(&version_reply))
    }

    /// Handler for shutdown endpoint
    pub async fn shutdown_handler(hams: Hams) -> Result<impl warp::Reply, Infallible> {
        let shutdown_reply = VersionNameReply {
            version: hams.version,
            name: hams.name,
        };

        match hams.shutdown_cb.lock().unwrap().as_ref() {
            Some(hams_callback) => {
                info!("Executing CB");
                unsafe { (hams_callback.cb)(hams_callback.user_data) };
            }
            None => error!("Call shutdown CB with None"),
        }

        Ok(warp::reply::json(&shutdown_reply))
    }

    /// Handler for alive endpoint
    pub async fn alive_handler(hams: Hams) -> Result<impl warp::Reply, Infallible> {
        let (valid, content) = hams.check_alive();

        Ok(warp::reply::with_status(
            content,
            if valid {
                warp::http::StatusCode::OK
            } else {
                warp::http::StatusCode::NOT_ACCEPTABLE
            },
        ))
    }

    /// Handler for ready endpoint
    pub async fn ready_handler(hams: Hams) -> Result<impl warp::Reply, Infallible> {
        let (valid, content) = hams.check_ready();

        Ok(warp::reply::with_status(
            content,
            if valid {
                warp::http::StatusCode::OK
            } else {
                warp::http::StatusCode::NOT_ACCEPTABLE
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{healthcheck::HealthCheckResult, healthkicked::AliveCheckKicked};

    use super::*;
    use std::time::Duration;

    #[test]
    fn hams_add_remove() {
        let mut hams = Hams::new("apple");

        let hc0 = AliveCheckKicked::new("Howdy", Duration::from_secs(20));
        hams.add_alive(Box::new(hc0.clone()));
        hams.print_names_alive();

        let hc1 = AliveCheckKicked::new("Hellow", Duration::from_secs(20));
        hams.add_alive(Box::new(hc1));
        hams.print_names_alive();

        assert_eq!(2, hams.alive.lock().unwrap().len());

        println!("Removing {:?}", hc0);

        let reply = hams.remove_alive(Box::new(hc0.clone()) as Box<dyn HealthCheck>);
        if reply {
            println!("removed some elements")
        };
        // println!("removed {} elements", if reply {"OK"});
        // assert_eq!(1, reply.len());
        assert!(reply);
        assert_eq!(1, hams.alive.lock().unwrap().len());
        // for removed in reply {
        //     println!("removed => {:?}", removed.get_name());
        // }
        hams.print_names_alive();
    }

    #[derive(Debug)]
    struct I {
        name: String,
    }

    impl HealthCheck for I {
        fn get_name(&self) -> &str {
            println!("HealthCheck for I {}", self.name);
            &self.name
        }

        fn check(&self, time: std::time::Instant) -> HealthCheckResult {
            todo!()
        }
    }

    #[derive(Debug)]
    struct J {
        name: String,
    }
    impl HealthCheck for J {
        fn get_name(&self) -> &str {
            println!("HealthCheck for J {}", self.name);
            &self.name
        }

        fn check(&self, time: std::time::Instant) -> HealthCheckResult {
            todo!()
        }
    }

    #[test]
    fn test_vec() {
        let mut myvec = Hams::new("test");

        myvec.add_alive(Box::new(AliveCheckKicked::new(
            "sofa",
            Duration::from_secs(10),
        )));
        myvec.add_alive(Box::new(J {
            name: "hello".to_owned(),
        }));

        myvec.add_alive(Box::new(AliveCheckKicked::new(
            "sofa",
            Duration::from_secs(10),
        )));

        {
            let newby = Box::new(I {
                name: "hello".to_owned(),
            });

            myvec.add_alive(newby);
            myvec.add_alive(Box::new(AliveCheckKicked::new(
                "sofa",
                Duration::from_secs(10),
            )));

            myvec.add_alive(Box::new(AliveCheckKicked::new(
                "sofa",
                Duration::from_secs(10),
            )));
        }

        myvec.print_names_alive();

        println!(
            "vecing done wtih size {}",
            myvec.alive.lock().unwrap().len()
        );
    }
}
