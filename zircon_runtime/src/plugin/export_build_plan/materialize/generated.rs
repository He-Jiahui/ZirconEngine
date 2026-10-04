//! 非原生生成文件的落盘与预览共用路径校验；内容相同则保持已有文件时间戳，供重复导出复用。
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::super::ExportBuildPlan;
use super::paths::resolve_materialized_relative_path;

/// ExportBuildPlan 已完成 fatal、链接源及原生根准入后才能调用；此处只负责计划内文本工件。
pub(super) fn write_generated_files(
    plan: &ExportBuildPlan,
    root: &Path,
) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut written = Vec::with_capacity(plan.generated_files.len());
    let mut created_parents = HashSet::with_capacity(plan.generated_files.len());
    for file in &plan.generated_files {
        let path = resolve_materialized_relative_path(root, &file.path)?;
        if let Some(parent) = path.parent() {
            if created_parents.insert(parent.to_path_buf()) {
                fs::create_dir_all(parent)?;
            }
        }
        write_if_changed(&path, &file.contents)?;
        written.push(path);
    }
    Ok(written)
}

// Generated contents are already resident in the plan, so equality is verified from bytes rather
// than trusting filesystem timestamps that can alias across export generations.
fn write_if_changed(path: &Path, contents: &str) -> Result<bool, std::io::Error> {
    match fs::read(path) {
        Ok(current) if current == contents.as_bytes() => return Ok(false),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    fs::write(path, contents)?;
    Ok(true)
}

#[cfg(test)]
#[path = "tests/generated.rs"]
mod tests;

/// 预览必须与真实写入共享路径拒绝规则，避免报告一个无法安全落盘的工件。
pub(super) fn preview_generated_files(
    plan: &ExportBuildPlan,
    root: &Path,
) -> Result<Vec<PathBuf>, std::io::Error> {
    plan.generated_files
        .iter()
        .map(|file| resolve_materialized_relative_path(root, &file.path))
        .collect()
}
