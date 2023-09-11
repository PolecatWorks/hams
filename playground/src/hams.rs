use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

// Hams class allows the creation of a liveness health check.
// We can add/remove alive and ready probes to it.
#[derive(Debug, Clone)]
struct Hams<HealthCheck> {
    /// A HaMS has a nmae which is used for distinguishing it on APIs
    pub name: String,

    /// Provide the version of the release of HaMS
    version: String,
    /// Provide the name of the package
    package: String,

    /// Provide the port on which to serve the HaMS readyness and liveness
    port: u16,

    alive: Arc<Mutex<HashSet<HealthCheck>>>,
    ready: Arc<Mutex<HashSet<HealthCheck>>>,
}

impl<HealthCheck> Hams<HealthCheck> {
    /// Returns a HaMS instance with the name given
    ///
    /// # Arguments
    ///
    /// * 'name' - A string slice that holds the name of the HaMS
    pub fn new<S: Into<String>>(name: S) -> Hams<HealthCheck> {
        Hams {
            name: name.into(),

            // channels: Arc::new(Mutex::new(vec![])),
            // handles: Arc::new(Mutex::new(vec![])),
            version: env!("CARGO_PKG_VERSION").to_string(),
            package: env!("CARGO_PKG_NAME").to_string(),
            port: 8079,
            alive: Arc::new(Mutex::new(HashSet::new())),
            ready: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_destruct() {
        let my_hams = Hams::<u32>::new("me");
    }

    #[test]
    fn add_remove_alive() {
        let my_hams = Hams::<u32>::new("me");

        let my_hk = 32;

        // my_hams.insert(&my_hk);

        // my_hams.remove(&my_hk);
    }
}
