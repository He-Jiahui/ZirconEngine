use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime::scene::world::SceneProjectError;

/// 操作前解析已存在的真实项目根；仅manifest文件取父目录，显示路径不能代替此验证。
pub(crate) fn project_root_path(
    path: impl AsRef<std::path::Path>,
) -> Result<PathBuf, SceneProjectError> {
    let candidate = path.as_ref();
    let root = if ProjectPaths::is_project_manifest_file(candidate) {
        candidate.parent().unwrap_or(candidate)
    } else {
        candidate
    };
    Ok(ProjectPaths::resolve_existing_path(root)?)
}

#[cfg(test)]
#[path = "tests/project_root_path.rs"]
mod tests;
