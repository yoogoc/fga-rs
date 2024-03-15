use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Config {
    pub http: Option<HttpConfig>,
    pub grpc: Option<GrpcConfig>,
    pub datasource: Datasource,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct HttpConfig {
    pub addr: String,
    #[serde(with = "humantime_serde")]
    pub timeout: Option<Duration>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct GrpcConfig {
    addr: String,
    #[serde(with = "humantime_serde")]
    pub timeout: Option<Duration>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Datasource {
    pub uri: String,
}
