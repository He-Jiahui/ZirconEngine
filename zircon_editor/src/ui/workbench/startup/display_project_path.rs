use std::path::Path;

use zircon_runtime::asset::project::ProjectPaths;

/// 只生成用户可读路径，移除系统专用前缀不应改变实际文件访问依据。
pub(crate) fn display_project_path(path: impl AsRef<str>) -> String {
    let display_path = ProjectPaths::display_path(Path::new(path.as_ref()));
    display_path.to_string_lossy().into_owned()
}

/// 从显示路径提取窗口标题；标题不是项目唯一身份。
pub(crate) fn display_project_title(path: impl AsRef<str>) -> String {
    project_title_from_display_path(display_project_path(path))
}

fn project_title_from_display_path(display_path: String) -> String {
    let trimmed = display_path.trim_end_matches(['/', '\\']);
    let title = trimmed
        .rsplit(['/', '\\'])
        .find(|segment| !segment.trim().is_empty())
        .unwrap_or(trimmed);
    if title.is_empty() {
        display_path
    } else {
        title.to_string()
    }
}

#[cfg(test)]
#[path = "tests/display_project_path.rs"]
mod tests;

#[cfg(test)]
#[path = "display_project_path/tests/direct_title_tests.rs"]
mod direct_title_tests;
