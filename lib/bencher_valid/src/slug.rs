use derive_more::Display;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use std::{fmt, str::FromStr};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{benchmark_name::MAX_BENCHMARK_NAME_LEN, ValidError};

#[typeshare::typeshare]
#[derive(Debug, Display, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Slug(String);

impl FromStr for Slug {
    type Err = ValidError;

    fn from_str(slug: &str) -> Result<Self, Self::Err> {
        if is_valid_slug(slug) {
            Ok(Self(slug.into()))
        } else {
            Err(ValidError::Slug(slug.into()))
        }
    }
}

impl AsRef<str> for Slug {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Slug> for String {
    fn from(slug: Slug) -> Self {
        slug.0
    }
}

impl<'de> Deserialize<'de> for Slug {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SlugVisitor)
    }
}

struct SlugVisitor;

impl Visitor<'_> for SlugVisitor {
    type Value = Slug;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid slug")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value.parse().map_err(E::custom)
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty() && slug.len() <= MAX_BENCHMARK_NAME_LEN && slug == slug::slugify(slug)
}

#[cfg(test)]
mod test {

    use super::is_valid_slug;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_slug() {
        assert_eq!(true, is_valid_slug("a-valid-slug"));
        assert_eq!(true, is_valid_slug("2nd-valid-slug"));
        assert_eq!(true, is_valid_slug("client-submit-serialize-deserialize-handle-client-submit-serialize-deserialize-handle-1996529012"));

        assert_eq!(false, is_valid_slug(""));
        assert_eq!(false, is_valid_slug(" a-valid-slug"));
        assert_eq!(false, is_valid_slug("a- valid-slug"));
        assert_eq!(false, is_valid_slug("a-valid-slug "));
        assert_eq!(false, is_valid_slug(" a-valid-slug "));
        assert_eq!(false, is_valid_slug("-a-valid-slug"));
        assert_eq!(false, is_valid_slug("a-valid-slug-"));
        assert_eq!(false, is_valid_slug("-a-valid-slug-"));
        assert_eq!(false, is_valid_slug("a--valid-slug"));
        assert_eq!(false, is_valid_slug("a-Valid-slug"));
    }
}
