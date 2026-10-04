use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 把配置、缺失资源与后端失败分开，供脚本和编辑器决定重试、提示或停止路径请求。
pub enum NavigationErrorKind {
    InvalidConfiguration,
    MissingNavMesh,
    NoPath,
    BakeFailed,
    BackendFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationError {
    pub kind: NavigationErrorKind,
    pub message: String,
}

impl NavigationError {
    pub fn new(kind: NavigationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn missing_nav_mesh(message: impl Into<String>) -> Self {
        Self::new(NavigationErrorKind::MissingNavMesh, message)
    }
}

impl fmt::Display for NavigationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.kind, self.message)
    }
}

impl Error for NavigationError {}
