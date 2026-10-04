use std::path::Path;

use super::runtime_artifact_manifest_path;

#[test]
fn runtime_artifact_manifest_is_a_library_sidecar() {
    assert_eq!(
        runtime_artifact_manifest_path(Path::new("E:/build/zircon_runtime.dll")).unwrap(),
        Path::new("E:/build/zircon_runtime.dll.manifest.json"),
    );
}
