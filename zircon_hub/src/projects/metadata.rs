use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::hub_protocol::hub_recent_project_path_key;

/// 以跨进程一致的项目路径键索引 Hub 私有展示元数据，不承载项目清单本身。
pub type ProjectMetadataMap = BTreeMap<String, ProjectMetadata>;

/// 项目列表附加状态；引擎 ID 可随源码登记迁移，空记录可安全清理。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub engine_id: Option<String>,
    #[serde(default)]
    pub last_selected_template: Option<String>,
}

impl ProjectMetadata {
    pub fn is_empty(&self) -> bool {
        !self.pinned && self.engine_id.is_none() && self.last_selected_template.is_none()
    }
}

/// 使用共享近期项目协议的词法路径键，保证 Hub 设置与共享登记能按同一项目查找。
pub fn project_metadata_key(path: impl AsRef<Path>) -> String {
    hub_recent_project_path_key(path)
}

/// 对现存文件先解析真实路径，用于目录去重和源码引擎 ID；不存在时退回词法身份。
pub fn project_filesystem_path_key(path: impl AsRef<Path>) -> String {
    let resolved = path
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| path.as_ref().to_path_buf());
    project_metadata_key(resolved)
}

/// 为项目启动和 Editor 租约提供可显示的规范根路径，Windows 下去除扩展长度前缀。
pub fn normalize_project_root(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    strip_windows_extended_length_prefix(resolved)
}

fn strip_windows_extended_length_prefix(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(stripped) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{stripped}"));
    }
    if let Some(stripped) = text.strip_prefix(r"\\?\") {
        return PathBuf::from(stripped.to_string());
    }
    path
}

pub fn project_paths_match(left: impl AsRef<Path>, right: impl AsRef<Path>) -> bool {
    project_metadata_key(left) == project_metadata_key(right)
}

pub fn metadata_for_path<'a>(
    metadata: &'a ProjectMetadataMap,
    path: impl AsRef<Path>,
) -> Option<&'a ProjectMetadata> {
    metadata.get(&project_metadata_key(path))
}

/// 更新项目附加状态时创建缺席条目；持久化前可清理未设置任何字段的记录。
pub fn metadata_for_path_mut<'a>(
    metadata: &'a mut ProjectMetadataMap,
    path: impl AsRef<Path>,
) -> &'a mut ProjectMetadata {
    let key = project_metadata_key(path);
    metadata.entry(key).or_default()
}

pub fn prune_empty_metadata(metadata: &mut ProjectMetadataMap) {
    metadata.retain(|_, value| !value.is_empty());
}

#[cfg(test)]
#[path = "tests/metadata.rs"]
mod tests;
