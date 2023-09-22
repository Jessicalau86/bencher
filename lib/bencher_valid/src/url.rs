use derive_more::Display;
use once_cell::sync::Lazy;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use std::{fmt, str::FromStr};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::ValidError;

const NOT_FOUND_STR: &str = "/404";
#[allow(clippy::unwrap_used)]
static NOT_FOUND: Lazy<url::Url> =
    Lazy::new(|| url::Url::from_file_path(std::path::PathBuf::from(NOT_FOUND_STR)).unwrap());

#[typeshare::typeshare]
#[derive(Debug, Display, Clone, Eq, PartialEq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Url(String);

impl FromStr for Url {
    type Err = ValidError;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        if is_valid_url(url) {
            Ok(Self(url.into()))
        } else {
            Err(ValidError::Url(url.into()))
        }
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Url> for String {
    fn from(url: Url) -> Self {
        url.0
    }
}

impl From<url::Url> for Url {
    fn from(url: url::Url) -> Self {
        Self(url.into())
    }
}

impl From<Url> for url::Url {
    fn from(url: Url) -> Self {
        match url::Url::from_str(url.as_ref()) {
            Ok(url) => url,
            Err(e) => {
                debug_assert!(
                    false,
                    "URL ({url}) was already validated but failed to parse: {e}"
                );
                NOT_FOUND.clone()
            },
        }
    }
}

impl<'de> Deserialize<'de> for Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(UrlVisitor)
    }
}

struct UrlVisitor;

impl<'de> Visitor<'de> for UrlVisitor {
    type Value = Url;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid URL")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value.parse().map_err(E::custom)
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn is_valid_url(url: &str) -> bool {
    ::url::Url::from_str(url).is_ok()
}

#[cfg(test)]
mod test {
    use super::{is_valid_url, NOT_FOUND, NOT_FOUND_STR};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_email() {
        assert_eq!(true, is_valid_url("http://example.com"));
        assert_eq!(true, is_valid_url("https://example.com"));

        assert_eq!(false, is_valid_url(""));
        assert_eq!(false, is_valid_url("bad"));
        assert_eq!(false, is_valid_url("example.com"));
    }

    #[test]
    fn test_not_found() {
        assert_eq!(format!("file://{NOT_FOUND_STR}"), NOT_FOUND.to_string());
    }
}
