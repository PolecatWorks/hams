use reqwest::StatusCode;
use tokio::time::Instant;
use url::Url;

use crate::error::HamsError;

use super::{BoxedHealthProbe, HealthProbe};

#[derive(Debug, Clone)]
pub struct Get {
    name: String,
    url: Url,
    status: StatusCode,
}

impl Get {
    pub fn new<S: Into<String>>(name: S, url: &str) -> Self {
        Self {
            name: name.into(),
            url: Url::parse(url).unwrap(),
            status: StatusCode::OK,
        }
    }
}

impl HealthProbe for Get {
    fn name(&self) -> Result<String, HamsError> {
        Ok(self.name.clone())
    }

    fn check(&self, _time: Instant) -> Result<bool, HamsError> {
        let client = reqwest::Client::new();

        let response = client.get(self.url).send().await?;
        Ok(response.status() == self.status)
    }

    fn ffi_boxed(&self) -> BoxedHealthProbe<'static> {
        BoxedHealthProbe::new(self.clone())
    }
}
