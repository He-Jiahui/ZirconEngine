use std::path::PathBuf;

use super::ProjectAuthorityError;

#[cfg(windows)]
#[test]
fn project_authority_error_displays_windows_operation_paths_without_verbatim_prefixes() {
    let error = ProjectAuthorityError::ManifestMissing {
        path: PathBuf::from(r"\\?\C:\ZirconBuilds\stage\project\zircon-project.toml"),
    };

    assert_eq!(
        error.to_string(),
        r"project manifest is missing: C:\ZirconBuilds\stage\project\zircon-project.toml"
    );
}
