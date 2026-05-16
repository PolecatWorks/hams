use super::AsyncHealthProbe;
use crate::error::HamsError;
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use std::time::Duration;
use std::time::SystemTime;
use url::Url;

#[derive(Debug, Clone)]
pub struct AsyncHttpProbe {
    name: String,
    url: Url,
    expected_codes: Vec<StatusCode>,
    timeout: Duration,
}

impl AsyncHttpProbe {
    pub fn new<S: Into<String>>(
        name: S,
        url: &str,
        expected_codes: Option<Vec<u16>>,
        timeout: Option<Duration>,
    ) -> Result<Self, HamsError> {
        let name = name.into();
        let url = Url::parse(url).map_err(|e| HamsError::Message(format!("Invalid URL: {}", e)))?;

        let codes = match expected_codes {
            Some(codes) => codes
                .iter()
                .filter_map(|&c| StatusCode::from_u16(c).ok())
                .collect(),
            None => vec![StatusCode::OK],
        };

        Ok(Self {
            name,
            url,
            expected_codes: codes,
            timeout: timeout.unwrap_or(Duration::from_secs(5)),
        })
    }
}

#[async_trait]
impl AsyncHealthProbe for AsyncHttpProbe {
    fn name(&self) -> Result<String, HamsError> {
        Ok(self.name.clone())
    }

    async fn check(&self, _time: SystemTime) -> Result<bool, HamsError> {
        let client = Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| HamsError::Message(format!("Failed to build client: {}", e)))?;

        match client.get(self.url.clone()).send().await {
            Ok(response) => Ok(self.expected_codes.contains(&response.status())),
            Err(_) => Ok(false),
        }
    }
}
