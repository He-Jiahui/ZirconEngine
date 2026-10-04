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
