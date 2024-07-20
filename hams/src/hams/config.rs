use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct HamsConfig {
    /// Hostname to start the webservice on
    /// This allows chainging to localhost for dev and 0.0.0.0 or specific address for deployment
    pub address: SocketAddr,
    /// Name for the service
    pub name: String,
}

impl Default for HamsConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:8079".parse().unwrap(),
            name: "NO_NAME".to_string(),
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
