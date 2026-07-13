use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serialize};

/// Stable identity for one versioned payload family.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct SchemaId(Cow<'static, str>);

impl SchemaId {
    pub const fn new(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'de> Deserialize<'de> for SchemaId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| Self(Cow::Owned(value)))
    }
}
