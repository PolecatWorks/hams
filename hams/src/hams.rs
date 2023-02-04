use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use futures::future;
use log::info;
use std::{mem, thread};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use warp::Filter;

use crate::error::HamsError;

/// A HaMS provides essential facilities to support a k8s microservice.
/// health, liveness, startup, shutdown, monitoring, logging
#[derive(Debug, Clone)]
pub struct Hams {
    /// A HaMS has a nmae which is used for distinguishing it on APIs
    pub name: String,
    pub base_path: String,
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
            port: 8079,
            base_path: "health".to_string(),
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

    async fn start_async(&mut self, mut kill_signal: Receiver<()>) {
        info!("Starting ASYNC");

        // Put code here to spawn the service parts (ie hams service)
        // for each servcie get a channel to allow us to shut it down
        // and when spawning save the handle to allow us to wait on it finishing.

        let (channel_health, kill_recv_health) = mpsc::channel(1);
        let health_listen = self.service_listen(kill_recv_health).await;

        self.add_picosvc(channel_health, health_listen);

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
            Hams::shutdown(channels_register).await;
            info!("my_services complete");
        });

        my_services
            .await
            .expect("Barried completes from a signal or service shutdown (explicit kill)");

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

    /// Start the port listening and exposing the servcie on it
    pub async fn service_listen(&self, mut kill_recv: Receiver<()>) -> tokio::task::JoinHandle<()> {
        let api = self.hams_service();

        let routes = api.with(warp::log("hams"));

        let (_addr, server) = warp::serve(routes).bind_with_graceful_shutdown(
            ([0, 0, 0, 0], self.port),
            async move {
                kill_recv.recv().await;
            },
        );

        info!("Serving service ({}) on port {}", self.name, self.port);
        tokio::task::spawn(server)
    }

    fn add_picosvc(&self, channel: Sender<()>, handle: tokio::task::JoinHandle<()>) {
        self.handles.lock().unwrap().push(handle);
        self.channels.lock().unwrap().push(channel);
    }

    fn with_name(
        &self,
    ) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
        let myname = self.name.clone();
        warp::any().map(move || myname.clone())
    }

    // fn  with_so_services(
    //     &self,
    // ) -> impl Filter<Extract = (Arc<Mutex<HashMap<String, Box<SoService>>>>,), Error = std::convert::Infallible> + Clone
    // {
    //     let so_services = self.so_services.clone();
    //     warp::any().map(move || so_services.clone())
    // }

    fn with_hams(
        &self,
    ) -> impl Filter<Extract = (Hams,), Error = std::convert::Infallible> + Clone {
        let my_hams = self.clone();
        warp::any().map(move || my_hams.clone())
    }

    // pub fn new_service(&self, ) -> impl Filter<Extract = impl warp::Reply, Error=warp::Rejection> + Clone + 'a {
    pub fn new_service(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("version").and(self.with_hams()).map(|hams| {
            info!("Looking at hams = {:?}", hams);
            format!("ALOOH")
        })
    }

    pub fn hams_service(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let svc_name = self.name.clone();
        let version = warp::path("version").map(|| "v1");
        let name = warp::path("name").map(move || svc_name.clone());
        let alive = warp::path("alive")
            .and(self.with_hams())
            .and_then(handlers::alive_handler);
        let ready = warp::path("ready")
            .and(self.with_hams())
            .and_then(handlers::ready_handler);

        warp::path("health").and(version.or(name).or(alive).or(ready))
    }

    pub fn service_x(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'a {
        warp::path(self.base_path.clone())
            .and(warp::path(self.version.clone()))
            .and(warp::get())
            .and(self.with_name())
            .and(warp::path::param())
            // .and(self.with_so_services())
            .map(|name, label: String| {
                // let pservicecount=so_services.lock().unwrap().iter().count();
                // let mytest = so_services.lock().unwrap().iter().map(|(s,_)| &**s).collect::<Vec<_>>().join("-");
                format!("Hello {}, whose agent is {}", name, label)
            })
    }
}

/// Handlers for health system
mod handlers {
    use super::Hams;
    use serde::Serialize;
    use std::convert::Infallible;

    /// Detail structure for ready and alive
    #[derive(Serialize)]
    struct HealthCheckResult {
        name: String,
        valid: bool,
    }

    /// Return structure for alive and ready endpoints
    #[derive(Serialize)]
    struct HealthSystemResult {
        name: String,
        valid: bool,
        detail: Vec<HealthCheckResult>,
    }

    /// Handler for alive endpoint
    pub async fn alive_handler(hams: Hams) -> Result<impl warp::Reply, Infallible> {
        let our_ids = HealthSystemResult {
            name: "alive".to_owned(),
            valid: true,
            detail: vec![],
        };

        Ok(warp::reply::with_status(
            warp::reply::json(&our_ids),
            warp::http::StatusCode::OK,
        ))
    }

    /// Handler for ready endpoint
    pub async fn ready_handler(hams: Hams) -> Result<impl warp::Reply, Infallible> {
        let our_ids = HealthSystemResult {
            name: "ready".to_owned(),
            valid: true,
            detail: vec![],
        };

        Ok(warp::reply::with_status(
            warp::reply::json(&our_ids),
            warp::http::StatusCode::OK,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use futures::Future;
    use warp::hyper::{body, Client, StatusCode};

    use super::*;

    #[ignore]
    #[test]
    fn init_start_stop() {
        let mut my_hams = Hams::new("apple");

        my_hams.start().expect("Hams started");

        my_hams.stop().expect("Hams stopped");

        drop(my_hams);
    }

    /// Dispatch instructions to a tokio runtime using an async thread
    fn tokio_async<F, C>(operation: C)
    where
        F: Future<Output = ()>,
        C: FnOnce() -> F + 'static,
        C: Send,
    {
        let client = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Runtime created in current thread");
            let _guard = rt.enter();

            rt.block_on(async {
                println!("in the tokio runtime");

                operation().await
            });

            println!("Client complete");
        });

        client.join().expect("Couldnt join thread");
    }

    /// Tests create their own async environment for calling async APIs
    #[test]
    fn api_calls() {
        let mut my_hams = Hams::new("apple");

        my_hams.start().expect("Client started");

        let test_hams = my_hams.clone();

        #[derive(Debug)]
        struct TestReply {
            status: StatusCode,
            body: String,
        }

        let testvals = HashMap::from([
            (
                "health/name",
                TestReply {
                    status: StatusCode::OK,
                    body: String::from("apple"),
                },
            ),
            (
                "health/version",
                TestReply {
                    status: StatusCode::OK,
                    body: String::from("v1"),
                },
            ),
            (
                "health/alive",
                TestReply {
                    status: StatusCode::OK,
                    body: String::from("{\"name\":\"alive\",\"valid\":true,\"detail\":[]}"),
                },
            ),
            (
                "health/ready",
                TestReply {
                    status: StatusCode::OK,
                    body: String::from("{\"name\":\"ready\",\"valid\":true,\"detail\":[]}"),
                },
            ),
        ]);

        tokio_async(|| async move {
            let client = Client::new();

            for (path, expected_response) in testvals {
                println!("Checking {} = {:?}", path, expected_response);

                let uri = format!("http://localhost:8079/{}", path).parse().unwrap();
                let response = client.get(uri).await.unwrap();

                assert_eq!(expected_response.status, response.status());
                let body = body::to_bytes(response.into_body()).await.unwrap();

                assert_eq!(expected_response.body, body);
            }
        });

        my_hams.stop().unwrap();

        drop(my_hams);
    }
}
