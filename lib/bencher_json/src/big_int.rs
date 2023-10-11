#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Do not typeshare this type in order to obfuscate the u64
// https://github.com/1Password/typeshare/issues/24
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct BigInt(pub u64);

impl From<u64> for BigInt {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<usize> for BigInt {
    fn from(value: usize) -> Self {
        Self(u64::try_from(value).unwrap_or_default())
    }
}
