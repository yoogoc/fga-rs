use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub http: Option<HttpConfig>,
    pub grpc: Option<GrpcConfig>,
    pub datasource: Datasource,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HttpConfig {
    pub(crate) addr: String,
    #[serde(with = "humantime_serde")]
    pub(crate) timeout: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GrpcConfig {
    addr: String,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Datasource {
    pub(crate) uri: String,
}
