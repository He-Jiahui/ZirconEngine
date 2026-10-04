use super::*;

#[test]
fn explicit_additions_are_part_of_the_build_set_identity() {
    let mut value = serde_json::json!({
        "schema_version": 2,
        "build_set_kind": BUILD_SET_KIND,
        "status": BUILD_SET_STATUS,
        "build_set_id": "D".repeat(64),
        "created_utc": "2026-09-26T00:00:00Z",
        "snapshot_relative_path": BUILD_SET_SNAPSHOT_RELATIVE_PATH,
        "source_policy": BUILD_SET_ADDITIONS_SOURCE_POLICY,
        "git_revision": "a".repeat(40),
        "dirty_overlay_sha256": "C".repeat(64),
        "files": [
            {"relative_path": "Cargo.toml", "sha256": "A".repeat(64), "byte_length": 10},
            {"relative_path": "src/extra.rs", "sha256": "B".repeat(64), "byte_length": 25}
        ],
        "source_additions": [
            {"relative_path": "src/extra.rs", "sha256": "B".repeat(64), "byte_length": 25}
        ]
    });
    let manifest: BuildSetManifest = serde_json::from_value(value.clone()).unwrap();
    validate_manifest_authority(&manifest).unwrap();
    validate_source_additions(&manifest).unwrap();
    let v2_id = derive_build_set_id(&manifest);

    value["schema_version"] = 1.into();
    value["source_policy"] = BUILD_SET_SOURCE_POLICY.into();
    value.as_object_mut().unwrap().remove("source_additions");
    let legacy: BuildSetManifest = serde_json::from_value(value.clone()).unwrap();
    validate_manifest_authority(&legacy).unwrap();
    assert_ne!(v2_id, derive_build_set_id(&legacy));

    value["schema_version"] = 2.into();
    value["source_policy"] = BUILD_SET_ADDITIONS_SOURCE_POLICY.into();
    value["source_additions"] = serde_json::json!([
        {"relative_path": "src/extra.rs", "sha256": "E".repeat(64), "byte_length": 25}
    ]);
    let tampered: BuildSetManifest = serde_json::from_value(value).unwrap();
    assert!(validate_source_additions(&tampered).is_err());
}
