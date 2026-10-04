use std::fs;

use crate::asset::AssetUri;

use super::*;

#[test]
fn manifest_save_accepts_exact_limit_and_rejects_limit_plus_one_before_replace() {
    let root = std::env::temp_dir().join(format!(
        "zircon-manifest-save-limit-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let exact_path = root.join("exact.toml");
    let protected_path = root.join("protected.toml");
    let old_bytes = b"existing manifest bytes";
    fs::write(&protected_path, old_bytes).unwrap();

    let mut manifest = ProjectManifest::new(
        "Manifest Save Limit",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.asset_manifest = Some(String::new());
    let fixed_len = serialize_current_project_manifest(&manifest).unwrap().len();
    manifest.asset_manifest = Some("a".repeat(MAX_PROJECT_MANIFEST_BYTES - fixed_len));
    assert_eq!(
        serialize_current_project_manifest(&manifest).unwrap().len(),
        MAX_PROJECT_MANIFEST_BYTES
    );
    manifest.save(&exact_path).unwrap();
    assert_eq!(
        fs::metadata(&exact_path).unwrap().len(),
        MAX_PROJECT_MANIFEST_BYTES as u64
    );
    let loaded = ProjectManifest::load(&exact_path).unwrap();
    assert_eq!(loaded.asset_manifest, manifest.asset_manifest);

    manifest
        .asset_manifest
        .as_mut()
        .expect("asset manifest payload")
        .push('a');
    let error = manifest.save(&protected_path).unwrap_err();
    assert!(matches!(
        error,
        ProjectManifestError::Summary(ProjectManifestSummaryError::DocumentTooLarge {
            max: MAX_PROJECT_MANIFEST_BYTES,
            found,
        }) if found == MAX_PROJECT_MANIFEST_BYTES + 1
    ));
    assert_eq!(fs::read(&protected_path).unwrap(), old_bytes);

    let _ = fs::remove_dir_all(root);
}
