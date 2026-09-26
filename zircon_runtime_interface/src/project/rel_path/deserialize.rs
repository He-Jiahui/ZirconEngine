use serde::{Deserialize, Deserializer};

use super::RelPath;

// 持久化路径必须经过同一词法解析入口，不能借由 Serde 绕过相对路径约束。
impl<'de> Deserialize<'de> for RelPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(serde::de::Error::custom)
    }
}
