use std::path::Path;

use super::AutosaveSourcePath;

#[cfg(windows)]
#[test]
fn source_path_normalizes_windows_separators_to_the_project_form() {
    let source = AutosaveSourcePath::parse(r"assets\ui\panel.zui").unwrap();

    assert_eq!(source.as_path(), Path::new("assets/ui/panel.zui"));
}
