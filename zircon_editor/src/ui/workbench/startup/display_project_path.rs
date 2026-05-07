pub(crate) fn display_project_path(path: impl AsRef<str>) -> String {
    display_project_text(path.as_ref())
}

pub(crate) fn display_project_title(path: impl AsRef<str>) -> String {
    let display_path = display_project_path(path);
    let normalized = display_path.replace('\\', "/");
    let trimmed = normalized.trim_end_matches('/');
    let title = trimmed
        .rsplit('/')
        .find(|segment| !segment.trim().is_empty())
        .unwrap_or(trimmed);
    if title.is_empty() {
        display_path
    } else {
        title.to_string()
    }
}

pub(crate) fn display_project_text(text: &str) -> String {
    text.replace("\\\\?\\UNC\\", "\\\\").replace("\\\\?\\", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_project_path_removes_windows_verbatim_drive_prefix() {
        assert_eq!(
            display_project_path("\\\\?\\C:\\Users\\Me\\ZirconProject"),
            "C:\\Users\\Me\\ZirconProject"
        );
    }

    #[test]
    fn display_project_path_removes_windows_verbatim_unc_prefix() {
        assert_eq!(
            display_project_path("\\\\?\\UNC\\server\\share\\ZirconProject"),
            "\\\\server\\share\\ZirconProject"
        );
    }

    #[test]
    fn display_project_text_removes_embedded_verbatim_path_prefixes() {
        assert_eq!(
            display_project_text("Reopened \\\\?\\C:\\Users\\Me\\ZirconProject"),
            "Reopened C:\\Users\\Me\\ZirconProject"
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
