use std::path::{Path, PathBuf};

const COVER_BASENAMES: &[&str] = &["cover", "thumbnail", "project"];
const COVER_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "svg"];
const COVER_DIRECTORIES: &[&[&str]] = &[&[".zircon"], &[], &["Assets"], &["assets"]];

// TODO: [CR-HUBCORE-0004] 确认项目封面路径是否应接入生产视图；目前只有本文件测试调用，view_model.rs 使用 project_cover_id 生成占位标识；下一步核对 Web 项目卡片的封面资源协议。
/// 按约定优先级查找项目封面文件，优先使用项目元数据目录中的封面；不加载图片内容。
/// 缺少目录或候选文件时返回空值，调用端可提供占位图。
pub fn project_cover_path(project_root: impl AsRef<Path>) -> Option<PathBuf> {
    let project_root = project_root.as_ref();
    if project_root.as_os_str().is_empty() || !project_root.is_dir() {
        return None;
    }

    cover_candidates(project_root)
        .into_iter()
        .find(|candidate| candidate.is_file())
}

fn cover_candidates(project_root: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for directory in COVER_DIRECTORIES {
        let base_dir = directory
            .iter()
            .fold(project_root.to_path_buf(), |path, segment| {
                path.join(segment)
            });
        for basename in COVER_BASENAMES {
            for extension in COVER_EXTENSIONS {
                candidates.push(base_dir.join(format!("{basename}.{extension}")));
            }
        }
    }
    candidates
}

#[cfg(test)]
#[path = "tests/cover.rs"]
mod tests;
