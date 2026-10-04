use std::fs;

use super::*;

#[test]
fn source_engine_validation_requires_manifest_and_build_tool() {
    let root = std::env::temp_dir().join(format!(
        "zircon_hub_engine_validation_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("tools")).unwrap();

    assert_eq!(
        validate_source_engine(&root),
        SourceEngineValidation::MissingWorkspaceManifest
    );
    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    assert_eq!(
        validate_source_engine(&root),
        SourceEngineValidation::MissingRuntimeWorkspaceMember
    );
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"zircon_runtime\"]\n",
    )
    .unwrap();
    assert_eq!(
        validate_source_engine(&root),
        SourceEngineValidation::MissingBuildTool
    );
    fs::write(root.join("tools").join("zircon_build.py"), "").unwrap();
    assert_eq!(validate_source_engine(&root), SourceEngineValidation::Valid);
    assert_eq!(
        SourceEngineValidation::MissingBuildTool.summary(),
        "Source checkout is missing tools/zircon_build.py"
    );
    assert_eq!(
        SourceEngineValidation::MissingWorkspaceManifest.recovery_hint(),
        "Select the ZirconEngine repository root that contains the workspace Cargo.toml"
    );
    assert_eq!(
        SourceEngineValidation::MissingRuntimeWorkspaceMember.summary(),
        "Source checkout workspace is missing zircon_runtime member"
    );

    let _ = fs::remove_dir_all(root);
}
