//! 规范化项目相对源路径，作为自动保存身份与元数据键；拒绝绝对路径和穿越，跨平台分隔符归一。

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::recovery::AutosaveError;

/// A source-document location persisted with an autosave snapshot.
///
/// Recovery records only project-relative paths, so replay cannot be redirected
/// outside the project by an autosave directory entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutosaveSourcePath(PathBuf);

impl AutosaveSourcePath {
    pub fn parse(value: impl Into<PathBuf>) -> Result<Self, AutosaveError> {
        let path = value.into();
        let is_project_relative = !path.as_os_str().is_empty()
            && !path.is_absolute()
            && path.to_str().is_some()
            && path
                .components()
                .all(|component| matches!(component, Component::Normal(_)));
        if !is_project_relative {
            return Err(AutosaveError::InvalidRecoverySourcePath { path });
        }
        let normalized = normalize_project_relative_path(&path);
        Ok(Self(PathBuf::from(normalized)))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

fn normalize_project_relative_path(path: &Path) -> String {
    let mut normalized = String::with_capacity(path.as_os_str().len());
    for (index, component) in path.components().enumerate() {
        if index != 0 {
            normalized.push('/');
        }
        normalized.push_str(
            component
                .as_os_str()
                .to_str()
                .expect("validated autosave source components are UTF-8"),
        );
    }
    normalized
}

#[cfg(test)]
#[path = "source_path/tests/direct_join_tests.rs"]
mod direct_join_tests;

#[cfg(test)]
#[path = "tests/source_path.rs"]
mod tests;
