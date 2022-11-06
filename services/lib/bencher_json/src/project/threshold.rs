use ordered_float::OrderedFloat;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::perf::JsonPerfKind;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonNewThreshold {
    pub branch: Uuid,
    pub testbed: Uuid,
    pub kind: JsonPerfKind,
    pub statistic: JsonNewStatistic,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonNewStatistic {
    pub test: JsonStatisticKind,
    pub max_sample_size: Option<u32>,
    pub window: Option<u32>,
    pub left_side: Option<OrderedFloat<f32>>,
    pub right_side: Option<OrderedFloat<f32>>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonThreshold {
    pub uuid: Uuid,
    pub branch: Uuid,
    pub testbed: Uuid,
    pub kind: JsonPerfKind,
    pub statistic: Uuid,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonStatistic {
    pub uuid: Uuid,
    pub test: JsonStatisticKind,
    pub max_sample_size: Option<u32>,
    pub window: Option<u32>,
    pub left_side: Option<OrderedFloat<f32>>,
    pub right_side: Option<OrderedFloat<f32>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum JsonStatisticKind {
    Z,
    T,
}
