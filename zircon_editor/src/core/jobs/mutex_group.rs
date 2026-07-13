use serde::{Deserialize, Deserializer, Serialize};

use super::MutexGroupError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct MutexGroup(String);

impl MutexGroup {
    pub fn parse(value: impl Into<String>) -> Result<Self, MutexGroupError> {
        let value = value.into();
        if value.is_empty() {
            return Err(MutexGroupError::Empty);
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(MutexGroupError::Invalid { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for MutexGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}
