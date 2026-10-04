//! 为访问同一资源的异步工作提供有界串行化身份；规格携带此身份供调度端建立前后关系，名称相同意味着共享互斥域而非重复请求合并。
use serde::{Deserialize, Deserializer, Serialize};

use super::MutexGroupError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct MutexGroup(String);

impl MutexGroup {
    /// Maximum retained UTF-8 bytes for one resource serialization identity.
    pub const MAX_BYTES: usize = 128;

    pub fn parse(value: impl Into<String>) -> Result<Self, MutexGroupError> {
        let value = value.into();
        if value.is_empty() {
            return Err(MutexGroupError::Empty);
        }
        if value.len() > Self::MAX_BYTES {
            return Err(MutexGroupError::TooLong {
                len: value.len(),
                max: Self::MAX_BYTES,
            });
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

#[cfg(test)]
#[path = "tests/mutex_group.rs"]
mod tests;
