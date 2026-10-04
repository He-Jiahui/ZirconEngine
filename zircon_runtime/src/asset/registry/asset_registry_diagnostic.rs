use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetUri, AssetUuid};

/// 可恢复的注册表事件，供编辑器诊断展示；这些事件不等同于索引构建失败。
/// Recoverable registry events surfaced to editor diagnostics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetRegistryDiagnostic {
    CorruptPersistenceRebuilt {
        path: PathBuf,
        reason: String,
    },
    DuplicateGuidReminted {
        original: AssetUuid,
        first_path: AssetUri,
        path: AssetUri,
        replacement: AssetUuid,
    },
    UnresolvedDependency {
        owner: AssetUuid,
        path: AssetUri,
    },
}
