use std::path::Path;

use serde::Deserialize;

use figment::{
    providers::{Format, Yaml},
    Figment,
};

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct WebServiceConfig {
    prefix: String,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Config {
    webservice: WebServiceConfig,
}

impl Config {
    // Note the `nested` option on both `file` providers. This makes each
    // top-level dictionary act as a profile.
    pub fn figment<P: AsRef<Path>>(path: P) -> Figment {
        Figment::new().merge(Yaml::file(path))
    }
}
