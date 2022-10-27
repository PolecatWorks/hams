/// A HaMS provides essential facilities to support a k8s microservice.
/// health, liveness, startup, shutdown, monitoring, logging
#[derive(Debug)]
pub struct Hams {
    /// A HaMS has a nmae which is used for distinguishing it on APIs
    pub name: String,
    // pub rt: tokio::runtime::Runtime,
    // channels: Arc<Mutex<Vec<mpsc::Sender<()>>>>,
    // handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    // so_services: Arc<Mutex<HashMap<String, Box<SoService>>>>,
    // liveness: HealthCheck,
    // readyness: HealthCheck,
    // kill: Option<Mutex<Sender<()>>>,
    /// Provide the version of the release of HaMS
    version: String,
    /// Provide the port on which to serve the HaMS readyness and liveness
    port: u16,
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

            version: "v1".to_string(),
            port: 8080,
        }
    }
}
