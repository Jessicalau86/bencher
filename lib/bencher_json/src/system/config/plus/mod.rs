#![cfg(feature = "plus")]

use bencher_valid::Sanitize;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod cloud;
pub mod stats;

pub use cloud::JsonCloud;
pub use stats::JsonStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonPlus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<JsonStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud: Option<JsonCloud>,
}

impl Sanitize for JsonPlus {
    fn sanitize(&mut self) {
        self.cloud.sanitize();
    }
}
