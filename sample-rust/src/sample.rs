use std::net::SocketAddr;
use std::{
    mem,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use figment::{
    providers::{Format, Yaml},
    Figment,
};
use futures::future;
use serde::Deserialize;

use tokio::sync::mpsc;

use log::{error, info};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::sampleerror::SampleError;

const TRACE_HEADERS: [&str; 7] = [
    "x-request-id",
    "x-b3-traceid",
    "x-b3-spanid",
    "x-b3-parentspanid",
    "x-b3-sampled",
    "x-b3-flags",
    "b3",
];

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct WebServiceConfig {
    prefix: String,
    version: String,
    address: SocketAddr,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct SampleConfig {
    webservice: WebServiceConfig,
}

impl SampleConfig {
    // Note the `nested` option on both `file` providers. This makes each
    // top-level dictionary act as a profile.
    pub fn figment<P: AsRef<Path>>(path: P) -> Figment {
        Figment::new().merge(Yaml::file(path))
    }
}

#[derive(Debug, Clone)]
pub struct Sample {
    count: Arc<Mutex<i32>>,
    name: String,
    /// Provide the port on which to serve the Service
    port: u16,
    config: SampleConfig,

    channels: Arc<Mutex<Vec<mpsc::Sender<()>>>>,
    handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    kill: Arc<Mutex<Option<Sender<()>>>>,
    running: Arc<AtomicBool>,

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

    let routes = api.with(warp::log("sample"));

    let (_addr, server) = warp::serve(routes).bind_with_graceful_shutdown(
        sample.config.webservice.address,
        async move {
            kill_recv.recv().await;
        },
    );

    info!("Serving service ({}) on port {}", sample.name, sample.port);
    tokio::task::spawn(server)
}

impl Sample {
    pub fn new<S: Into<String>>(name: S, config: SampleConfig) -> Self {
        Sample {
            count: Arc::new(Mutex::new(0)),
            name: name.into(),

            kill: Arc::new(Mutex::new(None)),
            port: 8080,
            thread_jh: Arc::new(Mutex::new(None)),
            channels: Arc::new(Mutex::new(vec![])),
            handles: Arc::new(Mutex::new(vec![])),
            running: Arc::new(AtomicBool::new(false)),
            config,
        }
    }

    pub fn running(&self) -> Arc<AtomicBool> {
        self.running.clone()
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
        let (shutdown_sample, shutdown_sample_recv) = mpsc::channel(1);
        let sample_listen = service_listen(self.clone(), shutdown_sample_recv).await;

        self.channels.lock().unwrap().push(shutdown_sample);
        self.handles.lock().unwrap().push(sample_listen);
        let channels_register = self.channels.clone();

        let my_running = self.running.clone();

        let my_services = tokio::spawn(async move {
            // TODO: Check if this future should be waited via the join
            info!("Starting Tokio spawn");

            while my_running.load(Ordering::Relaxed) {
                info!("Waiting on signal handlers");

                tokio::select! {
                    ks = kill_signal.recv() => {
                        info!("Received kill {:?}", ks);
                        my_running.store(false, Ordering::Relaxed);
                    },
                };
            }
            info!("my_services complete");
        });

        my_services
            .await
            .expect("Barried completes from a signal or service shutdown (explicit kill)");

        shutdown(channels_register).await;
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
        self.running.store(true, Ordering::Relaxed);

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
        error!("GOT 0 HERE");
        old_kill
            .unwrap()
            .blocking_send(())
            .expect("Send close to async");

        error!("GOT 1 HERE");

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
        let prefix = sample.config.webservice.prefix.clone();
        let name = warp::path("name")
            .and(warp::get())
            .and(with_sample(sample.clone()))
            .and_then(warp_handlers::name_handler);

        let get_relay = warp::get()
            .and(warp::header::headers_cloned())
            .and(with_sample(sample.clone()))
            .and_then(warp_handlers::call_relay_get_handler);

        let put_relay = warp::put()
            .and(warp::header::headers_cloned())
            .and(with_sample(sample.clone()))
            .and(warp::body::json())
            .and_then(warp_handlers::call_relay_put_handler);

        let call_relay = warp::path("relay").and(get_relay.or(put_relay));

        warp::path(prefix).and(name.or(call_relay))
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

    use log::{error, info};
    use serde::{Deserialize, Serialize};
    use warp::hyper::{self, Client, HeaderMap, Method, Request};

    use crate::sample::TRACE_HEADERS;

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

    /// Reply structure for Name endpoint
    #[derive(Serialize, Deserialize)]
    struct CallRelayReply {
        names: Vec<String>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct RelayBodyRequest {
        name: String,
        urls: Vec<String>,
    }

    /// Handler for name endpoint
    /// curl http://localhost:8080/hellothere/relay
    /// BUT also need to work out to handle a PUT with
    ///
    /// curl -X GET http://localhost:8080/hellothere/relay -H 'Content-Type: application/json' -d '{"path":"/sample","hosts":["sample00","sample01"]}'
    pub async fn call_relay_put_handler(
        headers: HeaderMap,
        hams: Sample,
        mut body: RelayBodyRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        println!("Headers for get = {:?}", headers);
        println!("Got body = {:?}", body);

        match body.urls.pop() {
            Some(next_uri) => {
                println!("next url is {}", next_uri);
                println!("Body request is {:?}", body);

                let client = Client::new();

                let mut request = Request::builder()
                    .method(Method::PUT)
                    .uri(next_uri)
                    .body(serde_json::to_vec(&body).unwrap().into())
                    .expect("request builder");

                let outbound_headers = request.headers_mut();

                for name in &TRACE_HEADERS {
                    if let Some(value) = headers.get(*name) {
                        outbound_headers.append(*name, value.clone());
                        // request.header(*name, value);
                    }
                }

                println!("outbound headers = {:?}", outbound_headers);

                //Look at incoming headers and re-use.
                // Take list of inbound names and choose first on list and remove it from list. Then send message on to that item
                match client.request(request).await {
                    Ok(reply) => {
                        info!("Got a reply as {:?}", reply);

                        let (_parts, body) = reply.into_parts();
                        let body_bytes = hyper::body::to_bytes(body).await.unwrap();

                        let mut call_reply_reply: CallRelayReply =
                            serde_json::from_slice(&body_bytes).unwrap();

                        call_reply_reply.names.push(hams.name);

                        Ok(warp::reply::json(&call_reply_reply))
                    }
                    Err(e) => {
                        error!("Got an error of {}", e);
                        let call_reply_reply = CallRelayReply {
                            names: vec![hams.name],
                        };
                        Ok(warp::reply::json(&call_reply_reply))
                    }
                }
            }
            None => {
                println!("End of urls met");

                let call_reply_reply = CallRelayReply {
                    names: vec![hams.name],
                };
                Ok(warp::reply::json(&call_reply_reply))
            }
        }
    }

    pub async fn call_relay_get_handler(
        headers: HeaderMap,
        hams: Sample,
    ) -> Result<impl warp::Reply, Infallible> {
        println!("Headers for put = {:?}", headers);

        let call_reply_reply = CallRelayReply {
            names: vec![hams.name],
        };
        Ok(warp::reply::json(&call_reply_reply))
    }
}
