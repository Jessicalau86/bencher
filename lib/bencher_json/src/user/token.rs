use chrono::{DateTime, Utc};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonNewToken {
    pub name: String,
    pub ttl: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct JsonToken {
    pub uuid: Uuid,
    pub user: Uuid,
    pub name: String,
    pub token: String,
    pub creation: DateTime<Utc>,
    pub expiration: DateTime<Utc>,
}