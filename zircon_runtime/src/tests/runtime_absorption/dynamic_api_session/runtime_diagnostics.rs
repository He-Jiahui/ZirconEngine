use std::fs;
use std::path::Path;

use super::shared::{
    EXPECTED_RUNTIME_10_RUNTIME_DIAGNOSTICS_ANCHORS,
    EXPECTED_RUNTIME_10_SCENE_ASSET_RELOAD_DIAGNOSTIC_PATH_ANCHORS,
};

#[test]
fn runtime_10_profile_control_exposes_runtime_diagnostics_snapshot_without_abi_table_growth() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should live under the repository root");

    assert_eq!(
        EXPECTED_RUNTIME_10_RUNTIME_DIAGNOSTICS_ANCHORS.len(),
        15,
        "Runtime 10 runtime diagnostics profile-control anchor inventory should stay at 15 anchors"
    );
    for (relative_file, expected_anchor) in EXPECTED_RUNTIME_10_RUNTIME_DIAGNOSTICS_ANCHORS {
        let source = fs::read_to_string(repo_root.join(relative_file))
            .unwrap_or_else(|error| panic!("`{relative_file}` should be readable: {error}"));
        assert!(
            source.contains(expected_anchor),
            "`{relative_file}` should keep Runtime 10 runtime diagnostics anchor `{expected_anchor}`"
        );
    }

    let api_table =
        fs::read_to_string(repo_root.join("zircon_runtime_interface/src/runtime_api/api_table.rs"))
            .expect("runtime API table should be readable");
    assert!(
        !api_table.contains("runtime_diagnostics"),
        "runtime diagnostics snapshots must reuse profile_control JSON instead of growing ZrRuntimeApiV1"
    );
}

#[test]
fn runtime_10_scene_asset_reload_frame_diagnostics_keep_stable_store_paths() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should live under the repository root");

    assert_eq!(
        EXPECTED_RUNTIME_10_SCENE_ASSET_RELOAD_DIAGNOSTIC_PATH_ANCHORS.len(),
        21,
        "Runtime 10 scene-asset reload diagnostic path inventory should stay at 21 anchors"
    );
    for (relative_file, expected_anchor) in
        EXPECTED_RUNTIME_10_SCENE_ASSET_RELOAD_DIAGNOSTIC_PATH_ANCHORS
    {
        let source = fs::read_to_string(repo_root.join(relative_file))
            .unwrap_or_else(|error| panic!("`{relative_file}` should be readable: {error}"));
        assert!(
            source.contains(expected_anchor),
            "`{relative_file}` should keep scene-asset reload diagnostic anchor `{expected_anchor}`"
        );
    }
}
