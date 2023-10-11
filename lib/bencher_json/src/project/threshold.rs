use bencher_valid::{Boundary, SampleSize};
use chrono::{DateTime, Utc};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{JsonBranch, JsonMetricKind, JsonTestbed, ProjectUuid, ResourceId};

crate::typed_uuid::typed_uuid!(ThresholdUuid);
crate::typed_uuid::typed_uuid!(StatisticUuid);

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonNewThreshold {
    pub metric_kind: ResourceId,
    pub branch: ResourceId,
    pub testbed: ResourceId,
    #[serde(flatten)]
    pub statistic: JsonNewStatistic,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonNewStatistic {
    pub test: JsonStatisticKind,
    pub min_sample_size: Option<SampleSize>,
    pub max_sample_size: Option<SampleSize>,
    pub window: Option<u32>,
    pub lower_boundary: Option<Boundary>,
    pub upper_boundary: Option<Boundary>,
}

impl JsonNewStatistic {
    pub fn lower_boundary() -> Self {
        Self {
            test: JsonStatisticKind::T,
            min_sample_size: None,
            max_sample_size: Some(SampleSize::THIRTY),
            window: None,
            lower_boundary: Some(Boundary::EIGHTY),
            upper_boundary: None,
        }
    }

    pub fn upper_boundary() -> Self {
        Self {
            test: JsonStatisticKind::T,
            min_sample_size: None,
            max_sample_size: Some(SampleSize::THIRTY),
            window: None,
            lower_boundary: None,
            upper_boundary: Some(Boundary::EIGHTY),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonThresholds(pub Vec<JsonThreshold>);

crate::from_vec!(JsonThresholds[JsonThreshold]);

#[typeshare::typeshare]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonThreshold {
    pub uuid: ThresholdUuid,
    pub project: ProjectUuid,
    pub metric_kind: JsonMetricKind,
    pub branch: JsonBranch,
    pub testbed: JsonTestbed,
    pub statistic: JsonStatistic,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonStatistic {
    pub uuid: StatisticUuid,
    pub threshold: ThresholdUuid,
    pub test: JsonStatisticKind,
    pub min_sample_size: Option<SampleSize>,
    pub max_sample_size: Option<SampleSize>,
    pub window: Option<u32>,
    pub lower_boundary: Option<Boundary>,
    pub upper_boundary: Option<Boundary>,
    pub created: DateTime<Utc>,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum JsonStatisticKind {
    Z,
    T,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonThresholdStatistic {
    pub uuid: ThresholdUuid,
    pub project: ProjectUuid,
    pub statistic: JsonStatistic,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonUpdateThreshold {
    #[serde(flatten)]
    pub statistic: JsonNewStatistic,
}
