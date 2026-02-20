use std::path::Path;

use hams::hams::config::HamsConfig;
use serde::Deserialize;

use figment::{
    Figment,
    providers::{Format, Yaml},
};

#[derive(Debug, Deserialize, Clone)]
pub struct WebServiceConfig {
    prefix: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    webservice: WebServiceConfig,
    #[serde(default)]
    pub hams: HamsConfig,
}

impl Config {
    // Note the `nested` option on both `file` providers. This makes each
    // top-level dictionary act as a profile.
    pub fn figment<P: AsRef<Path>>(path: P) -> Figment {
        Figment::new().merge(Yaml::file(path))
    }
}
