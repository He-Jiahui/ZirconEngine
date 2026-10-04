use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// Canonical physical project-descriptor path supplied by a filesystem authority.
///
/// Construction is intentionally lexical only. Callers must resolve the path through their
/// platform filesystem authority before creating this cross-process identity value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct CanonicalDescriptorIdentity {
    path: PathBuf,
}

impl<'de> Deserialize<'de> for CanonicalDescriptorIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 持久化值也经过同一校验，避免 serde 绕过本地构造时的路径不变量。
        PathBuf::deserialize(deserializer)
            .and_then(|path| Self::new(path).map_err(serde::de::Error::custom))
    }
}

impl CanonicalDescriptorIdentity {
    /// 只校验绝对路径及其段形状，不查询或规范化文件系统；调用方须先解析物理路径。
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, CanonicalDescriptorIdentityError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(CanonicalDescriptorIdentityError::Empty);
        }
        if !path.is_absolute() {
            return Err(CanonicalDescriptorIdentityError::NotAbsolute { path });
        }
        if path
            .as_os_str()
            .as_encoded_bytes()
            .split(|byte| std::path::is_separator(char::from(*byte)))
            .any(|segment| matches!(segment, b"." | b".."))
        {
            return Err(CanonicalDescriptorIdentityError::ContainsDotSegment { path });
        }
        Ok(Self { path })
    }

    /// Physical descriptor path. This is not a display string and must not be substituted into UI.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CanonicalDescriptorIdentityError {
    #[error("canonical project descriptor identity cannot be empty")]
    Empty,
    #[error("canonical project descriptor identity must be absolute: {path}")]
    NotAbsolute { path: PathBuf },
    #[error("canonical project descriptor identity cannot contain dot segments: {path}")]
    ContainsDotSegment { path: PathBuf },
}
