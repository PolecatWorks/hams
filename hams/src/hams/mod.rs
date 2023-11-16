use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

use crate::{
    error::HamsError,
    health::{
        HealthCheck, HealthCheckReply, HealthCheckResults, HealthProbeInner, HealthProbeWrapper,
    },
    tokio_tools::run_in_tokio,
};

use libc::c_void;
use log::{error, info};
use serde::Serialize;

use tokio::signal::unix::signal;
use tokio::sync::mpsc::Receiver;
use tokio::{signal::unix::SignalKind, sync::mpsc};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
struct HamsCallback {
    user_data: *mut c_void,
    cb: unsafe extern "C" fn(*mut c_void),
}

/// Manually provide the Send impl for HamsCallBack to indicate it is thread safe.
/// This is required because HamsCallback cannot automatically derive if it can support
/// Send impl.
unsafe impl Send for HamsCallback {}

/// Define the version and package as an object that can be returned via health
#[derive(Serialize, Clone, Debug)]
pub struct HamsVersion {
    /// version from hams package in Cargo.toml
    version: String,
    /// version from package name in Cargo.toml
    package: String,
}

#[derive(Debug, Clone)]
pub struct Hams {
    /// A HaMS has a nmae which is used for distinguishing it on APIs
    pub name: String,

    /// Version of hams being used
    version: HamsVersion,

    /// Provide the port on which to serve the HaMS readyness and liveness
    port: u16,

    // Alive is a vector that is shared across clones AND the objects it refers to can also be independantly shared
    alive_old: Arc<Mutex<HashSet<HealthProbeWrapper>>>,
    alive_previous: Arc<AtomicBool>,
    ready_old: Arc<Mutex<HashSet<HealthProbeWrapper>>>,

    alive: HealthCheck,
    ready: HealthCheck,

    /// Callback to be called on shutdown
    shutdown_cb: Arc<Mutex<Option<HamsCallback>>>,
    /// joinhandle to wait when shutting down service
    thread_jh: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Value to indicate if service is running
    running: Arc<AtomicBool>,
    /// Cancellation token to enable easy shutdown
    ct: CancellationToken,
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

            version: HamsVersion {
                version: env!("CARGO_PKG_VERSION").to_string(),
                package: env!("CARGO_PKG_NAME").to_string(),
            },
            port: 8079,
            alive_old: Arc::new(Mutex::new(HashSet::new())),
            alive_previous: Arc::new(AtomicBool::new(false)),
            ready_old: Arc::new(Mutex::new(HashSet::new())),
            alive: HealthCheck::new("alive"),
            ready: HealthCheck::new("ready"),
            shutdown_cb: Arc::new(Mutex::new(None)),
            running: Arc::new(AtomicBool::new(false)),
            ct: CancellationToken::new(),
        }
    }

    pub fn register_shutdown(&self, user_data: *mut c_void, cb: unsafe extern "C" fn(*mut c_void)) {
        println!("Add shutdown to {}", self.name);
        *self.shutdown_cb.lock().unwrap() = Some(HamsCallback { user_data, cb });
    }

    pub fn add_ready(&self, newval: Box<dyn HealthProbeInner>) {
        self.ready_old
            .lock()
            .unwrap()
            .insert(HealthProbeWrapper(newval));
    }

    pub fn remove_ready(&mut self, ready: Box<dyn HealthProbeInner>) -> bool {
        let mut readys = self.ready_old.lock().unwrap();
        readys.remove(&HealthProbeWrapper(ready))
    }

    pub fn check_ready(&self) -> (bool, String) {
        let my_now = Instant::now();

        let my_lock = self.ready_old.lock().unwrap();

        let detail = my_lock
            .iter()
            .map(|health| health.check(my_now))
            .collect::<Vec<_>>();

        let valid = detail.iter().all(|result| result.valid);

        (
            valid,
            serde_json::to_string(&HealthCheckReply {
                name: "ready".to_owned(),
                valid,
                // detail,
            })
            .unwrap(),
        )
    }

    pub fn add_alive(&self, newval: Box<dyn HealthProbeInner>) {
        self.alive_old
            .lock()
            .unwrap()
            .insert(HealthProbeWrapper(newval));
    }

    pub fn remove_alive(&mut self, alive: Box<dyn HealthProbeInner>) -> bool {
        let mut alives = self.alive_old.lock().unwrap();
        alives.remove(&HealthProbeWrapper(alive))
    }

    pub fn check_alive(&self) -> (bool, String) {
        let my_now = Instant::now();

        let my_lock = self.alive_old.lock().unwrap();

        let detail = my_lock
            .iter()
            .map(|health| health.check(my_now))
            .collect::<Vec<_>>();

        let valid = detail.iter().all(|result| result.valid);
        if valid != self.alive_previous.load(Ordering::Relaxed) {
            info!(
                "Alive state changed to {} from {}",
                valid,
                HealthCheckResults(detail.clone())
            );
            self.alive_previous.store(valid, Ordering::Relaxed);
        }
        (
            valid,
            serde_json::to_string(&HealthCheckReply {
                name: "alive".to_owned(),
                valid,
                // detail,
                // detail: detail.into(),
            })
            .unwrap(),
        )
    }

    pub fn start(&mut self) -> Result<(), HamsError> {
        info!("started hams {}", self.name);
        self.running.store(true, Ordering::Relaxed);

        // Create a clone of self to be owned by the thread
        let mut thread_self = self.clone();

        let new_hams_thread = thread::spawn(move || {
            run_in_tokio(thread_self.start_async()).unwrap();
        });

        *self.thread_jh.lock().unwrap() = Some(new_hams_thread);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), HamsError> {
        info!("Stopping HaMS");

        self.ct.cancel();
        self.thread_jh
            .lock()?
            .take()
            .expect("take JH") // Uses Yeet which is unstable
            .join()
            .map_err(HamsError::JoinError)?;

        Ok(())
    }

    async fn start_async(&mut self) -> Result<(), HamsError> {
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
                    _ = self.ct.cancelled() => {
                        info!("Received Cancellation token: {:?}", my_shutdown_cb);
                        my_running.store(false, Ordering::Relaxed);
                    },
                    _ = tokio::signal::ctrl_c() => {
                        info!("Received ctrl-c signal: {:?}", my_shutdown_cb);
                        Hams::tigger_callback(my_shutdown_cb.clone());
                    },
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
        Ok(())
    }

    fn tigger_callback(shutdown_cb: Arc<Mutex<Option<HamsCallback>>>) {
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
    let temp_hams = hams.clone();
    // TODO:  use a direct clone not temp
    let api = warp_filters::hams_service(temp_hams);

    let (_addr, server) =
        warp::serve(api).bind_with_graceful_shutdown(([127, 0, 0, 1], hams.port), async move {
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
        let version = warp::path("version")
            .and(warp::get())
            .and(with_hams(hams.clone()))
            .and_then(warp_handlers::version);

        let alive = warp::path("alive")
            .and(warp::get())
            .and(with_hams(hams.clone()))
            .and_then(warp_handlers::alive);

        let ready = warp::path("ready")
            .and(warp::get())
            .and(with_hams(hams.clone()))
            .and_then(warp_handlers::ready);

        let shutdown = warp::path("shutdown")
            .and(warp::post())
            .and(with_hams(hams.clone()))
            .and_then(warp_handlers::shutdown);

        warp::path("hams").and(version.or(alive).or(ready).or(shutdown))
    }

    fn with_hams(
        hams: Hams,
    ) -> impl Filter<Extract = (Hams,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || hams.clone())
    }
}

#[cfg(feature = "warp")]
mod warp_handlers {
    use crate::health::HealthCheckReply;

    use super::{Hams, HamsVersion};
    use serde::Serialize;
    use std::convert::Infallible;

    impl warp::Reply for HamsVersion {
        fn into_response(self) -> warp::reply::Response {
            warp::reply::json(&self).into_response()
        }
    }

    /// Reply structure for Name endpoint
    #[derive(Serialize)]
    pub struct NameVersionReply {
        name: String,
        version: HamsVersion,
    }
    impl warp::Reply for NameVersionReply {
        fn into_response(self) -> warp::reply::Response {
            warp::reply::json(&self).into_response()
        }
    }

    /// Handler for name endpoint
    pub async fn version(hams: Hams) -> Result<NameVersionReply, Infallible> {
        Ok(NameVersionReply {
            name: hams.name,
            version: hams.version,
        })
    }

    /// Handler for shutdown endpoint
    pub async fn shutdown(hams: Hams) -> Result<NameVersionReply, Infallible> {
        Hams::tigger_callback(hams.shutdown_cb.clone());

        Ok(NameVersionReply {
            name: hams.name,
            version: hams.version,
        })
    }
    /// Call ready check and return result as HealthCheckReply
    pub async fn ready(hams: Hams) -> Result<HealthCheckReply, Infallible> {
        Ok(hams.ready.check_reply())
    }
    /// Call alive check and return result as HealthCheckReply
    pub async fn alive(hams: Hams) -> Result<HealthCheckReply, Infallible> {
        Ok(hams.alive.check_reply())
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{healthcheck::HealthCheckResult, healthkicked::AliveCheckKicked};

//     use super::*;
//     use std::time::Duration;

//     #[test]
//     fn hams_add_remove() {
//         let mut hams = Hams::new("apple");

//         let hc0 = AliveCheckKicked::new("Howdy", Duration::from_secs(20));
//         hams.add_alive(Box::new(hc0.clone()));
//         hams.print_names_alive();

//         let hc1 = AliveCheckKicked::new("Hellow", Duration::from_secs(20));
//         hams.add_alive(Box::new(hc1));
//         hams.print_names_alive();

//         assert_eq!(2, hams.alive.lock().unwrap().len());

//         println!("Removing {:?}", hc0);

//         let reply = hams.remove_alive(Box::new(hc0.clone()) as Box<dyn HealthCheck>);
//         if reply {
//             println!("removed some elements")
//         };
//         // println!("removed {} elements", if reply {"OK"});
//         // assert_eq!(1, reply.len());
//         assert!(reply);
//         assert_eq!(1, hams.alive.lock().unwrap().len());
//         // for removed in reply {
//         //     println!("removed => {:?}", removed.get_name());
//         // }
//         hams.print_names_alive();
//     }

//     #[derive(Debug)]
//     struct I {
//         name: String,
//     }

//     impl HealthCheck for I {
//         fn get_name(&self) -> &str {
//             println!("HealthCheck for I {}", self.name);
//             &self.name
//         }

//         fn check(&self, time: std::time::Instant) -> HealthCheckResult {
//             todo!()
//         }
//     }

//     #[derive(Debug)]
//     struct J {
//         name: String,
//     }
//     impl HealthCheck for J {
//         fn get_name(&self) -> &str {
//             println!("HealthCheck for J {}", self.name);
//             &self.name
//         }

//         fn check(&self, time: std::time::Instant) -> HealthCheckResult {
//             todo!()
//         }
//     }

//     #[test]
//     fn test_vec() {
//         let myvec = Hams::new("test");

//         myvec.add_alive(Box::new(AliveCheckKicked::new(
//             "sofa",
//             Duration::from_secs(10),
//         )));
//         myvec.add_alive(Box::new(J {
//             name: "hello".to_owned(),
//         }));

//         myvec.add_alive(Box::new(AliveCheckKicked::new(
//             "sofa",
//             Duration::from_secs(10),
//         )));

//         {
//             let newby = Box::new(I {
//                 name: "hello".to_owned(),
//             });

//             myvec.add_alive(newby);
//             myvec.add_alive(Box::new(AliveCheckKicked::new(
//                 "sofa",
//                 Duration::from_secs(10),
//             )));

//             myvec.add_alive(Box::new(AliveCheckKicked::new(
//                 "sofa",
//                 Duration::from_secs(10),
//             )));
//         }

//         myvec.print_names_alive();

//         println!(
//             "vecing done wtih size {}",
//             myvec.alive.lock().unwrap().len()
//         );
//     }
// }
