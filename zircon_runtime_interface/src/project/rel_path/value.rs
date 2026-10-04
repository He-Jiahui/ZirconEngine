use std::fmt;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::RelPathError;

/// Portable normalized relative path that cannot escape its owning project root.
/// 上述英文描述只保证词法上不含越界片段；实际文件系统目标仍须由调用方检查。
/// 用于清单、模板和资产引用的归一化项目相对路径；解析仅做词法检查，不保证实际目标位于项目根内。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct RelPath(pub(super) String);

impl RelPath {
    pub fn project_assets() -> Self {
        Self("assets".to_string())
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, RelPathError> {
        super::parse::parse(value.as_ref())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.0.split('/').collect()
    }

    /// 拼接到所属项目根；调用方用于磁盘读写前须另行保证目标仍位于该根目录内。
    pub fn join_to(&self, root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(self.to_path_buf())
    }
}

impl fmt::Display for RelPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
