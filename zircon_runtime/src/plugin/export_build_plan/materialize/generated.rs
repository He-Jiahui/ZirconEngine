use std::fs;
use std::path::{Path, PathBuf};

use super::super::ExportBuildPlan;
use super::paths::resolve_materialized_relative_path;

pub(super) fn write_generated_files(
    plan: &ExportBuildPlan,
    root: &Path,
) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut written = Vec::new();
    for file in &plan.generated_files {
        let path = resolve_materialized_relative_path(root, &file.path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, &file.contents)?;
        written.push(path);
    }
    Ok(written)
}

pub(super) fn preview_generated_files(
    plan: &ExportBuildPlan,
    root: &Path,
) -> Result<Vec<PathBuf>, std::io::Error> {
    plan.generated_files
        .iter()
        .map(|file| resolve_materialized_relative_path(root, &file.path))
        .collect()
}
