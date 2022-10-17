/// A UService allowsing multiple pServices to be assembled within it.
/// The UService provides the basic scaffolding for the web service and a loader capabilty to load and service picoservices.
/// The basic UService will reply with status information on the picoservcies provided
#[derive(Debug)]
pub struct Hams {
    pub name: String,
    // pub rt: tokio::runtime::Runtime,
    // channels: Arc<Mutex<Vec<mpsc::Sender<()>>>>,
    // handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    // so_services: Arc<Mutex<HashMap<String, Box<SoService>>>>,
    // liveness: HealthCheck,
    // readyness: HealthCheck,
    // kill: Option<Mutex<Sender<()>>>,
    version: String,
    port: u16,
}

impl<'a> Hams {
    pub fn new(name: &str) -> Hams {
        Hams {
            name: name.to_string(),

            version: "v1".to_string(),
            port: 8080,
        }
    }
}
