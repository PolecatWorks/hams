use serde::Deserialize;
use std::net::SocketAddr;

/// Configuration for the HAMS service
#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct HamsConfig {
    /// Hostname to start the webservice on
    /// This allows changing to localhost for dev and 0.0.0.0 or specific address for deployment
    pub address: SocketAddr,
    /// Name for the service
    pub name: String,
    /// Name for the service
    pub version: String,
    /// enables logging for HaMS API
    pub logging: bool,
}

impl Default for HamsConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:8079".parse().unwrap(),
            name: "NO_NAME".to_string(),
            version: "0.0.0".to_string(),
            logging: false,
        }
    }
}

/// Configuration for startup and shutdown tasks
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct TaskConfig {
    /// Number of retries for the task
    pub retries: u32,
    /// Sleep duration in milliseconds between retries
    pub sleep_ms: u64,
    /// Timeout in milliseconds for the task
    pub timeout_ms: u64,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            retries: 3,
            sleep_ms: 1000,
            timeout_ms: 5000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let config = HamsConfig::default();
        assert_eq!(
            config.address,
            SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
                8079
            )
        );
    }
}
