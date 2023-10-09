use chrono::{DateTime, Utc};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BigInt, JsonThreshold};

use super::{benchmark::JsonBenchmarkMetric, boundary::JsonLimit, report::ReportUuid};

crate::typed_uuid::typed_uuid!(AlertUuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonAlerts(pub Vec<JsonAlert>);

crate::from_vec!(JsonAlerts[JsonAlert]);

#[typeshare::typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonAlert {
    pub uuid: AlertUuid,
    pub report: ReportUuid,
    pub iteration: u32,
    pub threshold: JsonThreshold,
    pub benchmark: JsonBenchmarkMetric,
    pub limit: JsonLimit,
    pub status: JsonAlertStatus,
    pub modified: DateTime<Utc>,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum JsonAlertStatus {
    Active,
    Dismissed,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonAlertStats {
    pub active: BigInt,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonUpdateAlert {
    pub status: Option<JsonAlertStatus>,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonPerfAlert {
    pub uuid: AlertUuid,
    pub limit: JsonLimit,
    pub status: JsonAlertStatus,
    pub modified: DateTime<Utc>,
}
