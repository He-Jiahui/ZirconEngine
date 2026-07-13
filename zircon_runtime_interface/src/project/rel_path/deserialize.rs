use serde::{Deserialize, Deserializer};

use super::RelPath;

impl<'de> Deserialize<'de> for RelPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(serde::de::Error::custom)
    }
}
