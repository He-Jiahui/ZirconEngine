use std::path::Path;

use zircon_runtime::asset::project::ProjectPaths;

pub(crate) fn display_project_path(path: impl AsRef<str>) -> String {
    let display_path = ProjectPaths::display_path(Path::new(path.as_ref()));
    display_path.to_string_lossy().into_owned()
}

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
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn display_project_path_removes_windows_verbatim_drive_prefix() {
        assert_eq!(
            display_project_path("\\\\?\\C:\\Users\\Me\\ZirconProject"),
            "C:\\Users\\Me\\ZirconProject"
        );
    }

    #[cfg(windows)]
    #[test]
    fn display_project_path_removes_windows_verbatim_unc_prefix() {
        assert_eq!(
            display_project_path("\\\\?\\UNC\\server\\share\\ZirconProject"),
            "\\\\server\\share\\ZirconProject"
        );
    }

    #[test]
    fn display_project_title_uses_last_path_segment() {
        assert_eq!(
            display_project_title("\\\\?\\C:\\Users\\Me\\ZirconProject"),
            "ZirconProject"
        );
    }
}

#[cfg(test)]
#[path = "display_project_path/direct_title_tests.rs"]
mod direct_title_tests;
