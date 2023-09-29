use bencher_valid::GitHash;
use chrono::{DateTime, Utc};
use derive_more::Display;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{JsonAlert, JsonMetricKind, JsonProject, JsonTestbed, JsonUser, ResourceId};

use super::{
    benchmark::JsonBenchmarkMetric, branch::JsonBranchVersion, threshold::JsonThresholdStatistic,
};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonNewReport {
    pub branch: ResourceId,
    pub hash: Option<GitHash>,
    pub testbed: ResourceId,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub results: Vec<String>,
    pub settings: Option<JsonReportSettings>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonReportSettings {
    pub adapter: Option<JsonAdapter>,
    pub average: Option<JsonAverage>,
    pub fold: Option<JsonFold>,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, Copy, Default, Display, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum JsonAdapter {
    #[default]
    Magic,
    Json,
    CSharp,
    CSharpDotNet,
    Cpp,
    CppCatch2,
    CppGoogle,
    Go,
    GoBench,
    Java,
    JavaJmh,
    Js,
    JsBenchmark,
    JsTime,
    Python,
    PythonAsv,
    PythonPytest,
    Ruby,
    RubyBenchmark,
    Rust,
    RustBench,
    RustCriterion,
    RustIai,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum JsonAverage {
    #[default]
    Mean,
    Median,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum JsonFold {
    Min,
    Max,
    Mean,
    Median,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonReports(pub Vec<JsonReport>);

crate::from_vec!(JsonReports[JsonReport]);

#[typeshare::typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonReport {
    pub uuid: Uuid,
    pub user: JsonUser,
    pub project: JsonProject,
    pub branch: JsonBranchVersion,
    pub testbed: JsonTestbed,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub adapter: JsonAdapter,
    pub results: JsonReportResults,
    pub alerts: JsonReportAlerts,
    pub created: DateTime<Utc>,
}

#[typeshare::typeshare]
pub type JsonReportResults = Vec<JsonReportIteration>;

#[typeshare::typeshare]
pub type JsonReportIteration = Vec<JsonReportResult>;

#[typeshare::typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonReportResult {
    pub metric_kind: JsonMetricKind,
    // The threshold should be the same for all the benchmark results
    pub threshold: Option<JsonThresholdStatistic>,
    pub benchmarks: Vec<JsonBenchmarkMetric>,
}

#[typeshare::typeshare]
pub type JsonReportAlerts = Vec<JsonAlert>;
