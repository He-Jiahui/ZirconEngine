use std::fs;
use std::path::Path;

use zircon_runtime_interface::project::ProjectManifestSummary;

/// 导入、启动和交付前共用的项目根检查结果，供调用方选择具体恢复提示。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectValidation {
    Valid,
    MissingRoot,
    MissingManifest,
    InvalidManifest,
}

/// 以共享项目清单解析器判断根目录可否作为现有项目使用；不负责资产或运行时完整性检查。
pub fn validate_project_root(path: impl AsRef<Path>) -> ProjectValidation {
    let path = path.as_ref();
    if !path.is_dir() {
        return ProjectValidation::MissingRoot;
    }
    if !path.join("zircon-project.toml").is_file() {
        return ProjectValidation::MissingManifest;
    }
    if !manifest_is_parsable(&path.join("zircon-project.toml")) {
        return ProjectValidation::InvalidManifest;
    }
    ProjectValidation::Valid
}

fn manifest_is_parsable(manifest_path: &Path) -> bool {
    fs::read(manifest_path)
        .ok()
        .is_some_and(|bytes| ProjectManifestSummary::parse_toml_bytes(&bytes).is_ok())
}

#[cfg(test)]
#[path = "tests/validation.rs"]
mod tests;
