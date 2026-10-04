use std::path::PathBuf;

use super::RuntimeProjectError;

#[cfg(windows)]
#[test]
fn runtime_project_error_displays_windows_operation_roots_without_verbatim_prefixes() {
    let error = RuntimeProjectError::PreparedProjectManagerTransferred {
        root: PathBuf::from(r"\\?\C:\ZirconBuilds\stage\project"),
    };

    assert_eq!(
        error.to_string(),
        "runtime project C:\\ZirconBuilds\\stage\\project already transferred its prepared ProjectManager to AssetModule"
    );
}
